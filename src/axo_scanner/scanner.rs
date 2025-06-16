use {
    super::{
        error::{CharParseError, ErrorKind},
        Operator, Punctuation, PunctuationKind, ScanError, Token, TokenKind,
    },
    crate::{
        axo_cursor::{Peekable, Position, Span},
        axo_form::{
            form::{Form, FormKind},
            former::Former,
            pattern::Pattern,
        },
        axo_text::unicode::{is_alphabetic, is_numeric},
        char::from_u32,
        compiler::Context,
        compiler::Marked,
        float::FloatLiteral,
        Path,
    },
};

#[derive(Clone)]
pub struct Scanner {
    pub context: Context,
    pub index: usize,
    pub position: Position,
    pub input: Vec<char>,
    pub output: Vec<Token>,
    pub errors: Vec<ScanError>,
}

impl Peekable<char> for Scanner {
    fn len(&self) -> usize {
        self.input.len()
    }

    fn peek_ahead(&self, n: usize) -> Option<&char> {
        let current = self.index + n;

        if current < self.input.len() {
            Some(&self.input[current])
        } else {
            None
        }
    }

    fn peek_behind(&self, n: usize) -> Option<&char> {
        let mut current = self.index;

        if current < n {
            return None;
        }

        current -= n;

        if current < self.input.len() {
            Some(&self.input[current])
        } else {
            None
        }
    }

    fn restore(&mut self) {
        self.set_position(Position {
            line: 1,
            column: 1,
            path: self.position.path.clone(),
        })
    }

    fn next(&mut self, position: &mut Position) -> Option<char> {
        if self.index < self.input.len() {
            let ch = self.input[self.index];

            if ch == '\n' {
                position.line += 1;
                position.column = 1;
            } else {
                position.column += 1;
            }

            Some(ch)
        } else {
            None
        }
    }

    fn input(&self) -> &[char] {
        self.input.as_slice()
    }

    fn input_mut(&mut self) -> &mut [char] {
        &mut self.input
    }

    fn position(&self) -> Position {
        self.position.clone()
    }

    fn position_mut(&mut self) -> &mut Position {
        &mut self.position
    }

    fn index(&self) -> usize {
        self.index
    }

    fn index_mut(&mut self) -> &mut usize {
        &mut self.index
    }
}

impl Scanner {
    pub fn new(context: Context, input: String, file: Path) -> Scanner {
        let chars: Vec<char> = input.chars().collect();

        Scanner {
            context,
            input: chars,
            index: 0,
            position: Position::new(file),
            output: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn create_span(&self, start: (usize, usize), end: (usize, usize)) -> Span {
        let file = self.position.path.clone();

        let start = Position {
            line: start.0,
            column: start.1,
            path: file.clone(),
        };

        let end = Position {
            line: end.0,
            column: end.1,
            path: file,
        };

        Span { start, end }
    }

    pub fn push_token(&mut self, kind: TokenKind, span: Span) {
        self.output.push(Token { kind, span });
    }

    fn line_comment() -> Pattern<char, Token, ScanError> {
        Pattern::sequence([
            Pattern::sequence([Pattern::exact('/'), Pattern::exact('/')]).with_ignore(),
            Pattern::repeat(Pattern::predicate(|c| *c != '\n'), 0, None),
        ])
        .with_transform(|_, form| {
            let content: String = form.inputs().into_iter().collect();

            Ok(Token::new(
                TokenKind::Comment(content.to_string()),
                form.span,
            ))
        })
    }

    fn multiline_comment() -> Pattern<char, Token, ScanError> {
        Pattern::sequence([
            Pattern::sequence([Pattern::exact('/'), Pattern::exact('*')]).with_ignore(),
            Pattern::repeat(
                Pattern::negate(
                    Pattern::sequence([Pattern::exact('*'), Pattern::exact('/')]).with_ignore(),
                ),
                0,
                None,
            ),
            Pattern::sequence([Pattern::exact('*'), Pattern::exact('/')]).with_ignore(),
        ])
        .with_transform(|_, form: Form<char, Token, ScanError>| {
            let content: String = form.inputs().into_iter().collect();

            Ok(Token::new(
                TokenKind::Comment(content.to_string()),
                form.span,
            ))
        })
    }

    fn hex_number() -> Pattern<char, Token, ScanError> {
        Pattern::transform(
            Pattern::sequence([
                Pattern::exact('0'),
                Pattern::alternative([Pattern::exact('x'), Pattern::exact('X')]),
                Pattern::repeat(
                    Pattern::alternative([
                        Pattern::predicate(|c| {
                            is_numeric(*c) || ('a'..='f').contains(c) || ('A'..='F').contains(c)
                        }),
                        Pattern::exact('_').with_ignore(),
                    ]),
                    1,
                    None,
                ),
            ]),
            |_, form| {
                let number: String = form.inputs().into_iter().collect();

                let parser = crate::axo_text::parser::<i128>();

                match parser.parse(&number) {
                    Ok(num) => Ok(Token::new(TokenKind::Integer(num), form.span)),
                    Err(e) => Err(ScanError::new(ErrorKind::NumberParse(e), form.span)),
                }
            },
        )
    }

    fn binary_number() -> Pattern<char, Token, ScanError> {
        Pattern::transform(
            Pattern::sequence([
                Pattern::exact('0'),
                Pattern::alternative([Pattern::exact('b'), Pattern::exact('B')]),
                Pattern::repeat(
                    Pattern::alternative([
                        Pattern::predicate(|c| *c == '0' || *c == '1'),
                        Pattern::exact('_').with_ignore(),
                    ]),
                    1,
                    None,
                ),
            ]),
            |_, form| {
                let number: String = form.inputs().into_iter().collect();

                let parser = crate::axo_text::parser::<i128>();
                match parser.parse(&number) {
                    Ok(num) => Ok(Token::new(TokenKind::Integer(num), form.span)),
                    Err(e) => Err(ScanError::new(ErrorKind::NumberParse(e), form.span)),
                }
            },
        )
    }

    fn octal_number() -> Pattern<char, Token, ScanError> {
        Pattern::transform(
            Pattern::sequence([
                Pattern::exact('0'),
                Pattern::alternative([Pattern::exact('o'), Pattern::exact('O')]),
                Pattern::repeat(
                    Pattern::alternative([
                        Pattern::predicate(|c| ('0'..='7').contains(c)),
                        Pattern::exact('_').with_ignore(),
                    ]),
                    1,
                    None,
                ),
            ]),
            |_, form| {
                let number: String = form.inputs().into_iter().collect();

                let parser = crate::axo_text::parser::<i128>();
                match parser.parse(&number) {
                    Ok(num) => Ok(Token::new(TokenKind::Integer(num), form.span)),
                    Err(e) => Err(ScanError::new(ErrorKind::NumberParse(e), form.span)),
                }
            },
        )
    }

    fn decimal_number() -> Pattern<char, Token, ScanError> {
        Pattern::transform(
            Pattern::sequence([
                Pattern::predicate(|c| is_numeric(*c)),
                Pattern::repeat(
                    Pattern::alternative([
                        Pattern::predicate(|c| is_numeric(*c)),
                        Pattern::exact('_').with_ignore(),
                    ]),
                    0,
                    None,
                ),
                Pattern::optional(Pattern::sequence([
                    Pattern::exact('.'),
                    Pattern::repeat(
                        Pattern::alternative([
                            Pattern::predicate(|c| is_numeric(*c)),
                            Pattern::exact('_').with_ignore(),
                        ]),
                        0,
                        None,
                    ),
                ])),
                Pattern::optional(Pattern::sequence([
                    Pattern::predicate(|c| *c == 'e' || *c == 'E'),
                    Pattern::optional(Pattern::predicate(|c| *c == '+' || *c == '-')),
                    Pattern::repeat(Pattern::predicate(|c| is_numeric(*c)), 1, None),
                ])),
            ]),
            |_, form| {
                let number: String = form.inputs().into_iter().collect();

                if number.contains('.') || number.to_lowercase().contains('e') {
                    let parser = crate::axo_text::parser::<f64>();
                    match parser.parse(&number) {
                        Ok(num) => Ok(Token::new(
                            TokenKind::Float(FloatLiteral::from(num)),
                            form.span,
                        )),
                        Err(e) => Err(ScanError::new(ErrorKind::NumberParse(e), form.span)),
                    }
                } else {
                    let parser = crate::axo_text::parser::<i128>();
                    match parser.parse(&number) {
                        Ok(num) => Ok(Token::new(TokenKind::Integer(num), form.span)),
                        Err(e) => Err(ScanError::new(ErrorKind::NumberParse(e), form.span)),
                    }
                }
            },
        )
    }

    fn number() -> Pattern<char, Token, ScanError> {
        Pattern::alternative([
            Self::hex_number(),
            Self::binary_number(),
            Self::octal_number(),
            Self::decimal_number(),
        ])
    }

    fn identifier() -> Pattern<char, Token, ScanError> {
        Pattern::transform(
            Pattern::sequence([
                Pattern::predicate(|c| is_alphabetic(*c) || *c == '_'),
                Pattern::repeat(
                    Pattern::predicate(|c| is_alphabetic(*c) || is_numeric(*c) || *c == '_'),
                    0,
                    None,
                ),
            ]),
            |_, form| {
                let identifier: String = form.inputs().into_iter().collect();

                Ok(Token::new(
                    TokenKind::from_str(&identifier).unwrap_or(TokenKind::Identifier(identifier)),
                    form.span,
                ))
            },
        )
    }

    fn quoted_string() -> Pattern<char, Token, ScanError> {
        Pattern::transform(
            Pattern::sequence([
                Pattern::exact('"'),
                Pattern::repeat(
                    Pattern::alternative([
                        Pattern::sequence([Pattern::exact('\\'), Pattern::predicate(|_| true)]),
                        Pattern::predicate(|c| *c != '"' && *c != '\\' && *c != '\n'),
                    ]),
                    0,
                    None,
                ),
                Pattern::exact('"'),
            ]),
            |_, form| {
                let mut content = String::new();
                let mut i = 1;

                let flat_chars = form.inputs();

                while i < flat_chars.len() - 1 {
                    let c = flat_chars[i];
                    if c == '\\' {
                        i += 1;
                        if i < flat_chars.len() - 1 {
                            let escaped_c = flat_chars[i];
                            content.push(match escaped_c {
                                'n' => '\n',
                                'r' => '\r',
                                't' => '\t',
                                '\\' => '\\',
                                '"' => '"',
                                '0' => '\0',
                                'x' => {
                                    i += 1;
                                    let mut hex = String::new();
                                    for _ in 0..2 {
                                        if i < flat_chars.len() - 1 {
                                            let hex_c = flat_chars[i];
                                            if hex_c.is_digit(16) {
                                                hex.push(hex_c);
                                                i += 1;
                                            } else {
                                                return Err(ScanError::new(
                                                    ErrorKind::StringParseError(
                                                        CharParseError::InvalidEscapeSequence,
                                                    ),
                                                    form.span,
                                                ));
                                            }
                                        } else {
                                            return Err(ScanError::new(
                                                ErrorKind::StringParseError(
                                                    CharParseError::UnterminatedEscapeSequence,
                                                ),
                                                form.span,
                                            ));
                                        }
                                    }
                                    i -= 1;
                                    u32::from_str_radix(&hex, 16)
                                        .ok()
                                        .and_then(from_u32)
                                        .unwrap_or('\0')
                                }
                                'u' => {
                                    i += 1;
                                    if i < flat_chars.len() - 1 {
                                        if flat_chars[i] == '{' {
                                            i += 1;
                                            let mut hex = String::new();
                                            while i < flat_chars.len() - 1 {
                                                let hex_c = flat_chars[i];
                                                if hex_c == '}' {
                                                    break;
                                                }
                                                hex.push(hex_c);
                                                i += 1;
                                            }
                                            if i < flat_chars.len() - 1 {
                                                if flat_chars[i] == '}' {
                                                    u32::from_str_radix(&hex, 16)
                                                        .ok()
                                                        .and_then(from_u32)
                                                        .unwrap_or('\0')
                                                } else {
                                                    return Err(ScanError::new(
                                                        ErrorKind::StringParseError(
                                                            CharParseError::InvalidEscapeSequence,
                                                        ),
                                                        form.span,
                                                    ));
                                                }
                                            } else {
                                                return Err(ScanError::new(
                                                    ErrorKind::StringParseError(
                                                        CharParseError::UnterminatedEscapeSequence,
                                                    ),
                                                    form.span,
                                                ));
                                            }
                                        } else {
                                            return Err(ScanError::new(
                                                ErrorKind::StringParseError(
                                                    CharParseError::InvalidEscapeSequence,
                                                ),
                                                form.span,
                                            ));
                                        }
                                    } else {
                                        return Err(ScanError::new(
                                            ErrorKind::StringParseError(
                                                CharParseError::UnterminatedEscapeSequence,
                                            ),
                                            form.span,
                                        ));
                                    }
                                }
                                _ => escaped_c,
                            });
                        }
                    } else {
                        content.push(c);
                    }
                    i += 1;
                }
                Ok(Token::new(TokenKind::String(content), form.span))
            },
        )
    }

    fn backtick_string() -> Pattern<char, Token, ScanError> {
        Pattern::transform(
            Pattern::sequence([
                Pattern::exact('`'),
                Pattern::repeat(Pattern::predicate(|c| *c != '`'), 0, None),
                Pattern::exact('`'),
            ]),
            |_, form| {
                let content: String = form.inputs().into_iter().collect();

                Ok(Token::new(TokenKind::String(content), form.span))
            },
        )
    }

    fn character() -> Pattern<char, Token, ScanError> {
        Pattern::transform(
            Pattern::sequence([
                Pattern::exact('\''),
                Pattern::alternative([
                    Pattern::sequence([Pattern::exact('\\'), Pattern::predicate(|_| true)]),
                    Pattern::predicate(|c| *c != '\'' && *c != '\\'),
                ]),
                Pattern::exact('\''),
            ]),
            |_, form| {
                let flat_chars = form.inputs();

                if flat_chars.len() < 3 {
                    return Err(ScanError::new(
                        ErrorKind::CharParseError(CharParseError::EmptyCharLiteral),
                        form.span,
                    ));
                }

                let ch = if flat_chars[1] == '\\' {
                    if flat_chars.len() < 4 {
                        return Err(ScanError::new(
                            ErrorKind::CharParseError(CharParseError::UnterminatedEscapeSequence),
                            form.span,
                        ));
                    }
                    let escaped_c = flat_chars[2];
                    match escaped_c {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        '\\' => '\\',
                        '\'' => '\'',
                        '0' => '\0',
                        'x' => {
                            if flat_chars.len() < 6 {
                                return Err(ScanError::new(
                                    ErrorKind::CharParseError(
                                        CharParseError::UnterminatedEscapeSequence,
                                    ),
                                    form.span,
                                ));
                            }
                            let h1 = flat_chars[3];
                            let h2 = flat_chars[4];
                            if h1.is_digit(16) && h2.is_digit(16) {
                                let hex = format!("{}{}", h1, h2);
                                u32::from_str_radix(&hex, 16)
                                    .ok()
                                    .and_then(from_u32)
                                    .unwrap_or('\0')
                            } else {
                                return Err(ScanError::new(
                                    ErrorKind::CharParseError(
                                        CharParseError::InvalidEscapeSequence,
                                    ),
                                    form.span,
                                ));
                            }
                        }
                        'u' => {
                            if flat_chars.len() < 5 || flat_chars[3] != '{' {
                                return Err(ScanError::new(
                                    ErrorKind::CharParseError(
                                        CharParseError::InvalidEscapeSequence,
                                    ),
                                    form.span,
                                ));
                            }
                            let mut i = 4;
                            let mut hex = String::new();
                            while i < flat_chars.len() && flat_chars[i] != '}' {
                                hex.push(flat_chars[i]);
                                i += 1;
                            }
                            if i >= flat_chars.len() || flat_chars[i] != '}' {
                                return Err(ScanError::new(
                                    ErrorKind::CharParseError(
                                        CharParseError::UnterminatedEscapeSequence,
                                    ),
                                    form.span,
                                ));
                            }
                            u32::from_str_radix(&hex, 16)
                                .ok()
                                .and_then(from_u32)
                                .unwrap_or('\0')
                        }
                        _ => escaped_c,
                    }
                } else {
                    flat_chars[1]
                };

                Ok(Token::new(TokenKind::Character(ch), form.span))
            },
        )
    }

    fn operator() -> Pattern<char, Token, ScanError> {
        Pattern::transform(
            Pattern::repeat(Pattern::predicate(|c: &char| c.is_operator()), 1, None),
            |_, form| {
                let operator: String = form.inputs().into_iter().collect();

                Ok(Token::new(
                    TokenKind::Operator(operator.to_operator()),
                    form.span,
                ))
            },
        )
    }

    fn punctuation() -> Pattern<char, Token, ScanError> {
        Pattern::transform(
            Pattern::predicate(|c: &char| c.is_punctuation()),
            |_, form| {
                let punctuation: String = form.inputs().into_iter().collect();

                Ok(Token::new(
                    TokenKind::Punctuation(punctuation.to_punctuation()),
                    form.span,
                ))
            },
        )
    }

    fn whitespace() -> Pattern<char, Token, ScanError> {
        Pattern::transform(
            Pattern::repeat(
                Pattern::predicate(|c: &char| c.is_whitespace() && *c != '\n'),
                1,
                None,
            ),
            |_, form| {
                let whitespace: String = form.inputs().into_iter().collect();

                if whitespace.len() == 1 {
                    Ok(Token::new(
                        TokenKind::Punctuation(PunctuationKind::Space),
                        form.span,
                    ))
                } else if whitespace.len() > 1 {
                    Ok(Token::new(
                        TokenKind::Punctuation(PunctuationKind::Indentation(whitespace.len())),
                        form.span,
                    ))
                } else {
                    unreachable!()
                }
            },
        )
    }

    fn fallback() -> Pattern<char, Token, ScanError> {
        Pattern::anything().with_ignore()
    }

    pub fn pattern() -> Pattern<char, Token, ScanError> {
        Pattern::repeat(
            Pattern::alternative([
                Self::whitespace(),
                Self::line_comment(),
                Self::multiline_comment(),
                Self::identifier(),
                Self::number(),
                Self::quoted_string(),
                Self::backtick_string(),
                Self::character(),
                Self::operator(),
                Self::punctuation(),
                Self::fallback(),
            ]),
            0,
            None,
        )
    }

    pub fn scan(&mut self) -> (Vec<Token>, Vec<ScanError>) {
        let mut tokens = Vec::new();
        let mut errors = Vec::new();

        while self.peek().is_some() {
            let form = self.form(Self::pattern());

            match form.kind {
                FormKind::Output(token) => {
                    tokens.push(token);
                }

                FormKind::Multiple(multi) => {
                    for item in multi {
                        match item.kind {
                            FormKind::Output(token) => {
                                tokens.push(token);
                            }
                            FormKind::Multiple(sub_multi) => {
                                for sub_item in sub_multi {
                                    if let FormKind::Output(token) = sub_item.kind {
                                        tokens.push(token);
                                    }
                                }
                            }
                            FormKind::Failure(err) => {
                                errors.push(err);
                            }
                            _ => {}
                        }
                    }
                }

                FormKind::Failure(err) => {
                    errors.push(err);
                }

                FormKind::Blank | FormKind::Input(_) => {}
            }
        }

        (tokens, errors)
    }
}

impl Marked for Scanner {
    fn context(&self) -> &Context {
        &self.context
    }

    fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }
}

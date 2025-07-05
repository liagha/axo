use {
    super::{
        error::{CharParseError, ErrorKind},
        character::Character,
        Operator, Punctuation, PunctuationKind, ScanError, Token, TokenKind,
    },
    crate::{
        char::from_u32,
        float::FloatLiteral,
        compiler::{
            Context, Marked,
        },
        axo_form::{
            form::{Form, FormKind},
            former::Former,
            pattern::Classifier,
        },
        axo_cursor::{
            Peekable, Position,
            Span,
        },
    },
};

#[derive(Clone)]
pub struct Scanner {
    pub context: Context,
    pub index: usize,
    pub position: Position,
    pub input: Vec<Character>,
    pub output: Vec<Token>,
    pub errors: Vec<ScanError>,
}

impl Peekable<Character> for Scanner {
    fn len(&self) -> usize {
        self.input.len()
    }

    fn peek_ahead(&self, n: usize) -> Option<&Character> {
        let current = self.index + n;

        self.get(current)
    }

    fn peek_behind(&self, n: usize) -> Option<&Character> {
        let current = self.index - n;

        self.get(current)
    }

    fn restore(&mut self) {
        self.set_position(Position {
            line: 1,
            column: 1,
            location: self.position.location,
        })
    }

    fn next(&self, index: &mut usize, position: &mut Position) -> Option<Character> {
        if let Some(ch) = self.get(*index) {
            if *ch == '\n' {
                position.line += 1;
                position.column = 1;
            } else {
                position.column += 1;
            }

            *index += 1;

            return Some(*ch);
        }

        None
    }

    fn input(&self) -> &Vec<Character> {
        &self.input
    }

    fn input_mut(&mut self) -> &mut Vec<Character> {
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
    pub fn inspect(start: Position, input: Vec<char>) -> Vec<Character> {
        let mut position = start;
        let mut characters = Vec::new();

        for char in input {
            let character = match char {
                '\n' => {
                    let start = position;
                    position.add_line(1);
                    position.set_column(1);
                        
                    Character {
                        value: char,
                        span: Span {
                            start,
                            end: position,
                        }
                    }
                }
                char => {
                    let start = position;
                    position.add_column(1);
                    
                    Character {
                        value: char,
                        span: Span {
                            start,
                            end: position,
                        }
                    }
                }
            };

            characters.push(character);
        }

        characters
    }

    pub fn new(context: Context, input: String, file: &'static str) -> Scanner {
        let start = Position::new(file);
        let chars: Vec<char> = input.chars().collect();

        let characters = Self::inspect(start, chars);

        Scanner {
            context,
            input: characters,
            index: 0,
            position: Position::new(file),
            output: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn line_comment() -> Classifier<Character, Token, ScanError> {
        Classifier::sequence([
            Classifier::sequence([Classifier::literal('/'), Classifier::literal('/')]).with_ignore(),
            Classifier::repeat(Classifier::predicate(|c: &Character| *c != '\n'), 0, None),
        ])
        .with_transform(|_, form| {
            let content: String = form.inputs().into_iter().collect();

            Ok(Token::new(
                TokenKind::Comment(content.trim().to_string()),
                form.span,
            ))
        })
    }

    fn multiline_comment() -> Classifier<Character, Token, ScanError> {
        Classifier::sequence([
            Classifier::sequence([Classifier::literal('/'), Classifier::literal('*')]).with_ignore(),
            Classifier::repeat(
                Classifier::negate(
                    Classifier::sequence([Classifier::literal('*'), Classifier::literal('/')]).with_ignore(),
                ),
                0,
                None,
            ),
            Classifier::sequence([Classifier::literal('*'), Classifier::literal('/')]).with_ignore(),
        ])
        .with_transform(|_, form: Form<Character, Token, ScanError>| {
            let content: String = form.inputs().into_iter().collect();

            Ok(Token::new(
                TokenKind::Comment(content.trim().to_string()),
                form.span,
            ))
        })
    }

    fn hex_number() -> Classifier<Character, Token, ScanError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::literal('0'),
                Classifier::alternative([Classifier::literal('x'), Classifier::literal('X')]),
                Classifier::repeat(
                    Classifier::alternative([
                        Classifier::predicate(|c: &Character| {
                            c.is_alphanumeric()
                        }),
                        Classifier::literal('_').with_ignore(),
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

    fn binary_number() -> Classifier<Character, Token, ScanError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::literal('0'),
                Classifier::alternative([Classifier::literal('b'), Classifier::literal('B')]),
                Classifier::repeat(
                    Classifier::alternative([
                        Classifier::predicate(|c: &Character| *c == '0' || *c == '1'),
                        Classifier::literal('_').with_ignore(),
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

    fn octal_number() -> Classifier<Character, Token, ScanError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::literal('0'),
                Classifier::alternative([Classifier::literal('o'), Classifier::literal('O')]),
                Classifier::repeat(
                    Classifier::alternative([
                        Classifier::predicate(|c: &Character| ('0'..='7').contains(&c.value)),
                        Classifier::literal('_').with_ignore(),
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

    fn decimal_number() -> Classifier<Character, Token, ScanError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::predicate(|c: &Character| c.is_numeric()),
                Classifier::repeat(
                    Classifier::alternative([
                        Classifier::predicate(|c: &Character| c.is_numeric()),
                        Classifier::literal('_').with_ignore(),
                    ]),
                    0,
                    None,
                ),
                Classifier::optional(Classifier::sequence([
                    Classifier::literal('.'),
                    Classifier::repeat(
                        Classifier::alternative([
                            Classifier::predicate(|c: &Character| c.is_numeric()),
                            Classifier::literal('_').with_ignore(),
                        ]),
                        0,
                        None,
                    ),
                ])),
                Classifier::optional(Classifier::sequence([
                    Classifier::predicate(|c: &Character| *c == 'e' || *c == 'E'),
                    Classifier::optional(Classifier::predicate(|c: &Character| *c == '+' || *c == '-')),
                    Classifier::repeat(Classifier::predicate(|c: &Character| c.is_numeric()), 1, None),
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

    fn number() -> Classifier<Character, Token, ScanError> {
        Classifier::alternative([
            Self::hex_number(),
            Self::binary_number(),
            Self::octal_number(),
            Self::decimal_number(),
        ])
    }

    fn identifier() -> Classifier<Character, Token, ScanError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::predicate(|c: &Character| c.is_alphabetic() || *c == '_'),
                Classifier::repeat(
                    Classifier::predicate(|c: &Character| c.is_alphabetic() || c.is_numeric() || *c == '_'),
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

    fn quoted_string() -> Classifier<Character, Token, ScanError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::literal('"'),
                Classifier::repeat(
                    Classifier::alternative([
                        Classifier::sequence([Classifier::literal('\\'), Classifier::predicate(|_| true)]),
                        Classifier::predicate(|c: &Character| *c != '"' && *c != '\\' && *c != '\n'),
                    ]),
                    0,
                    None,
                ),
                Classifier::literal('"'),
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
                            let escaped_c = flat_chars[i].value;
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
                                                hex.push(hex_c.value);
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
                                                hex.push(hex_c.value);
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
                        content.push(c.value);
                    }
                    i += 1;
                }
                Ok(Token::new(TokenKind::String(content), form.span))
            },
        )
    }

    fn backtick_string() -> Classifier<Character, Token, ScanError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::literal('`'),
                Classifier::repeat(Classifier::predicate(|c: &Character| *c != '`'), 0, None),
                Classifier::literal('`'),
            ]),
            |_, form| {
                let content: String = form.inputs().into_iter().collect();

                Ok(Token::new(TokenKind::String(content), form.span))
            },
        )
    }

    fn character() -> Classifier<Character, Token, ScanError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::literal('\''),
                Classifier::alternative([
                    Classifier::sequence([Classifier::literal('\\'), Classifier::predicate(|_| true)]),
                    Classifier::predicate(|c: &Character| *c != '\'' && *c != '\\'),
                ]),
                Classifier::literal('\''),
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
                    let escaped_c = flat_chars[2].value;
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
                            let h1 = flat_chars[3].value;
                            let h2 = flat_chars[4].value;
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
                                hex.push(flat_chars[i].value);
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
                    flat_chars[1].value
                };

                Ok(Token::new(TokenKind::Character(ch), form.span))
            },
        )
    }

    fn operator() -> Classifier<Character, Token, ScanError> {
        Classifier::transform(
            Classifier::repeat(Classifier::predicate(|c: &Character| c.is_operator()), 1, None),
            |_, form| {
                let operator: String = form.inputs().into_iter().collect();

                Ok(Token::new(
                    TokenKind::Operator(operator.to_operator()),
                    form.span,
                ))
            },
        )
    }

    fn punctuation() -> Classifier<Character, Token, ScanError> {
        Classifier::transform(
            Classifier::predicate(|c: &Character| c.is_punctuation()),
            |_, form| {
                let punctuation: String = form.inputs().into_iter().collect();

                Ok(Token::new(
                    TokenKind::Punctuation(punctuation.to_punctuation()),
                    form.span,
                ))
            },
        )
    }

    fn whitespace() -> Classifier<Character, Token, ScanError> {
        Classifier::transform(
            Classifier::repeat(
                Classifier::predicate(|c: &Character| c.is_whitespace() && *c != '\n'),
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

    fn fallback() -> Classifier<Character, Token, ScanError> {
        Classifier::anything().with_ignore()
    }

    pub fn pattern() -> Classifier<Character, Token, ScanError> {
        Classifier::repeat(
            Classifier::alternative([
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

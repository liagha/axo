use {
    super::{
        error::{
            ErrorKind, CharParseError,
        },
        
        OperatorLexer, 
        PunctuationKind, PunctuationLexer, 
        Token, TokenKind,
        LexError,
    },

    crate::{
        Path,

        thread::Arc,
        char::from_u32,
        float::FloatLiteral,
        
        compiler::Context,

        axo_form::{
            former::Former,
            form::FormKind,
            
            pattern::Pattern,
        },

        axo_data::peekable::Peekable,
        axo_rune::unicode::{is_alphabetic, is_numeric},

        axo_span::{
            Position, Span
        },
    },
};
use crate::compiler::Marked;

#[derive(Clone)]
pub struct Lexer {
    pub context: Context,
    pub input: Vec<char>,
    pub index: usize,
    pub position: Position,
    pub output: Vec<Token>,
    pub errors: Vec<LexError>,
}

impl Peekable<char> for Lexer {
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
        self.restore_position(Position {
            line: 1,
            column: 1,
            file: self.position.file.clone(),
        })
    }

    fn next(&mut self) -> Option<char> {
        if self.index < self.input.len() {
            let ch = self.input[self.index];

            self.index += 1;

            if ch == '\n' {
                self.position.line += 1;
                self.position.column = 1;
            } else {
                self.position.column += 1;
            }

            Some(ch)
        } else {
            None
        }
    }

    fn position(&self) -> Position {
        self.position.clone()
    }

    fn set_index(&mut self, index: usize) {
        self.index = index
    }

    fn set_line(&mut self, line: usize) {
        self.position.line = line
    }

    fn set_column(&mut self, column: usize) {
        self.position.column = column
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }
}

impl Lexer {
    pub fn new(context: Context, input: String, file: Path) -> Lexer {
        let chars: Vec<char> = input.chars().collect();

        Lexer {
            context,
            input: chars,
            index: 0,
            position: Position::new(file),
            output: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn create_span(&self, start: (usize, usize), end: (usize, usize)) -> Span {
        let file = self.position.file.clone();

        let start = Position {
            line: start.0,
            column: start.1,
            file: file.clone(),
        };

        let end = Position {
            line: end.0,
            column: end.1,
            file,
        };

        Span { start, end }
    }

    pub fn push_token(&mut self, kind: TokenKind, span: Span) {
        self.output.push(Token { kind, span });
    }

    fn line_comment() -> Pattern<char, Token, LexError> {
        Pattern::sequence([
            Pattern::sequence([Pattern::exact('/'), Pattern::exact('/')]).with_ignore(),
            Pattern::repeat(Pattern::predicate(Arc::new(|c| *c != '\n')), 0, None),
        ])
            .with_transform(Arc::new(|_, form| {
                let content: String = form.inputs().into_iter().collect();

                Ok(Token::new(TokenKind::Comment(content.to_string()), form.span))
            }))
    }

    fn multiline_comment() -> Pattern<char, Token, LexError> {
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
            .with_transform(Arc::new(|_, form| {
                let content: String = form.inputs().into_iter().collect();

                Ok(Token::new(TokenKind::Comment(content.to_string()), form.span))
            }))
    }

    fn hex_number() -> Pattern<char, Token, LexError> {
        Pattern::transform(
            Pattern::sequence([
                Pattern::exact('0'),
                Pattern::alternative([Pattern::exact('x'), Pattern::exact('X')]),
                Pattern::repeat(
                    Pattern::alternative([
                        Pattern::predicate(Arc::new(|c| {
                            is_numeric(*c) || ('a'..='f').contains(c) || ('A'..='F').contains(c)
                        })),
                        Pattern::exact('_').with_ignore(),
                    ]),
                    1,
                    None,
                ),
            ]),
            Arc::new(|_, form| {
                let number: String = form.inputs().into_iter().collect();

                let parser = crate::axo_rune::parser::<i128>();

                match parser.parse(&number) {
                    Ok(num) => Ok(Token::new(TokenKind::Integer(num), form.span)),
                    Err(e) => Err(LexError::new(ErrorKind::NumberParse(e), form.span)),
                }
            }),
        )
    }

    fn binary_number() -> Pattern<char, Token, LexError> {
        Pattern::transform(
            Pattern::sequence([
                Pattern::exact('0'),
                Pattern::alternative([Pattern::exact('b'), Pattern::exact('B')]),
                Pattern::repeat(
                    Pattern::alternative([
                        Pattern::predicate(Arc::new(|c| *c == '0' || *c == '1')),
                        Pattern::exact('_').with_ignore(),
                    ]),
                    1,
                    None,
                ),
            ]),
            Arc::new(|_, form| {
                let number: String = form.inputs().into_iter().collect();

                let parser = crate::axo_rune::parser::<i128>();
                match parser.parse(&number) {
                    Ok(num) => Ok(Token::new(TokenKind::Integer(num), form.span)),
                    Err(e) => Err(LexError::new(ErrorKind::NumberParse(e), form.span)),
                }
            }),
        )
    }

    fn octal_number() -> Pattern<char, Token, LexError> {
        Pattern::transform(
            Pattern::sequence([
                Pattern::exact('0'),
                Pattern::alternative([Pattern::exact('o'), Pattern::exact('O')]),
                Pattern::repeat(
                    Pattern::alternative([
                        Pattern::predicate(Arc::new(|c| ('0'..='7').contains(c))),
                        Pattern::exact('_').with_ignore(),
                    ]),
                    1,
                    None,
                ),
            ]),
            Arc::new(|_, form| {
                let number: String = form.inputs().into_iter().collect();

                let parser = crate::axo_rune::parser::<i128>();
                match parser.parse(&number) {
                    Ok(num) => Ok(Token::new(TokenKind::Integer(num), form.span)),
                    Err(e) => Err(LexError::new(ErrorKind::NumberParse(e), form.span)),
                }
            }),
        )
    }

    fn decimal_number() -> Pattern<char, Token, LexError> {
        Pattern::transform(
            Pattern::sequence([
                Pattern::predicate(Arc::new(|c| is_numeric(*c))),
                Pattern::repeat(
                    Pattern::alternative([
                        Pattern::predicate(Arc::new(|c| is_numeric(*c))),
                        Pattern::exact('_').with_ignore(),
                    ]),
                    0,
                    None,
                ),
                Pattern::optional(Pattern::sequence([
                    Pattern::exact('.'),
                    Pattern::repeat(
                        Pattern::alternative([
                            Pattern::predicate(Arc::new(|c| is_numeric(*c))),
                            Pattern::exact('_').with_ignore(),
                        ]),
                        0,
                        None,
                    ),
                ])),
                Pattern::optional(Pattern::sequence([
                    Pattern::predicate(Arc::new(|c| *c == 'e' || *c == 'E')),
                    Pattern::optional(Pattern::predicate(Arc::new(|c| *c == '+' || *c == '-'))),
                    Pattern::repeat(Pattern::predicate(Arc::new(|c| is_numeric(*c))), 1, None),
                ])),
            ]),
            Arc::new(|_, form| {
                let number: String = form.inputs().into_iter().collect();

                if number.contains('.') || number.to_lowercase().contains('e') {
                    let parser = crate::axo_rune::parser::<f64>();
                    match parser.parse(&number) {
                        Ok(num) => Ok(Token::new(TokenKind::Float(FloatLiteral::from(num)), form.span)),
                        Err(e) => Err(LexError::new(ErrorKind::NumberParse(e), form.span)),
                    }
                } else {
                    let parser = crate::axo_rune::parser::<i128>();
                    match parser.parse(&number) {
                        Ok(num) => Ok(Token::new(TokenKind::Integer(num), form.span)),
                        Err(e) => Err(LexError::new(ErrorKind::NumberParse(e), form.span)),
                    }
                }
            }),
        )
    }

    fn number() -> Pattern<char, Token, LexError> {
        Pattern::alternative([
            Self::hex_number(),
            Self::binary_number(),
            Self::octal_number(),
            Self::decimal_number(),
        ])
    }

    fn identifier() -> Pattern<char, Token, LexError> {
        Pattern::transform(
            Pattern::sequence([
                Pattern::predicate(Arc::new(|c| is_alphabetic(*c) || *c == '_')),
                Pattern::repeat(
                    Pattern::predicate(Arc::new(|c| {
                        is_alphabetic(*c) || is_numeric(*c) || *c == '_'
                    })),
                    0,
                    None,
                ),
            ]),
            Arc::new(|_, form| {
                let identifier: String = form.inputs().into_iter().collect();

                Ok(Token::new(
                    TokenKind::from_str(&identifier).unwrap_or(TokenKind::Identifier(identifier)),
                    form.span,
                ))
            }),
        )
    }

    fn quoted_string() -> Pattern<char, Token, LexError> {
        Pattern::transform(
            Pattern::sequence([
                Pattern::exact('"'),
                Pattern::repeat(
                    Pattern::alternative([
                        Pattern::sequence([
                            Pattern::exact('\\'),
                            Pattern::predicate(Arc::new(|_| true)),
                        ]),
                        Pattern::predicate(Arc::new(|c| *c != '"' && *c != '\\' && *c != '\n')),
                    ]),
                    0,
                    None,
                ),
                Pattern::exact('"'),
            ]),
            Arc::new(|_, form| {
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
                                                return Err(LexError::new(
                                                    ErrorKind::StringParseError(
                                                        CharParseError::InvalidEscapeSequence,
                                                    ),
                                                    form.span,
                                                ));
                                            }
                                        } else {
                                            return Err(LexError::new(
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
                                                    return Err(LexError::new(
                                                        ErrorKind::StringParseError(
                                                            CharParseError::InvalidEscapeSequence,
                                                        ),
                                                        form.span,
                                                    ));
                                                }
                                            } else {
                                                return Err(LexError::new(
                                                    ErrorKind::StringParseError(
                                                        CharParseError::UnterminatedEscapeSequence,
                                                    ),
                                                    form.span,
                                                ));
                                            }
                                        } else {
                                            return Err(LexError::new(
                                                ErrorKind::StringParseError(
                                                    CharParseError::InvalidEscapeSequence,
                                                ),
                                                form.span,
                                            ));
                                        }
                                    } else {
                                        return Err(LexError::new(
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
            }),
        )
    }

    fn backtick_string() -> Pattern<char, Token, LexError> {
        Pattern::transform(
            Pattern::sequence([
                Pattern::exact('`'),
                Pattern::repeat(Pattern::predicate(Arc::new(|c| *c != '`')), 0, None),
                Pattern::exact('`'),
            ]),
            Arc::new(|_, form| {
                let content: String = form.inputs().into_iter().collect();

                Ok(Token::new(TokenKind::String(content), form.span))
            }),
        )
    }

    fn character() -> Pattern<char, Token, LexError> {
        Pattern::transform(
            Pattern::sequence([
                Pattern::exact('\''),
                Pattern::alternative([
                    Pattern::sequence([
                        Pattern::exact('\\'),
                        Pattern::predicate(Arc::new(|_| true)),
                    ]),
                    Pattern::predicate(Arc::new(|c| *c != '\'' && *c != '\\')),
                ]),
                Pattern::exact('\''),
            ]),
            Arc::new(|_, form| {
                let flat_chars = form.inputs();

                if flat_chars.len() < 3 {
                    return Err(LexError::new(
                        ErrorKind::CharParseError(CharParseError::EmptyCharLiteral),
                        form.span,
                    ));
                }

                let ch = if flat_chars[1] == '\\' {
                    if flat_chars.len() < 4 {
                        return Err(LexError::new(
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
                                return Err(LexError::new(
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
                                return Err(LexError::new(
                                    ErrorKind::CharParseError(
                                        CharParseError::InvalidEscapeSequence,
                                    ),
                                    form.span,
                                ));
                            }
                        }
                        'u' => {
                            if flat_chars.len() < 5 || flat_chars[3] != '{' {
                                return Err(LexError::new(
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
                                return Err(LexError::new(
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
            }),
        )
    }

    fn operator() -> Pattern<char, Token, LexError> {
        Pattern::transform(
            Pattern::repeat(
                Pattern::predicate(Arc::new(|c: &char| c.is_operator())),
                1,
                None,
            ),
            Arc::new(|_, form| {
                let operator: String = form.inputs().into_iter().collect();

                Ok(Token::new(
                    TokenKind::Operator(operator.to_operator()),
                    form.span,
                ))
            }),
        )
    }

    fn punctuation() -> Pattern<char, Token, LexError> {
        Pattern::transform(
            Pattern::predicate(Arc::new(|c: &char| c.is_punctuation())),
            Arc::new(|_, form| {
                let punctuation: String = form.inputs().into_iter().collect();

                Ok(Token::new(
                    TokenKind::Punctuation(punctuation.to_punctuation()),
                    form.span,
                ))
            }),
        )
    }

    fn whitespace() -> Pattern<char, Token, LexError> {
        Pattern::transform(
            Pattern::repeat(
                Pattern::predicate(Arc::new(|c: &char| c.is_whitespace())),
                1,
                None,
            ),
            Arc::new(|_, form| {
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
                    println!("--------- {:?}", whitespace);
                    unreachable!()
                }
            })
        )
    }

    fn fallback() -> Pattern<char, Token, LexError> {
        Pattern::anything().with_ignore()
    }

    pub fn pattern() -> Pattern<char, Token, LexError> {
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

    pub fn lex(&mut self) -> (Vec<Token>, Vec<LexError>) {
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

                FormKind::Empty | FormKind::Input(_) => {}
            }
        }

        (tokens, errors)
    }
}

impl Marked for Lexer {
    fn context(&self) -> &Context {
        &self.context
    }
    
    fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }
}
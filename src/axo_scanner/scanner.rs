use {
    super::{
        error::{CharParseError, ErrorKind},
        character::Character,
        Operator, Punctuation, PunctuationKind, ScanError, Token, TokenKind,
    },
    crate::{
        character::from_u32,
        float::FloatLiteral,
        compiler::{
            Context, Marked,
        },
        axo_text::{
            parser,
        },
        axo_form::{
            form::{Form, FormKind},
            former::Former,
            pattern::Classifier,
        },
        axo_cursor::{
            Peekable, Position,
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
        self.get(self.index + n)
    }

    fn peek_behind(&self, n: usize) -> Option<&Character> {
        self.index.checked_sub(n).and_then(|idx| self.get(idx))
    }

    fn restore(&mut self) {
        self.set_position(Position {
            line: 1,
            column: 1,
            location: self.position.location,
        })
    }

    fn next(&self, index: &mut usize, position: &mut Position) -> Option<Character> {
        let ch = self.get(*index)?;

        if *ch == '\n' {
            position.line += 1;
            position.column = 1;
        } else {
            position.column += 1;
        }

        *index += 1;
        Some(*ch)
    }

    fn input(&self) -> &Vec<Character> {
        &self.input
    }

    fn input_mut(&mut self) -> &mut Vec<Character> {
        &mut self.input
    }

    fn position(&self) -> Position {
        self.position
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

    fn hexadecimal() -> Classifier<Character, Token, ScanError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::literal('0'),
                Classifier::alternative([Classifier::literal('x'), Classifier::literal('X')]),
                Classifier::repeat(
                    Classifier::alternative([
                        Classifier::predicate(|c: &Character| c.is_alphanumeric()),
                        Classifier::literal('_').with_ignore(),
                    ]),
                    1,
                    None,
                ),
            ]),
            |_, form| {
                let number: String = form.inputs().into_iter().collect();
                let parser = parser::<i128>();

                parser.parse(&number)
                    .map(|num| Token::new(TokenKind::Integer(num), form.span))
                    .map_err(|e| ScanError::new(ErrorKind::NumberParse(e), form.span))
            },
        )
    }

    fn binary() -> Classifier<Character, Token, ScanError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::literal('0'),
                Classifier::alternative([Classifier::literal('b'), Classifier::literal('B')]),
                Classifier::repeat(
                    Classifier::alternative([
                        Classifier::predicate(|c: &Character| matches!(c.value, '0' | '1')),
                        Classifier::literal('_').with_ignore(),
                    ]),
                    1,
                    None,
                ),
            ]),
            |_, form| {
                let number: String = form.inputs().into_iter().collect();
                let parser = parser::<i128>();

                parser.parse(&number)
                    .map(|num| Token::new(TokenKind::Integer(num), form.span))
                    .map_err(|e| ScanError::new(ErrorKind::NumberParse(e), form.span))
            },
        )
    }

    fn octal() -> Classifier<Character, Token, ScanError> {
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
                let parser = parser::<i128>();

                parser.parse(&number)
                    .map(|num| Token::new(TokenKind::Integer(num), form.span))
                    .map_err(|e| ScanError::new(ErrorKind::NumberParse(e), form.span))
            },
        )
    }

    fn decimal() -> Classifier<Character, Token, ScanError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::optional(
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
                    ])
                ),
                Classifier::optional(Classifier::sequence([
                    Classifier::literal('.'),
                    Classifier::repeat(
                        Classifier::alternative([
                            Classifier::predicate(|c: &Character| c.is_numeric()),
                            Classifier::literal('_').with_ignore(),
                        ]),
                        1,
                        None,
                    ),
                ])),
                Classifier::optional(Classifier::sequence([
                    Classifier::predicate(|c: &Character| matches!(c.value, 'e' | 'E')),
                    Classifier::optional(Classifier::predicate(|c: &Character| matches!(c.value, '+' | '-'))),
                    Classifier::repeat(Classifier::predicate(|c: &Character| c.is_numeric()), 1, None),
                ])),
            ]),
            |_, form| {
                let number: String = form.inputs().into_iter().collect();

                if number.contains('.') || number.to_lowercase().contains('e') {
                    let parser = parser::<f64>();
                    parser.parse(&number)
                        .map(|num| Token::new(TokenKind::Float(FloatLiteral::from(num)), form.span))
                        .map_err(|e| ScanError::new(ErrorKind::NumberParse(e), form.span))
                } else {
                    let parser = parser::<i128>();
                    parser.parse(&number)
                        .map(|num| Token::new(TokenKind::Integer(num), form.span))
                        .map_err(|e| ScanError::new(ErrorKind::NumberParse(e), form.span))
                }
            },
        )
    }

    fn number() -> Classifier<Character, Token, ScanError> {
        Classifier::alternative([
            Self::hexadecimal(),
            Self::binary(),
            Self::octal(),
            Self::decimal(),
        ])
    }

    fn string() -> Classifier<Character, Token, ScanError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::literal('"'),
                Classifier::repeat(
                    Classifier::alternative([
                        Classifier::sequence([Classifier::literal('\\'), Classifier::predicate(|_| true)]),
                        Classifier::predicate(|c: &Character| !matches!(c.value, '"' | '\\' | '\n')),
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
                            let escaped = flat_chars[i].value;
                            content.push(match escaped {
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
                                            let hex_char = flat_chars[i];
                                            if hex_char.is_digit(16) {
                                                hex.push(hex_char.value);
                                                i += 1;
                                            } else {
                                                return Err(ScanError::new(
                                                    ErrorKind::StringParseError(CharParseError::InvalidEscapeSequence),
                                                    form.span,
                                                ));
                                            }
                                        } else {
                                            return Err(ScanError::new(
                                                ErrorKind::StringParseError(CharParseError::UnterminatedEscapeSequence),
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
                                    if i < flat_chars.len() - 1 && flat_chars[i] == '{' {
                                        i += 1;
                                        let mut hex = String::new();
                                        while i < flat_chars.len() - 1 && flat_chars[i] != '}' {
                                            hex.push(flat_chars[i].value);
                                            i += 1;
                                        }
                                        if i < flat_chars.len() - 1 && flat_chars[i] == '}' {
                                            u32::from_str_radix(&hex, 16)
                                                .ok()
                                                .and_then(from_u32)
                                                .unwrap_or('\0')
                                        } else {
                                            return Err(ScanError::new(
                                                ErrorKind::StringParseError(CharParseError::UnterminatedEscapeSequence),
                                                form.span,
                                            ));
                                        }
                                    } else {
                                        return Err(ScanError::new(
                                            ErrorKind::StringParseError(CharParseError::InvalidEscapeSequence),
                                            form.span,
                                        ));
                                    }
                                }
                                c => c,
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

    fn backtick() -> Classifier<Character, Token, ScanError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::literal('`'),
                Classifier::repeat(
                    Classifier::predicate(|c: &Character| *c != '`'),
                    0,
                    None
                ),
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
                    Classifier::predicate(|c: &Character| !matches!(c.value, '\'' | '\\')),
                ]),
                Classifier::literal('\''),
            ]),
            |_, form| {
                let flat_chars = form.inputs();

                let ch = if flat_chars[1] == '\\' {
                    if flat_chars.len() < 4 {
                        return Err(ScanError::new(
                            ErrorKind::CharParseError(CharParseError::UnterminatedEscapeSequence),
                            form.span,
                        ));
                    }
                    let escaped = flat_chars[2].value;
                    match escaped {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        '\\' => '\\',
                        '\'' => '\'',
                        '0' => '\0',
                        'x' => {
                            if flat_chars.len() < 6 {
                                return Err(ScanError::new(
                                    ErrorKind::CharParseError(CharParseError::UnterminatedEscapeSequence),
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
                                    ErrorKind::CharParseError(CharParseError::InvalidEscapeSequence),
                                    form.span,
                                ));
                            }
                        }
                        'u' => {
                            if flat_chars.len() < 5 || flat_chars[3] != '{' {
                                return Err(ScanError::new(
                                    ErrorKind::CharParseError(CharParseError::InvalidEscapeSequence),
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
                                    ErrorKind::CharParseError(CharParseError::UnterminatedEscapeSequence),
                                    form.span,
                                ));
                            }
                            u32::from_str_radix(&hex, 16)
                                .ok()
                                .and_then(from_u32)
                                .unwrap_or('\0')
                        }
                        c => c,
                    }
                } else {
                    flat_chars[1].value
                };

                Ok(Token::new(TokenKind::Character(ch), form.span))
            },
        )
    }

    fn identifier() -> Classifier<Character, Token, ScanError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::predicate(|c: &Character| c.is_alphabetic() || *c == '_'),
                Classifier::repeat(
                    Classifier::predicate(|c: &Character| c.is_alphanumeric() || *c == '_'),
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

    fn operator() -> Classifier<Character, Token, ScanError> {
        Classifier::transform(
            Classifier::repeat(
                Classifier::predicate(|c: &Character| c.is_operator()),
                1,
                None
            ),
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
                let kind = match whitespace.len() {
                    1 => TokenKind::Punctuation(PunctuationKind::Space),
                    len => TokenKind::Punctuation(PunctuationKind::Indentation(len)),
                };

                Ok(Token::new(kind, form.span))
            },
        )
    }

    fn comment() -> Classifier<Character, Token, ScanError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::alternative([
                    Classifier::sequence([
                        Classifier::sequence([Classifier::literal('/'), Classifier::literal('/')]).with_ignore(),
                        Classifier::repeat(
                            Classifier::predicate(|c: &Character| *c != '\n'),
                            0,
                            None
                        ),
                    ]),
                    Classifier::sequence([
                        Classifier::sequence([Classifier::literal('/'), Classifier::literal('*')]).with_ignore(),
                        Classifier::repeat(
                            Classifier::negate(
                                Classifier::sequence([Classifier::literal('*'), Classifier::literal('/')]).with_ignore()
                            ),
                            0,
                            None,
                        ),
                        Classifier::sequence([Classifier::literal('*'), Classifier::literal('/')]).with_ignore(),
                    ])
                ])
            ]),
            |_, form| {
                let content: String = form.inputs().into_iter().collect();
                Ok(Token::new(
                    TokenKind::Comment(content.trim().to_string()),
                    form.span,
                ))
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
                Self::comment(),
                Self::identifier(),
                Self::number(),
                Self::string(),
                Self::backtick(),
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
            self.process_form(form, &mut tokens, &mut errors);
        }

        (tokens, errors)
    }

    fn process_form(&self, form: Form<Character, Token, ScanError>, tokens: &mut Vec<Token>, errors: &mut Vec<ScanError>) {
        match form.kind {
            FormKind::Output(token) => tokens.push(token),
            FormKind::Multiple(multi) => {
                for item in multi {
                    self.process_form(item, tokens, errors);
                }
            }
            FormKind::Failure(err) => errors.push(err),
            FormKind::Blank | FormKind::Input(_) => {}
        }
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
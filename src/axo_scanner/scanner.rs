use {
    super::{
        error::{ErrorKind},
        character::Character,
        Operator, Punctuation, PunctuationKind, ScanError, Token, TokenKind,
    },
    crate::{
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

    fn number() -> Classifier<Character, Token, ScanError> {
        Classifier::alternative([
            Self::hexadecimal(),
            Self::binary(),
            Self::octal(),
            Self::decimal(),
        ])
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
                    .map(|num| Form::output(Token::new(TokenKind::Integer(num), form.span)))
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
                    .map(|num| Form::output(Token::new(TokenKind::Integer(num), form.span)))
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
                    .map(|num| Form::output(Token::new(TokenKind::Integer(num), form.span)))
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
                        .map(|num| Form::output(Token::new(TokenKind::Float(num.into()), form.span)))
                        .map_err(|e| ScanError::new(ErrorKind::NumberParse(e), form.span))
                } else {
                    let parser = parser::<i128>();
                    parser.parse(&number)
                        .map(|num| Form::output(Token::new(TokenKind::Integer(num), form.span)))
                        .map_err(|e| ScanError::new(ErrorKind::NumberParse(e), form.span))
                }
            },
        )
    }

    fn string() -> Classifier<Character, Token, ScanError> {
        Classifier::sequence([
            Classifier::literal('"'),
            Classifier::repeat(
                Classifier::alternative([
                    Self::escape_sequence(),
                    Classifier::predicate(|c: &Character| !matches!(c.value, '"' | '\\')),
                ]),
                0,
                None,
            ),
            Classifier::literal('"'),
        ]).with_transform(|_, form| {
            let inputs = form.inputs();
            let content: String = inputs.iter()
                .skip(1)
                .take(inputs.len() - 2)
                .map(|c| c.value)
                .collect();

            Ok(Form::output(Token::new(TokenKind::String(content), form.span)))
        })
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
                Ok(Form::output(Token::new(TokenKind::String(content), form.span)))
            },
        )
    }

    fn character() -> Classifier<Character, Token, ScanError> {
        Classifier::sequence([
            Classifier::literal('\''),
            Classifier::alternative([
                Self::escape_sequence(),
                Classifier::predicate(|c: &Character| !matches!(c.value, '\'' | '\\')),
            ]),
            Classifier::literal('\''),
        ]).with_transform(|_, form| {
            let inputs = form.inputs();
            let ch = inputs[1].value;
            Ok(Form::output(Token::new(TokenKind::Character(ch), form.span)))
        })
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
                Ok(Form::output(
                    Token::new(
                        TokenKind::from_str(&identifier).unwrap_or(TokenKind::Identifier(identifier)),
                        form.span,
                    )
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
                Ok(Form::output(
                    Token::new(
                        TokenKind::Operator(operator.to_operator()),
                        form.span,
                    )
                ))
            },
        )
    }

    fn punctuation() -> Classifier<Character, Token, ScanError> {
        Classifier::transform(
            Classifier::predicate(|c: &Character| c.is_punctuation()),
            |_, form| {
                let punctuation: String = form.inputs().into_iter().collect();
                Ok(Form::output(
                    Token::new(
                        TokenKind::Punctuation(punctuation.to_punctuation()),
                        form.span,
                    )
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

                Ok(Form::output(Token::new(kind, form.span)))
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

                Ok(Form::output(Token::new(TokenKind::Comment(content), form.span)))
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
                Self::escape_sequence(),
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
            let forms = self.form(Self::pattern()).expand();

            for form in forms {
                match form.kind {
                    FormKind::Output(element) => {
                        tokens.push(element);
                    }

                    FormKind::Failure(error) => {
                        errors.push(error);
                    }

                    FormKind::Multiple(_) | FormKind::Blank | FormKind::Input(_) => {}
                }
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
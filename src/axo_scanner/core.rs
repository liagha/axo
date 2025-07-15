use crate::axo_form::form::Form;
use crate::axo_form::order::Order;
use crate::axo_form::pattern::Classifier;
use crate::axo_scanner::{Character, Operator, Punctuation, PunctuationKind, ScanError, Scanner, Token, TokenKind};
use crate::axo_scanner::error::{CharacterError, ErrorKind};
use crate::parser;

impl Scanner {
    fn number() -> Classifier<Character, Token, ScanError> {
        Classifier::alternative([
            Self::hexadecimal(),
            Self::binary(),
            Self::octal(),
            Self::decimal(),
        ])
    }

    fn hexadecimal() -> Classifier<Character, Token, ScanError> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::literal('0'),
                Classifier::alternative([Classifier::literal('x'), Classifier::literal('X')]),
                Classifier::persistence(
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
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::literal('0'),
                Classifier::alternative([Classifier::literal('b'), Classifier::literal('B')]),
                Classifier::persistence(
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
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::literal('0'),
                Classifier::alternative([Classifier::literal('o'), Classifier::literal('O')]),
                Classifier::persistence(
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
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|c: &Character| c.is_numeric()),
                Classifier::persistence(
                    Classifier::alternative([
                        Classifier::predicate(|c: &Character| c.is_numeric()),
                        Classifier::literal('_').with_ignore(),
                    ]),
                    0,
                    None,
                ),
                Classifier::optional(Classifier::sequence([
                    Classifier::literal('.'),
                    Classifier::persistence(
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
                    Classifier::persistence(Classifier::predicate(|c: &Character| c.is_numeric()), 1, None),
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
            Classifier::repetition(
                Classifier::alternative([
                    Classifier::predicate(|c: &Character| !matches!(c.value, '"' | '\\')),
                    Self::escape_sequence(),
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
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::literal('`'),
                Classifier::persistence(
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
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|c: &Character| c.is_alphabetic() || *c == '_'),
                Classifier::persistence(
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
        Classifier::with_transform(
            Classifier::persistence(
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
        Classifier::with_transform(
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
        Classifier::with_transform(
            Classifier::persistence(
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
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::alternative([
                    Classifier::sequence([
                        Classifier::sequence([Classifier::literal('/'), Classifier::literal('/')]).with_ignore(),
                        Classifier::persistence(
                            Classifier::predicate(|c: &Character| *c != '\n'),
                            0,
                            None
                        ),
                    ]),
                    Classifier::sequence([
                        Classifier::sequence([Classifier::literal('/'), Classifier::literal('*')]).with_ignore(),
                        Classifier::persistence(
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
        Classifier::with_order(
            Classifier::anything(),
            Order::fail(|_, form: Form<Character, Token, ScanError>| {
                ScanError::new(
                    ErrorKind::InvalidCharacter(CharacterError::Unexpected(form.inputs()[0].value)),
                    form.span,
                )
            }),
        )
    }

    pub fn pattern() -> Classifier<Character, Token, ScanError> {
        Classifier::persistence(
            Classifier::choice([
                Self::whitespace(),
                Self::comment(),
                Self::identifier(),
                Self::number(),
                Self::string(),
                Self::escape_sequence(),
                Self::backtick(),
                Self::character(),
                Self::operator(),
                Self::punctuation(),
                Self::fallback(),
            ], vec![1, 0]),
            0,
            None,
        )
    }

}
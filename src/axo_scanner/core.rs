use {
    super::{
        Character, Operator, Punctuation, PunctuationKind, ScanError, Scanner, Token, TokenKind,
        error::{CharacterError, ErrorKind},
    },
    crate::{
        axo_cursor::{
            Spanned,
        },
        axo_form::{
            form::Form,
            order::Order,
            pattern::Classifier,
        },
    }
};

impl Scanner {
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
            let inputs = form.collect_inputs();
            let content = inputs.clone().into_iter().collect::<String>();

            Ok(Form::output(Token::new(TokenKind::String(content), inputs.span())))
        })
    }

    fn backtick() -> Classifier<Character, Token, ScanError> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::literal('`'),
                Classifier::alternative([
                    Classifier::predicate(|c: &Character| !matches!(c.value, '`' | '\\')),
                    Self::escape_sequence(),
                ]),
                Classifier::literal('`'),
            ]),
            |_, form| {
                let inputs = form.collect_inputs();
                let content = inputs.clone().into_iter().collect::<String>();

                Ok(Form::output(Token::new(TokenKind::String(content), inputs.span())))
            },
        )
    }

    fn character() -> Classifier<Character, Token, ScanError> {
        Classifier::sequence([
            Classifier::literal('\''),
            Classifier::alternative([
                Classifier::predicate(|c: &Character| !matches!(c.value, '\'' | '\\')),
                Self::escape_sequence(),
            ]),
            Classifier::literal('\''),
        ]).with_transform(|_, form| {
            let inputs = form.collect_inputs();
            let ch = inputs[1];

            Ok(Form::output(Token::new(TokenKind::Character(ch.value), ch.span)))
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
                let inputs = form.collect_inputs();
                let content = inputs.clone().into_iter().collect::<String>();

                Ok(Form::output(
                    Token::new(
                        TokenKind::from_str(&content).unwrap_or(TokenKind::Identifier(content)),
                        inputs.span(),
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
                let inputs = form.collect_inputs();
                let content = inputs.clone().into_iter().collect::<String>();

                Ok(Form::output(
                    Token::new(
                        TokenKind::Operator(content.to_operator()),
                        inputs.span(),
                    )
                ))
            },
        )
    }

    fn punctuation() -> Classifier<Character, Token, ScanError> {
        Classifier::with_transform(
            Classifier::predicate(|c: &Character| c.is_punctuation()),
            |_, form| {
                let inputs = form.collect_inputs();
                let content = inputs.clone().into_iter().collect::<String>();

                Ok(Form::output(
                    Token::new(
                        TokenKind::Punctuation(content.to_punctuation()),
                        inputs.span(),
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
                let inputs = form.collect_inputs();
                let content = inputs.clone().into_iter().collect::<String>();

                let kind = match content.len() {
                    1 => TokenKind::Punctuation(PunctuationKind::Space),
                    len => TokenKind::Punctuation(PunctuationKind::Indentation(len)),
                };

                Ok(Form::output(Token::new(kind, inputs.span())))
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
                let inputs = form.collect_inputs();
                let content = inputs.clone().into_iter().collect::<String>();

                Ok(Form::output(Token::new(TokenKind::Comment(content), inputs.span())))
            },
        )
    }

    fn fallback() -> Classifier<Character, Token, ScanError> {
        Classifier::with_order(
            Classifier::anything(),
            Order::fail(|_, form| {
                let ch : &Character = form.unwrap_input();

                ScanError::new(
                    ErrorKind::InvalidCharacter(CharacterError::Unexpected(ch.value)),
                    ch.span,
                )
            }),
        )
    }

    pub fn pattern() -> Classifier<Character, Token, ScanError> {
        Classifier::persistence(
            Classifier::alternative([
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
            ]),
            0,
            None,
        )
    }

}
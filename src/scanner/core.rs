use {
    super::{
        Character, Operator, OperatorKind, Punctuation, PunctuationKind, ScanError, Scanner, Token, TokenKind, CharacterError, ErrorKind,
    },
    crate::{
        data::string::Str,
        formation::{form::Form, classifier::Classifier},
        tracker::{
            Spanned,
        },
    }
};

impl<'scanner> Scanner<'scanner> {
    fn string() -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>> {
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
        ]).with_transform(move |_, form: Form<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>>| {
            let inputs = form.collect_inputs();
            let content = inputs.clone().into_iter().collect::<Str>();

            Ok(Form::output(Token::new(TokenKind::String(content), inputs.borrow_span())))
        })
    }

    fn backtick() -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::literal('`'),
                Classifier::alternative([
                    Classifier::predicate(|c: &Character| !matches!(c.value, '`' | '\\')),
                    Self::escape_sequence(),
                ]),
                Classifier::literal('`'),
            ]),
            |_, form: Form<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>>| {
                let inputs = form.collect_inputs();
                let content = inputs.clone().into_iter().collect::<Str>();

                Ok(Form::output(Token::new(TokenKind::String(content), inputs.borrow_span())))
            },
        )
    }

    fn character() -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>> {
        Classifier::sequence([
            Classifier::literal('\''),
            Classifier::alternative([
                Classifier::predicate(|c: &Character| !matches!(c.value, '\'' | '\\')),
                Self::escape_sequence(),
            ]),
            Classifier::literal('\''),
        ]).with_transform(|_, form: Form<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>>| {
            let inputs = form.collect_inputs();
            let ch = inputs[1];

            Ok(Form::output(Token::new(TokenKind::Character(ch.value), ch.span)))
        })
    }
    
    fn identifier() -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|c: &Character| c.is_alphabetic() || *c == '_'),
                Classifier::persistence(
                    Classifier::predicate(|c: &Character| c.is_alphanumeric() || *c == '_'),
                    0,
                    None,
                ),
            ]),
            |_, form: Form<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>>| {
                let inputs = form.collect_inputs();
                let content = inputs.clone().into_iter().collect::<Str>();
                
                let token = match content.unwrap_str() {
                    "true" => {
                        TokenKind::Boolean(true)
                    }
                    "false" => {
                        TokenKind::Boolean(false)
                    }
                    "in" => {
                        TokenKind::Operator(OperatorKind::In)
                    }
                    identifier => {
                        TokenKind::Identifier(content)
                    }
                };

                Ok(Form::output(
                    Token::new(
                        token,
                        inputs.borrow_span(),
                    )
                ))
            },
        )
    }

    fn operator() -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>> {
        Classifier::with_transform(
            Classifier::persistence(
                Classifier::predicate(|c: &Character| c.is_operator()),
                1,
                None
            ),
            |_, form: Form<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>>| {
                let inputs = form.collect_inputs();
                let content = inputs.clone().into_iter().collect::<String>();

                Ok(Form::output(
                    Token::new(
                        TokenKind::Operator(content.to_operator()),
                        inputs.borrow_span(),
                    )
                ))
            },
        )
    }

    fn punctuation() -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>> {
        Classifier::with_transform(
            Classifier::predicate(|c: &Character| c.is_punctuation()),
            |_, form: Form<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>>| {
                let inputs = form.collect_inputs();
                let content = inputs.clone().into_iter().collect::<String>();

                Ok(Form::output(
                    Token::new(
                        TokenKind::Punctuation(content.to_punctuation()),
                        inputs.borrow_span(),
                    )
                ))
            },
        )
    }

    fn whitespace() -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>> {
        Classifier::with_transform(
            Classifier::persistence(
                Classifier::predicate(|c: &Character| c.is_whitespace() && *c != '\n'),
                1,
                None,
            ),
            |_, form: Form<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>>| {
                let inputs = form.collect_inputs();
                let content = inputs.clone().into_iter().collect::<String>();

                let kind = match content.len() {
                    1 => TokenKind::Punctuation(PunctuationKind::Space),
                    len => TokenKind::Punctuation(PunctuationKind::Indentation(len)),
                };

                Ok(Form::output(Token::new(kind, inputs.borrow_span())))
            },
        )
    }

    fn comment() -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>> {
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
            |_, form: Form<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>>| {
                let inputs = form.collect_inputs();
                let content = inputs.clone().into_iter().collect::<Str>();

                Ok(Form::output(Token::new(TokenKind::Comment(content), inputs.borrow_span())))
            },
        )
    }

    fn fallback() -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>> {
        Classifier::with_order(
            Classifier::anything(),
            Classifier::fail(|_, form| {
                let ch : &Character = form.unwrap_input();

                ScanError::new(
                    ErrorKind::InvalidCharacter(CharacterError::Unexpected(ch.value)),
                    ch.span,
                )
            }),
        )
    }

    pub fn classifier() -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>> {
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
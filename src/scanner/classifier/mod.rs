mod escape;
mod number;

use {
    super::{
        Character, CharacterError, ErrorKind, Operator, Punctuation, PunctuationKind,
        ScanError, Scanner, Token, TokenKind,
    },
    crate::{
        data::Str,
        formation::{classifier::Classifier, form::Form},
        tracker::Spanned,
    },
};

impl<'scanner> Scanner<'scanner> {
    fn string() -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>> {
        Classifier::sequence([
            Classifier::literal('"').with_ignore(),
            Classifier::repetition(
                Classifier::alternative([
                    Classifier::predicate(|c: &Character| !matches!(c.value, '"' | '\\')),
                    Self::escape_sequence(),
                ]),
                0,
                None,
            ),
            Classifier::literal('"').with_ignore(),
        ])
            .with_transform(
                move |classifier| {
                    let inputs = classifier.form.collect_inputs();
                    let content = inputs.clone().into_iter().collect::<Str>();

                    classifier.form = Form::output(Token::new(
                        TokenKind::String(content),
                        inputs.borrow_span(),
                    ));
                    
                    Ok(())
                },
            )
    }

    fn backtick() -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>>
    {
        Classifier::sequence([
            Classifier::literal('`').with_ignore(),
            Classifier::repetition(
                Classifier::alternative([
                    Classifier::predicate(|c: &Character| !matches!(c.value, '`' | '\\')),
                    Self::escape_sequence(),
                ]),
                0,
                None,
            ),
            Classifier::literal('`').with_ignore(),
        ])
            .with_transform(
                move |classifier| {
                    let inputs = classifier.form.collect_inputs();
                    let content = inputs.clone().into_iter().collect::<Str>();

                    classifier.form = Form::output(Token::new(
                        TokenKind::String(content),
                        inputs.borrow_span(),
                    ));
                    
                    Ok(())
                },
            )
    }

    fn character() -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>>
    {
        Classifier::sequence([
            Classifier::literal('\''),
            Classifier::alternative([
                Classifier::predicate(|c: &Character| !matches!(c.value, '\'' | '\\')),
                Self::escape_sequence(),
            ]),
            Classifier::literal('\''),
        ])
            .with_transform(
                |classifier| {
                    let inputs = classifier.form.collect_inputs();
                    let character = inputs[1];

                    classifier.form = Form::output(Token::new(
                        TokenKind::Character(character.value),
                        character.span,
                    ));
                    
                    Ok(())
                },
            )
    }

    fn identifier(
    ) -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|c: &Character| c.is_alphabetic() || *c == '_'),
                Classifier::persistence(
                    Classifier::predicate(|c: &Character| c.is_alphanumeric() || *c == '_'),
                    0,
                    None,
                ),
            ]),
            |classifier| {
                let inputs = classifier.form.collect_inputs();
                let content = inputs.clone().into_iter().collect::<Str>();

                let token = match content.unwrap_str() {
                    "true" => TokenKind::Boolean(true),
                    "false" => TokenKind::Boolean(false),
                    _ => TokenKind::Identifier(content),
                };
                
                classifier.form = Form::output(Token::new(token, inputs.borrow_span())); 

                Ok(())
            },
        )
    }

    fn operator() -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>>
    {
        Classifier::with_transform(
            Classifier::persistence(
                Classifier::predicate(|c: &Character| c.is_operator()),
                1,
                Some(3),
            ),
            |classifier| {
                let inputs = classifier.form.collect_inputs();
                let content = inputs.clone().into_iter().collect::<Str>();

                classifier.form = Form::output(Token::new(
                    TokenKind::Operator(content.to_operator()),
                    inputs.borrow_span(),
                ));
                
                Ok(())
            },
        )
    }

    fn punctuation(
    ) -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>> {
        Classifier::with_transform(
            Classifier::predicate(|c: &Character| c.is_punctuation()),
            |classifier| {
                let inputs = classifier.form.collect_inputs();
                let content = inputs.clone().into_iter().collect::<Str>();

                classifier.form = Form::output(Token::new(
                    TokenKind::Punctuation(content.to_punctuation()),
                    inputs.borrow_span(),
                ));
                
                Ok(())
            },
        )
    }

    fn whitespace(
    ) -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>> {
        Classifier::with_transform(
            Classifier::persistence(
                Classifier::predicate(|c: &Character| c.is_whitespace() && *c != '\n'),
                1,
                None,
            ),
            |classifier| {
                let inputs = classifier.form.collect_inputs();
                let content = inputs.clone().into_iter().collect::<Str>();

                let kind = match content.len() {
                    1 => TokenKind::Punctuation(PunctuationKind::Space),
                    len => TokenKind::Punctuation(PunctuationKind::Indentation(len)),
                };

                classifier.form = Form::output(Token::new(kind, inputs.borrow_span()));
                
                Ok(())
            },
        )
    }

    fn comment() -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>>
    {
        Classifier::with_transform(
            Classifier::sequence([Classifier::alternative([
                Classifier::sequence([
                    Classifier::sequence([Classifier::literal('/'), Classifier::literal('/')])
                        .with_ignore(),
                    Classifier::persistence(
                        Classifier::predicate(|c: &Character| *c != '\n'),
                        0,
                        None,
                    ),
                ]),
                Classifier::sequence([
                    Classifier::sequence([Classifier::literal('/'), Classifier::literal('*')])
                        .with_ignore(),
                    Classifier::persistence(
                        Classifier::negate(
                            Classifier::sequence([
                                Classifier::literal('*'),
                                Classifier::literal('/'),
                            ])
                                .with_ignore(),
                        ),
                        0,
                        None,
                    ),
                    Classifier::sequence([Classifier::literal('*'), Classifier::literal('/')])
                        .with_ignore(),
                ]),
            ])]),
            |classifier| {
                let inputs = classifier.form.collect_inputs();
                let content = inputs.clone().into_iter().collect::<Str>();

                classifier.form = Form::output(Token::new(
                    TokenKind::Comment(content),
                    inputs.borrow_span(),
                ));
                
                Ok(())
            },
        )
    }

    fn fallback() -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>>
    {
        Classifier::with_order(
            Classifier::anything(),
            Classifier::fail(|classifier| {
                let ch: &Character = classifier.form.unwrap_input();

                ScanError::new(
                    ErrorKind::InvalidCharacter(CharacterError::Unexpected(*ch)),
                    ch.span,
                )
            }),
        )
    }

    pub fn classifier(
    ) -> Classifier<'scanner, Character<'scanner>, Token<'scanner>, ScanError<'scanner>> {
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

use {
    crate::{
        formation::{
            classifier::Classifier,
            form::Form,
        },
        scanner::{PunctuationKind, Token, TokenKind},
        schema::Delimited,
        tracker::{Location, Span, Spanned},
    },
    super::{ErrorKind, Element, ElementKind, ParseError, Parser},
};

impl<'parser> Parser<'parser> {
    pub fn bundle(item: Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>) -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::LeftBrace)
                }),
                item.as_optional(),
                Classifier::persistence(
                    Classifier::sequence([
                        Classifier::with_fallback(
                            Classifier::predicate(|token: &Token| {
                                token.kind == TokenKind::Punctuation(PunctuationKind::Comma)
                            }),
                            Classifier::fail(|_form: Form<Token, Element, ParseError>| {
                                let span = Span::default(Location::Flag);
                                ParseError::new(
                                    ErrorKind::MissingSeparator(TokenKind::Punctuation(
                                        PunctuationKind::Comma,
                                    )),
                                    span,
                                )
                            }),
                        ),
                        item.as_optional(),
                    ]),
                    0,
                    None,
                ),
                Classifier::with_fallback(
                    Classifier::predicate(|token: &Token| {
                        token.kind == TokenKind::Punctuation(PunctuationKind::RightBrace)
                    }),
                    Classifier::fail(|_form: Form<Token, Element, ParseError>| {
                        let span = Span::default(Location::Flag);
                        ParseError::new(
                            ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                                PunctuationKind::LeftBrace,
                            )),
                            span,
                        )
                    }),
                ),
            ]),
            move |form| {
                let delimiters = form.collect_inputs();
                let elements = form.collect_outputs();
                let start = delimiters.first().unwrap();
                let end = delimiters.last().unwrap();
                let span = Span::merge(&start.span, &end.span);

                if delimiters.len() == 2 {
                    Ok(Form::output(Element::new(
                        ElementKind::delimited(Delimited::new(
                            start.clone(),
                            elements.clone(),
                            None,
                            end.clone(),
                        )),
                        span,
                    )))
                } else {
                    let separator = delimiters[1].clone();
                    Ok(Form::output(Element::new(
                        ElementKind::delimited(Delimited::new(
                            start.clone(),
                            elements.clone(),
                            Some(separator),
                            end.clone(),
                        )),
                        span,
                    )))
                }
            },
        )
    }

    pub fn block(item: Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>) -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::LeftBrace)
                }),
                item.as_optional(),
                Classifier::persistence(
                    Classifier::sequence([
                        Classifier::with_fallback(
                            Classifier::predicate(|token: &Token| {
                                token.kind == TokenKind::Punctuation(PunctuationKind::Semicolon)
                            }),
                            Classifier::fail(|_form: Form<Token, Element, ParseError>| {
                                let span = Span::default(Location::Flag);
                                ParseError::new(
                                    ErrorKind::MissingSeparator(TokenKind::Punctuation(
                                        PunctuationKind::Semicolon,
                                    )),
                                    span,
                                )
                            }),
                        ),
                        item.as_optional(),
                    ]),
                    0,
                    None,
                ),
                Classifier::with_fallback(
                    Classifier::predicate(|token: &Token| {
                        token.kind == TokenKind::Punctuation(PunctuationKind::RightBrace)
                    }),
                    Classifier::fail(|_form: Form<Token, Element, ParseError>| {
                        let span = Span::default(Location::Flag);
                        ParseError::new(
                            ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                                PunctuationKind::LeftBrace,
                            )),
                            span,
                        )
                    }),
                ),
            ]),
            move |form| {
                let delimiters = form.collect_inputs();
                let elements = form.collect_outputs();
                let start = delimiters.first().unwrap();
                let end = delimiters.last().unwrap();
                let span = Span::merge(&start.span, &end.span);

                if delimiters.len() == 2 {
                    Ok(Form::output(Element::new(
                        ElementKind::delimited(Delimited::new(
                            start.clone(),
                            elements.clone(),
                            None,
                            end.clone(),
                        )),
                        span,
                    )))
                } else {
                    let separator = delimiters[1].clone();
                    Ok(Form::output(Element::new(
                        ElementKind::delimited(Delimited::new(
                            start.clone(),
                            elements.clone(),
                            Some(separator),
                            end.clone(),
                        )),
                        span,
                    )))
                }
            },
        )
    }

    pub fn group(item: Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>) -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::LeftParenthesis)
                }),
                item.as_optional(),
                Classifier::persistence(
                    Classifier::sequence([
                        Classifier::with_order(
                            Classifier::predicate(|token: &Token| {
                                token.kind == TokenKind::Punctuation(PunctuationKind::Comma)
                            }),
                            Classifier::branch(
                                Classifier::ignore(),
                                Classifier::fail(|_form: Form<Token, Element, ParseError>| {
                                    let span = Span::default(Location::Flag);
                                    ParseError::new(
                                        ErrorKind::MissingSeparator(TokenKind::Punctuation(
                                            PunctuationKind::Comma,
                                        )),
                                        span,
                                    )
                                }),
                            ),
                        ),
                        item.as_optional(),
                    ]),
                    0,
                    None,
                ),
                Classifier::with_fallback(
                    Classifier::predicate(|token: &Token| {
                        token.kind == TokenKind::Punctuation(PunctuationKind::RightParenthesis)
                    }),
                    Classifier::fail(|_form: Form<Token, Element, ParseError>| {
                        let span = Span::default(Location::Flag);
                        ParseError::new(
                            ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                                PunctuationKind::LeftParenthesis,
                            )),
                            span,
                        )
                    }),
                ),
            ]),
            move |form| {
                let delimiters = form.collect_inputs();
                let elements = form.collect_outputs();
                let start = delimiters.first().unwrap();
                let end = delimiters.last().unwrap();
                let span = Span::merge(&start.span, &end.span);

                if delimiters.len() == 2 {
                    Ok(Form::output(Element::new(
                        ElementKind::delimited(Delimited::new(
                            start.clone(),
                            elements.clone(),
                            None,
                            end.clone(),
                        )),
                        span,
                    )))
                } else {
                    let separator = delimiters[1].clone();
                    Ok(Form::output(Element::new(
                        ElementKind::delimited(Delimited::new(
                            start.clone(),
                            elements.clone(),
                            Some(separator),
                            end.clone(),
                        )),
                        span,
                    )))
                }
            },
        )
    }

    pub fn sequence(item: Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>) -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::LeftParenthesis)
                }),
                item.as_optional(),
                Classifier::persistence(
                    Classifier::sequence([
                        Classifier::with_order(
                            Classifier::predicate(|token: &Token| {
                                token.kind == TokenKind::Punctuation(PunctuationKind::Semicolon)
                            }),
                            Classifier::branch(
                                Classifier::ignore(),
                                Classifier::fail(|_form: Form<Token, Element, ParseError>| {
                                    let span = Span::default(Location::Flag);
                                    ParseError::new(
                                        ErrorKind::MissingSeparator(TokenKind::Punctuation(
                                            PunctuationKind::Semicolon,
                                        )),
                                        span,
                                    )
                                }),
                            ),
                        ),
                        item.as_optional(),
                    ]),
                    0,
                    None,
                ),
                Classifier::with_fallback(
                    Classifier::predicate(|token: &Token| {
                        token.kind == TokenKind::Punctuation(PunctuationKind::RightParenthesis)
                    }),
                    Classifier::fail(|_form: Form<Token, Element, ParseError>| {
                        let span = Span::default(Location::Flag);
                        ParseError::new(
                            ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                                PunctuationKind::LeftParenthesis,
                            )),
                            span,
                        )
                    }),
                ),
            ]),
            move |form| {
                let delimiters = form.collect_inputs();
                let elements = form.collect_outputs();
                let start = delimiters.first().unwrap();
                let end = delimiters.last().unwrap();
                let span = Span::merge(&start.span, &end.span);

                if delimiters.len() == 2 {
                    Ok(Form::output(Element::new(
                        ElementKind::delimited(Delimited::new(
                            start.clone(),
                            elements.clone(),
                            None,
                            end.clone(),
                        )),
                        span,
                    )))
                } else {
                    let separator = delimiters[1].clone();
                    Ok(Form::output(Element::new(
                        ElementKind::delimited(Delimited::new(
                            start.clone(),
                            elements.clone(),
                            Some(separator),
                            end.clone(),
                        )),
                        span,
                    )))
                }
            },
        )
    }

    pub fn collection(item: Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>) -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::LeftBracket)
                }),
                item.as_optional(),
                Classifier::persistence(
                    Classifier::sequence([
                        Classifier::with_fallback(
                            Classifier::predicate(|token: &Token| {
                                token.kind == TokenKind::Punctuation(PunctuationKind::Comma)
                            }),
                            Classifier::fail(|_form: Form<Token, Element, ParseError>| {
                                let span = Span::default(Location::Flag);
                                ParseError::new(
                                    ErrorKind::MissingSeparator(TokenKind::Punctuation(
                                        PunctuationKind::Comma,
                                    )),
                                    span,
                                )
                            }),
                        ),
                        item.as_optional(),
                    ]),
                    0,
                    None,
                ),
                Classifier::with_fallback(
                    Classifier::predicate(|token: &Token| {
                        token.kind == TokenKind::Punctuation(PunctuationKind::RightBracket)
                    }),
                    Classifier::fail(|_form: Form<Token, Element, ParseError>| {
                        let span = Span::default(Location::Flag);
                        ParseError::new(
                            ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                                PunctuationKind::LeftBracket,
                            )),
                            span,
                        )
                    }),
                ),
            ]),
            move |form| {
                let delimiters = form.collect_inputs();
                let elements = form.collect_outputs();
                let start = delimiters.first().unwrap();
                let end = delimiters.last().unwrap();
                let span = Span::merge(&start.span, &end.span);

                if delimiters.len() == 2 {
                    Ok(Form::output(Element::new(
                        ElementKind::delimited(Delimited::new(
                            start.clone(),
                            elements.clone(),
                            None,
                            end.clone(),
                        )),
                        span,
                    )))
                } else {
                    let separator = delimiters[1].clone();
                    Ok(Form::output(Element::new(
                        ElementKind::delimited(Delimited::new(
                            start.clone(),
                            elements.clone(),
                            Some(separator),
                            end.clone(),
                        )),
                        span,
                    )))
                }
            },
        )
    }

    pub fn series(item: Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>) -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::LeftBracket)
                }),
                item.as_optional(),
                Classifier::persistence(
                    Classifier::sequence([
                        Classifier::with_fallback(
                            Classifier::predicate(|token: &Token| {
                                token.kind == TokenKind::Punctuation(PunctuationKind::Semicolon)
                            }),
                            Classifier::fail(|_form: Form<Token, Element, ParseError>| {
                                let span = Span::default(Location::Flag);
                                ParseError::new(
                                    ErrorKind::MissingSeparator(TokenKind::Punctuation(
                                        PunctuationKind::Semicolon,
                                    )),
                                    span,
                                )
                            }),
                        ),
                        item.as_optional(),
                    ]),
                    0,
                    None,
                ),
                Classifier::with_fallback(
                    Classifier::predicate(|token: &Token| {
                        token.kind == TokenKind::Punctuation(PunctuationKind::RightBracket)
                    }),
                    Classifier::fail(|_form: Form<Token, Element, ParseError>| {
                        let span = Span::default(Location::Flag);
                        ParseError::new(
                            ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                                PunctuationKind::LeftBracket,
                            )),
                            span,
                        )
                    }),
                ),
            ]),
            move |form| {
                let delimiters = form.collect_inputs();
                let elements = form.collect_outputs();
                let start = delimiters.first().unwrap();
                let end = delimiters.last().unwrap();
                let span = Span::merge(&start.span, &end.span);

                if delimiters.len() == 2 {
                    Ok(Form::output(Element::new(
                        ElementKind::delimited(Delimited::new(
                            start.clone(),
                            elements.clone(),
                            None,
                            end.clone(),
                        )),
                        span,
                    )))
                } else {
                    let separator = delimiters[1].clone();
                    Ok(Form::output(Element::new(
                        ElementKind::delimited(Delimited::new(
                            start.clone(),
                            elements.clone(),
                            Some(separator),
                            end.clone(),
                        )),
                        span,
                    )))
                }
            },
        )
    }

    pub fn delimited() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::alternative([
            Self::bundle(Classifier::deferred(Self::element)),
            Self::block(Classifier::deferred(Self::element)),
            Self::group(Classifier::deferred(Self::element)),
            Self::sequence(Classifier::deferred(Self::element)),
            Self::collection(Classifier::deferred(Self::element)),
            Self::series(Classifier::deferred(Self::element)),
        ])
    }
}
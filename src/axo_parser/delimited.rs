use {
    super::{error::ErrorKind, Element, ElementKind, ParseError, Parser},
    crate::{
        axo_cursor::{Span, Spanned},
        axo_form::{
            form::Form,
            classifier::Classifier
        },
        axo_scanner::{PunctuationKind, Token, TokenKind},
        axo_schema::{
            Group, Sequence,
            Collection, Series,
            Bundle, Block,
        },
        thread::Arc,
    },
};

impl<'parser> Parser<'parser> {
    pub fn bundle(item: Classifier<Token, Element, ParseError>) -> Classifier<Token, Element, ParseError> {
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
                            Classifier::fail(|_, form: Form<Token, Element, ParseError>| {
                                let span = form.collect_inputs().span();

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
                    Classifier::fail(|_, form: Form<Token, Element, ParseError>| {
                        let span = form.collect_inputs().span();

                        ParseError::new(
                            ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                                PunctuationKind::LeftBrace,
                            )),
                            span,
                        )
                    }),
                ),
            ]),
            move |_, form| {
                let braces = form.collect_inputs();
                let elements = form.collect_outputs();

                Ok(Form::output(
                    Element::new(ElementKind::bundle(Bundle::new(elements.clone())), braces.span())
                ))
            },
        )
    }

    pub fn block(item: Classifier<Token, Element, ParseError>) -> Classifier<Token, Element, ParseError> {
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
                            Classifier::fail(|_, form: Form<Token, Element, ParseError>| {
                                let span = form.collect_inputs().span();

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
                    Classifier::fail(|_, form: Form<Token, Element, ParseError>| {
                        let span = form.collect_inputs().span();

                        ParseError::new(
                            ErrorKind::UnclosedDelimiter(
                                TokenKind::Punctuation(PunctuationKind::LeftBrace),
                            ),
                            span,
                        )
                    }),
                ),
            ]),
            move |_, form| {
                let braces = form.collect_inputs();
                let elements = form.collect_outputs();

                Ok(Form::output(
                    Element::new(ElementKind::block(Block::new(elements.clone())), braces.span())
                ))
            },
        )
    }

    pub fn group(item: Classifier<Token, Element, ParseError>) -> Classifier<Token, Element, ParseError> {
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
                                Classifier::fail(|_, form: Form<Token, Element, ParseError>| {
                                    let span = form.collect_inputs().span();

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
                    Classifier::fail(|_, form: Form<Token, Element, ParseError>| {
                        let span = form.collect_inputs().span();
                        
                        ParseError::new(
                            ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                                PunctuationKind::LeftParenthesis,
                            )),
                            span,
                        )
                    }),
                ),
            ]),
            move |_, form| {
                let parentheses = form.collect_inputs();
                let elements = form.collect_outputs();

                Ok(Form::output(
                    Element::new(ElementKind::group(Group::new(elements.clone())), parentheses.span())
                ))
            },
        )
    }

    pub fn sequence(item: Classifier<Token, Element, ParseError>) -> Classifier<Token, Element, ParseError> {
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
                                Classifier::fail(|_, form: Form<Token, Element, ParseError>| {
                                    let span = form.collect_inputs().span();

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
                    Classifier::fail(|_, form: Form<Token, Element, ParseError>| {
                        let span = form.collect_inputs().span();

                        ParseError::new(
                            ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                                PunctuationKind::LeftParenthesis,
                            )),
                            span,
                        )
                    }),
                ),
            ]),
            move |_, form| {
                let parentheses = form.collect_inputs();
                let elements = form.collect_outputs();

                Ok(Form::output(
                    Element::new(ElementKind::sequence(Sequence::new(elements.clone())), parentheses.span())
                ))
            },
        )
    }

    pub fn collection(item: Classifier<Token, Element, ParseError>) -> Classifier<Token, Element, ParseError> {
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
                            Classifier::fail(|_, form: Form<Token, Element, ParseError>| {
                                let span = form.collect_inputs().span();

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
                    Classifier::fail(|_, form: Form<Token, Element, ParseError>| {
                        let span = form.collect_inputs().span();

                        ParseError::new(
                            ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                                PunctuationKind::LeftBracket,
                            )),
                            span,
                        )
                    }),
                ),
            ]),
            move |_, form| {
                let brackets = form.collect_inputs();
                let elements = form.collect_outputs();

                Ok(Form::output(
                    Element::new(ElementKind::collection(Collection::new(elements.clone())), brackets.span())
                ))
            },
        )
    }

    pub fn series(item: Classifier<Token, Element, ParseError>) -> Classifier<Token, Element, ParseError> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::LeftBracket)
                })
                .with_ignore(),
                item.as_optional(),
                Classifier::persistence(
                    Classifier::sequence([
                        Classifier::with_fallback(
                            Classifier::predicate(|token: &Token| {
                                token.kind == TokenKind::Punctuation(PunctuationKind::Semicolon)
                            }),
                            Classifier::fail(|_, form: Form<Token, Element, ParseError>| {
                                let span = form.collect_inputs().span();

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
                    Classifier::fail(|_, form: Form<Token, Element, ParseError>| {
                        let span = form.collect_inputs().span();

                        ParseError::new(
                            ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                                PunctuationKind::LeftBracket,
                            )),
                            span,
                        )
                    }),
                ),
            ]),
            move |_, form| {
                let brackets = form.collect_inputs();
                let elements = form.collect_outputs();

                Ok(Form::output(
                    Element::new(ElementKind::series(Series::new(elements.clone())), brackets.span())
                ))
            },
        )
    }

    pub fn delimited() -> Classifier<Token, Element, ParseError> {
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

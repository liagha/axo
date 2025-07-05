use {
    super::{error::ErrorKind, Element, ElementKind, ParseError, Parser},
    crate::{
        axo_cursor::{Span, Spanned},
        axo_form::{order::Order, form::Form, pattern::Classifier},
        axo_scanner::{PunctuationKind, Token, TokenKind},
        thread::Arc,
    },
};

impl Parser {
    pub fn bundle() -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::LeftBrace)
                })
                .with_ignore(),
                Classifier::lazy(Self::element).as_optional(),
                Classifier::repeat(
                    Classifier::sequence([
                        Classifier::required(
                            Classifier::predicate(|token: &Token| {
                                token.kind == TokenKind::Punctuation(PunctuationKind::Comma)
                            }),
                            Order::failure(|_, form| {
                                ParseError::new(
                                    ErrorKind::MissingSeparator(TokenKind::Punctuation(
                                        PunctuationKind::Comma,
                                    )),
                                    form.span,
                                )
                            }),
                        ),
                        Classifier::lazy(Self::element).as_optional(),
                    ]),
                    0,
                    None,
                ),
                Classifier::required(
                    Classifier::predicate(|token: &Token| {
                        token.kind == TokenKind::Punctuation(PunctuationKind::RightBrace)
                    }),
                    Order::failure(|_, form| {
                        ParseError::new(
                            ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                                PunctuationKind::LeftBrace,
                            )),
                            form.span,
                        )
                    }),
                ),
            ]),
            move |_, form| {
                let elements = form.outputs();

                Ok(Element::new(ElementKind::Bundle(elements), form.span))
            },
        )
    }

    pub fn scope() -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::LeftBrace)
                })
                .with_ignore(),
                Classifier::lazy(Self::element).as_optional(),
                Classifier::repeat(
                    Classifier::sequence([
                        Classifier::required(
                            Classifier::predicate(|token: &Token| {
                                token.kind == TokenKind::Punctuation(PunctuationKind::Semicolon)
                            }),
                            Order::failure(|_, form| {
                                ParseError::new(
                                    ErrorKind::MissingSeparator(TokenKind::Punctuation(
                                        PunctuationKind::Semicolon,
                                    )),
                                    form.span,
                                )
                            }),
                        ),
                        Classifier::lazy(Self::element).as_optional(),
                    ]),
                    0,
                    None,
                ),
                Classifier::required(
                    Classifier::predicate(|token: &Token| {
                        token.kind == TokenKind::Punctuation(PunctuationKind::RightBrace)
                    }),
                    Order::failure(|_, form| {
                        ParseError::new(
                            ErrorKind::UnclosedDelimiter(
                                TokenKind::Punctuation(PunctuationKind::LeftBrace),
                            ),
                            form.span,
                        )
                    }),
                ),
            ]),
            move |_, form| {
                let elements = form.outputs();

                Ok(Element::new(ElementKind::Scope(elements), form.span))
            },
        )
    }

    pub fn group() -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::LeftParenthesis)
                }),
                Classifier::lazy(Self::element).as_optional(),
                Classifier::repeat(
                    Classifier::sequence([
                        Classifier::order(
                            Classifier::predicate(|token: &Token| {
                                token.kind == TokenKind::Punctuation(PunctuationKind::Comma)
                            }),
                            Order::trigger(
                                Order::ignore(),
                                Order::failure(|_, form| {
                                    ParseError::new(
                                        ErrorKind::MissingSeparator(TokenKind::Punctuation(
                                            PunctuationKind::Comma,
                                        )),
                                        form.span,
                                    )
                                }),
                            ),
                        ),
                        Classifier::lazy(Self::element).as_optional(),
                    ]),
                    0,
                    None,
                ),
                Classifier::required(
                    Classifier::predicate(|token: &Token| {
                        token.kind == TokenKind::Punctuation(PunctuationKind::RightParenthesis)
                    }),
                    Order::failure(|_, form| {
                        ParseError::new(
                            ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                                PunctuationKind::LeftParenthesis,
                            )),
                            form.span,
                        )
                    }),
                ),
            ]),
            move |_, form| {
                let elements = form.outputs();

                Ok(Element::new(ElementKind::Group(elements), form.span))
            },
        )
    }

    pub fn sequence() -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::LeftParenthesis)
                })
                .with_ignore(),
                Classifier::lazy(Self::element).as_optional(),
                Classifier::repeat(
                    Classifier::sequence([
                        Classifier::order(
                            Classifier::predicate(|token: &Token| {
                                token.kind == TokenKind::Punctuation(PunctuationKind::Semicolon)
                            }),
                            Order::trigger(
                                Order::ignore(),
                                Order::failure(|_, form| {
                                    ParseError::new(
                                        ErrorKind::MissingSeparator(TokenKind::Punctuation(
                                            PunctuationKind::Semicolon,
                                        )),
                                        form.span,
                                    )
                                }),
                            ),
                        ),
                        Classifier::lazy(Self::element).as_optional(),
                    ]),
                    0,
                    None,
                ),
                Classifier::required(
                    Classifier::predicate(|token: &Token| {
                        token.kind == TokenKind::Punctuation(PunctuationKind::RightParenthesis)
                    }),
                    Order::failure(|_, form| {
                        ParseError::new(
                            ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                                PunctuationKind::LeftParenthesis,
                            )),
                            form.span,
                        )
                    }),
                ),
            ]),
            move |_, form| {
                let elements = form.outputs();

                Ok(Element::new(ElementKind::Group(elements), form.span))
            },
        )
    }

    pub fn collection() -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::LeftBracket)
                })
                .with_ignore(),
                Classifier::lazy(Self::element).as_optional(),
                Classifier::repeat(
                    Classifier::sequence([
                        Classifier::required(
                            Classifier::predicate(|token: &Token| {
                                token.kind == TokenKind::Punctuation(PunctuationKind::Comma)
                            }),
                            Order::failure(|_, form| {
                                ParseError::new(
                                    ErrorKind::MissingSeparator(TokenKind::Punctuation(
                                        PunctuationKind::Comma,
                                    )),
                                    form.span,
                                )
                            }),
                        ),
                        Classifier::lazy(Self::element).as_optional(),
                    ]),
                    0,
                    None,
                ),
                Classifier::required(
                    Classifier::predicate(|token: &Token| {
                        token.kind == TokenKind::Punctuation(PunctuationKind::RightBracket)
                    }),
                    Order::failure(|_, form| {
                        ParseError::new(
                            ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                                PunctuationKind::LeftBracket,
                            )),
                            form.span,
                        )
                    }),
                ),
            ]),
            move |_, form| {
                let elements = form.outputs();

                Ok(Element::new(ElementKind::Collection(elements), form.span))
            },
        )
    }

    pub fn series() -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::LeftBracket)
                })
                .with_ignore(),
                Classifier::lazy(Self::element).as_optional(),
                Classifier::repeat(
                    Classifier::sequence([
                        Classifier::required(
                            Classifier::predicate(|token: &Token| {
                                token.kind == TokenKind::Punctuation(PunctuationKind::Semicolon)
                            }),
                            Order::failure(|_, form| {
                                ParseError::new(
                                    ErrorKind::MissingSeparator(TokenKind::Punctuation(
                                        PunctuationKind::Semicolon,
                                    )),
                                    form.span,
                                )
                            }),
                        ),
                        Classifier::lazy(Self::element).as_optional(),
                    ]),
                    0,
                    None,
                ),
                Classifier::required(
                    Classifier::predicate(|token: &Token| {
                        token.kind == TokenKind::Punctuation(PunctuationKind::RightBracket)
                    }),
                    Order::failure(|_, form| {
                        ParseError::new(
                            ErrorKind::UnclosedDelimiter(TokenKind::Punctuation(
                                PunctuationKind::LeftBracket,
                            )),
                            form.span,
                        )
                    }),
                ),
            ]),
            move |_, form| {
                let elements = form.outputs();

                Ok(Element::new(ElementKind::Series(elements), form.span))
            },
        )
    }

    pub fn delimited() -> Classifier<Token, Element, ParseError> {
        Classifier::alternative([
            Self::bundle(),
            Self::scope(),
            Self::group(),
            Self::sequence(),
            Self::collection(),
            Self::series(),
        ])
    }
}

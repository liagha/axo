use {
    super::{error::ErrorKind, Element, ElementKind, ParseError, Parser},
    crate::{
        axo_cursor::{Span, Spanned},
        axo_form::{order::Order, form::Form, pattern::Classifier},
        axo_scanner::{PunctuationKind, Token, TokenKind},
        axo_schema::{
            Group, Sequence,
            Collection, Series,
            Bundle, Scope,
        },
        thread::Arc,
    },
};

impl Parser {
    pub fn bundle(item: Classifier<Token, Element, ParseError>) -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::LeftBrace)
                })
                .with_ignore(),
                item.as_optional(),
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
                        item.as_optional(),
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

                Ok(Form::output(
                    Element::new(ElementKind::Bundle(Bundle::new(elements)), form.span)
                ))
            },
        )
    }

    pub fn scope(item: Classifier<Token, Element, ParseError>) -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::LeftBrace)
                })
                .with_ignore(),
                item.as_optional(),
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
                        item.as_optional(),
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

                Ok(Form::output(
                    Element::new(ElementKind::Scope(Scope::new(elements)), form.span)
                ))
            },
        )
    }

    pub fn group(item: Classifier<Token, Element, ParseError>) -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::LeftParenthesis)
                }),
                item.as_optional(),
                Classifier::repeat(
                    Classifier::sequence([
                        Classifier::ordered(
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
                        item.as_optional(),
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

                Ok(Form::output(
                    Element::new(ElementKind::Group(Group::new(elements)), form.span)
                ))
            },
        )
    }

    pub fn sequence(item: Classifier<Token, Element, ParseError>) -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::LeftParenthesis)
                })
                .with_ignore(),
                item.as_optional(),
                Classifier::repeat(
                    Classifier::sequence([
                        Classifier::ordered(
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
                        item.as_optional(),
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

                Ok(Form::output(
                    Element::new(ElementKind::Sequence(Sequence::new(elements)), form.span)
                ))
            },
        )
    }

    pub fn collection(item: Classifier<Token, Element, ParseError>) -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::LeftBracket)
                })
                .with_ignore(),
                item.as_optional(),
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
                        item.as_optional(),
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

                Ok(Form::output(
                    Element::new(ElementKind::Collection(Collection::new(elements)), form.span)
                ))
            },
        )
    }

    pub fn series(item: Classifier<Token, Element, ParseError>) -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::LeftBracket)
                })
                .with_ignore(),
                item.as_optional(),
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
                        item.as_optional(),
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

                Ok(Form::output(
                    Element::new(ElementKind::Series(Series::new(elements)), form.span)
                ))
            },
        )
    }

    pub fn delimited() -> Classifier<Token, Element, ParseError> {
        Classifier::alternative([
            Self::bundle(Classifier::lazy(Self::element)),
            Self::scope(Classifier::lazy(Self::element)),
            Self::group(Classifier::lazy(Self::element)),
            Self::sequence(Classifier::lazy(Self::element)),
            Self::collection(Classifier::lazy(Self::element)),
            Self::series(Classifier::lazy(Self::element)),
        ])
    }
}

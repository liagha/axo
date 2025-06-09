use {
    super::{
        error::ErrorKind,

        Parser,
        Element, ElementKind,
        ParseError
    },

    crate::{
        Peekable,
        thread::Arc,

        axo_lexer::{
            Token, TokenKind,
            PunctuationKind,
        },

        axo_form::{
            pattern::Pattern,
            action::Action,
            form::Form,
        },

        axo_span::Span,
    }
};

impl Parser {
    pub fn bundle() -> Pattern<Token, Element, ParseError> {
        Pattern::transform(
            Pattern::sequence([
                Pattern::ignore(Pattern::predicate(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::LeftBrace)
                })),
                Pattern::lazy(|| Self::pattern()).optional_self(),
                Pattern::repeat(
                    Pattern::sequence([
                        Pattern::action(
                            Pattern::predicate(|token: &Token| {
                                token.kind == TokenKind::Punctuation(PunctuationKind::Comma)
                            }),
                            Action::trigger(
                                Action::Ignore,
                                Action::failure(
                                    |span| {
                                        ParseError::new(
                                            ErrorKind::MissingSeparator(TokenKind::Punctuation(PunctuationKind::Comma)),
                                            span,
                                        )
                                    }
                                )
                            )
                        ),
                        Pattern::lazy(|| Self::pattern()).optional_self(),
                    ]),
                    0,
                    None
                ),
                Pattern::required(
                    Pattern::predicate(|token: &Token| {
                        token.kind == TokenKind::Punctuation(PunctuationKind::RightBrace)
                    }),
                    Action::failure(|span| {
                        ParseError::new(
                            ErrorKind::UnclosedDelimiter(Token::new(TokenKind::Punctuation(
                                PunctuationKind::LeftBrace,
                            ), Span::default())),
                            span,
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

    pub fn scope() -> Pattern<Token, Element, ParseError> {
        Pattern::transform(
            Pattern::sequence([
                Pattern::ignore(Pattern::predicate(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::LeftBrace)
                })),
                Pattern::lazy(|| Self::pattern()).optional_self(),
                Pattern::repeat(
                    Pattern::sequence([
                        Pattern::action(
                            Pattern::predicate(|token: &Token| {
                                token.kind == TokenKind::Punctuation(PunctuationKind::Semicolon)
                            }),
                            Action::trigger(
                                Action::Ignore,
                                Action::failure(
                                    |span| {
                                        ParseError::new(
                                            ErrorKind::MissingSeparator(TokenKind::Punctuation(PunctuationKind::Semicolon)),
                                            span,
                                        )
                                    }
                                )
                            )
                        ),
                        Pattern::lazy(|| Self::pattern()).optional_self(),
                    ]),
                    0,
                    None
                ),
                Pattern::required(
                    Pattern::predicate(|token: &Token| {
                        token.kind == TokenKind::Punctuation(PunctuationKind::RightBrace)
                    }),
                    Action::failure(|span| {
                        ParseError::new(
                            ErrorKind::UnclosedDelimiter(Token::new(TokenKind::Punctuation(
                                PunctuationKind::LeftBrace,
                            ), Span::default())),
                            span,
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

    pub fn group() -> Pattern<Token, Element, ParseError> {
        Pattern::transform(
            Pattern::sequence([
                Pattern::ignore(Pattern::predicate(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::LeftParenthesis)
                })),
                Pattern::lazy(|| Self::pattern()).optional_self(),
                Pattern::repeat(
                    Pattern::sequence([
                        Pattern::action(
                            Pattern::predicate(|token: &Token| {
                                token.kind == TokenKind::Punctuation(PunctuationKind::Comma)
                            }),
                            Action::trigger(
                                Action::Ignore,
                                Action::failure(
                                    |span| {
                                        ParseError::new(
                                            ErrorKind::MissingSeparator(TokenKind::Punctuation(PunctuationKind::Comma)),
                                            span,
                                        )
                                    }
                                )
                            )
                        ),
                        Pattern::lazy(|| Self::pattern()).optional_self(),
                    ]),
                    0,
                    None
                ),
                Pattern::required(
                    Pattern::predicate(|token: &Token| {
                        token.kind == TokenKind::Punctuation(PunctuationKind::RightParenthesis)
                    }),
                    Action::failure(|span| {
                        ParseError::new(
                            ErrorKind::UnclosedDelimiter(Token::new(TokenKind::Punctuation(
                                PunctuationKind::LeftParenthesis,
                            ), Span::default())),
                            span,
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

    pub fn sequence() -> Pattern<Token, Element, ParseError> {
        Pattern::transform(
            Pattern::sequence([
                Pattern::ignore(Pattern::predicate(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::LeftParenthesis)
                })),
                Pattern::lazy(|| Self::pattern()).optional_self(),
                Pattern::repeat(
                    Pattern::sequence([
                        Pattern::action(
                            Pattern::predicate(|token: &Token| {
                                token.kind == TokenKind::Punctuation(PunctuationKind::Semicolon)
                            }),
                            Action::trigger(
                                Action::Ignore,
                                Action::failure(
                                    |span| {
                                        ParseError::new(
                                            ErrorKind::MissingSeparator(TokenKind::Punctuation(PunctuationKind::Semicolon)),
                                            span,
                                        )
                                    }
                                )
                            )
                        ),
                        Pattern::lazy(|| Self::pattern()).optional_self(),
                    ]),
                    0,
                    None
                ),
                Pattern::required(
                    Pattern::predicate(|token: &Token| {
                        token.kind == TokenKind::Punctuation(PunctuationKind::RightParenthesis)
                    }),
                    Action::failure(|span| {
                        ParseError::new(
                            ErrorKind::UnclosedDelimiter(Token::new(TokenKind::Punctuation(
                                PunctuationKind::LeftParenthesis,
                            ), Span::default())),
                            span,
                        )
                    }),
                ),
            ]),
            move |_, form| {
                let elements = form.outputs();

                Ok(Element::new(ElementKind::Sequence(elements), form.span))
            },
        )
    }

    pub fn collection() -> Pattern<Token, Element, ParseError> {
        Pattern::transform(
            Pattern::sequence([
                Pattern::ignore(Pattern::predicate(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::LeftBracket)
                })),
                Pattern::lazy(|| Self::pattern()).optional_self(),
                Pattern::repeat(
                    Pattern::sequence([
                        Pattern::action(
                            Pattern::predicate(|token: &Token| {
                                token.kind == TokenKind::Punctuation(PunctuationKind::Comma)
                            }),
                            Action::trigger(
                                Action::Ignore,
                                Action::failure(
                                    |span| {
                                        ParseError::new(
                                            ErrorKind::MissingSeparator(TokenKind::Punctuation(PunctuationKind::Comma)),
                                            span,
                                        )
                                    }
                                )
                            )
                        ),
                        Pattern::lazy(|| Self::pattern()).optional_self(),
                    ]),
                    0,
                    None
                ),
                Pattern::required(
                    Pattern::predicate(|token: &Token| {
                        token.kind == TokenKind::Punctuation(PunctuationKind::RightBracket)
                    }),
                    Action::failure(|span| {
                        ParseError::new(
                            ErrorKind::UnclosedDelimiter(Token::new(TokenKind::Punctuation(
                                PunctuationKind::LeftBracket,
                            ), Span::default())),
                            span,
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

    pub fn series() -> Pattern<Token, Element, ParseError> {
        Pattern::transform(
            Pattern::sequence([
                Pattern::ignore(Pattern::predicate(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::LeftBracket)
                })),
                Pattern::lazy(|| Self::pattern()).optional_self(),
                Pattern::repeat(
                    Pattern::sequence([
                        Pattern::action(
                            Pattern::predicate(|token: &Token| {
                                token.kind == TokenKind::Punctuation(PunctuationKind::Semicolon)
                            }),
                            Action::trigger(
                                Action::Ignore,
                                Action::failure(
                                    |span| {
                                        ParseError::new(
                                            ErrorKind::MissingSeparator(TokenKind::Punctuation(PunctuationKind::Semicolon)),
                                            span,
                                        )
                                    }
                                )
                            )
                        ),
                        Pattern::lazy(|| Self::pattern()).optional_self(),
                    ]),
                    0,
                    None
                ),
                Pattern::required(
                    Pattern::predicate(|token: &Token| {
                        token.kind == TokenKind::Punctuation(PunctuationKind::RightBracket)
                    }),
                    Action::failure(|span| {
                        ParseError::new(
                            ErrorKind::UnclosedDelimiter(Token::new(TokenKind::Punctuation(
                                PunctuationKind::LeftBracket,
                            ), Span::default())),
                            span,
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

    pub fn delimited() -> Pattern<Token, Element, ParseError> {
        Pattern::alternative([
            Self::bundle(),
            Self::scope(),
            Self::group(),
            Self::sequence(),
            Self::collection(),
            Self::series(),
        ])
    }
}
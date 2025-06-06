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
            former::{Form},
            pattern::Pattern,
            action::Action,
        },

        axo_span::Span,
    }
};

impl Parser {
    pub fn scope() -> Pattern<Token, Element, ParseError> {
        Pattern::transform(
            Pattern::guard(
                Arc::new(|former: &dyn Peekable<Token>| {
                    let mut lookahead = 0;
                    let mut brace_count = 0;
                    let mut found_semicolon = false;

                    while let Some(token) = former.peek_ahead(lookahead) {
                        match &token.kind {
                            TokenKind::Punctuation(PunctuationKind::LeftBrace) => {
                                brace_count += 1;
                            }
                            TokenKind::Punctuation(PunctuationKind::RightBrace) => {
                                brace_count -= 1;
                                if brace_count == 0 {
                                    break;
                                }
                            }
                            TokenKind::Punctuation(PunctuationKind::Semicolon) if brace_count == 1 => {
                                found_semicolon = true;
                            }
                            _ => {}
                        }
                        lookahead += 1;
                    }

                    found_semicolon
                }),
                Pattern::sequence([
                    Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                        token.kind == TokenKind::Punctuation(PunctuationKind::LeftBrace)
                    }))),
                    Pattern::lazy(|| Self::pattern()).optional_self(),
                    Pattern::repeat(
                        Pattern::sequence([
                            Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                                token.kind == TokenKind::Punctuation(PunctuationKind::Semicolon)
                            }))),
                            Pattern::lazy(|| Self::pattern()).optional_self(),
                        ]),
                        0,
                        None
                    ),
                    Pattern::required(
                        Pattern::predicate(Arc::new(|token: &Token| {
                            token.kind == TokenKind::Punctuation(PunctuationKind::RightBrace)
                        })),
                        Action::Error(Arc::new(|span| {
                            ParseError::new(
                                ErrorKind::UnclosedDelimiter(Token::new(TokenKind::Punctuation(
                                    PunctuationKind::LeftBrace,
                                ), Span::default())),
                                span,
                            )
                        })),
                    ),
                ])
            ),
            Arc::new(|_, form| {
                let elements = form.outputs();

                Ok(Element::new(ElementKind::Scope(elements), form.span))
            }),
        )
    }

    pub fn bundle() -> Pattern<Token, Element, ParseError> {
        Pattern::transform(
            Pattern::guard(
                Arc::new(|former: &dyn Peekable<Token>| {
                    let mut lookahead = 0;
                    let mut brace_count = 0;
                    let mut found_comma = false;

                    while let Some(token) = former.peek_ahead(lookahead) {
                        match &token.kind {
                            TokenKind::Punctuation(PunctuationKind::LeftBrace) => {
                                brace_count += 1;
                            }
                            TokenKind::Punctuation(PunctuationKind::RightBrace) => {
                                brace_count -= 1;
                                if brace_count == 0 {
                                    break;
                                }
                            }
                            TokenKind::Punctuation(PunctuationKind::Comma) if brace_count == 1 => {
                                found_comma = true;
                            }
                            _ => {}
                        }
                        lookahead += 1;
                    }

                    found_comma
                }),
                Pattern::sequence([
                    Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                        token.kind == TokenKind::Punctuation(PunctuationKind::LeftBrace)
                    }))),
                    Pattern::lazy(|| Self::pattern()).optional_self(),
                    Pattern::repeat(
                        Pattern::sequence([
                            Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                                token.kind == TokenKind::Punctuation(PunctuationKind::Comma)
                            }))),
                            Pattern::lazy(|| Self::pattern()).optional_self(),
                        ]),
                        0,
                        None
                    ),
                    Pattern::required(
                        Pattern::predicate(Arc::new(|token: &Token| {
                            token.kind == TokenKind::Punctuation(PunctuationKind::RightBrace)
                        })),
                        Action::Error(Arc::new(|span| {
                            ParseError::new(
                                ErrorKind::UnclosedDelimiter(Token::new(TokenKind::Punctuation(
                                    PunctuationKind::LeftBrace,
                                ), Span::default())),
                                span,
                            )
                        })),
                    ),
                ])
            ),
            Arc::new(|_, form| {
                let elements = form.outputs();

                Ok(Element::new(ElementKind::Bundle(elements), form.span))
            }),
        )
    }

    pub fn group() -> Pattern<Token, Element, ParseError> {
        Pattern::transform(
            Pattern::guard(
                Arc::new(|former: &dyn Peekable<Token>| {
                    let mut lookahead = 0;
                    let mut paren_count = 0;
                    let mut found_comma = false;

                    while let Some(token) = former.peek_ahead(lookahead) {
                        match &token.kind {
                            TokenKind::Punctuation(PunctuationKind::LeftParenthesis) => {
                                paren_count += 1;
                            }
                            TokenKind::Punctuation(PunctuationKind::RightParenthesis) => {
                                paren_count -= 1;
                                if paren_count == 0 {
                                    break;
                                }
                            }
                            TokenKind::Punctuation(PunctuationKind::Comma) if paren_count == 1 => {
                                found_comma = true;
                            }
                            _ => {}
                        }
                        lookahead += 1;
                    }

                    found_comma
                }),
                Pattern::sequence([
                    Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                        token.kind == TokenKind::Punctuation(PunctuationKind::LeftParenthesis)
                    }))),
                    Pattern::lazy(|| Self::pattern()).optional_self(),
                    Pattern::repeat(
                        Pattern::sequence([
                            Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                                token.kind == TokenKind::Punctuation(PunctuationKind::Comma)
                            }))),
                            Pattern::lazy(|| Self::pattern()).optional_self(),
                        ]),
                        0,
                        None
                    ),
                    Pattern::required(
                        Pattern::predicate(Arc::new(|token: &Token| {
                            token.kind == TokenKind::Punctuation(PunctuationKind::RightParenthesis)
                        })),
                        Action::Error(Arc::new(|span| {
                            ParseError::new(
                                ErrorKind::UnclosedDelimiter(Token::new(TokenKind::Punctuation(
                                    PunctuationKind::LeftParenthesis,
                                ), Span::default())),
                                span,
                            )
                        })),
                    ),
                ])
            ),
            Arc::new(|_, form| {
                let elements = form.outputs();

                Ok(Element::new(ElementKind::Group(elements), form.span))
            }),
        )
    }

    pub fn sequence() -> Pattern<Token, Element, ParseError> {
        Pattern::transform(
            Pattern::guard(
                Arc::new(|former: &dyn Peekable<Token>| {
                    let mut lookahead = 0;
                    let mut paren_count = 0;
                    let mut found_semicolon = false;

                    while let Some(token) = former.peek_ahead(lookahead) {
                        match &token.kind {
                            TokenKind::Punctuation(PunctuationKind::LeftParenthesis) => {
                                paren_count += 1;
                            }
                            TokenKind::Punctuation(PunctuationKind::RightParenthesis) => {
                                paren_count -= 1;
                                if paren_count == 0 {
                                    break;
                                }
                            }
                            TokenKind::Punctuation(PunctuationKind::Semicolon) if paren_count == 1 => {
                                found_semicolon = true;
                            }
                            _ => {}
                        }
                        lookahead += 1;
                    }

                    found_semicolon
                }),
                Pattern::sequence([
                    Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                        token.kind == TokenKind::Punctuation(PunctuationKind::LeftParenthesis)
                    }))),
                    Pattern::lazy(|| Self::pattern()).optional_self(),
                    Pattern::repeat(
                        Pattern::sequence([
                            Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                                token.kind == TokenKind::Punctuation(PunctuationKind::Semicolon)
                            }))),
                            Pattern::lazy(|| Self::pattern()).optional_self(),
                        ]),
                        0,
                        None
                    ),
                    Pattern::required(
                        Pattern::predicate(Arc::new(|token: &Token| {
                            token.kind == TokenKind::Punctuation(PunctuationKind::RightParenthesis)
                        })),
                        Action::Error(Arc::new(|span| {
                            ParseError::new(
                                ErrorKind::UnclosedDelimiter(Token::new(TokenKind::Punctuation(
                                    PunctuationKind::LeftParenthesis,
                                ), Span::default())),
                                span,
                            )
                        })),
                    ),
                ])
            ),
            Arc::new(|_, form| {
                let elements = form.outputs();

                Ok(Element::new(ElementKind::Sequence(elements), form.span))
            }),
        )
    }

    pub fn collection() -> Pattern<Token, Element, ParseError> {
        Pattern::transform(
            Pattern::guard(
                Arc::new(|former: &dyn Peekable<Token>| {
                    let mut lookahead = 0;
                    let mut bracket_count = 0;
                    let mut found_comma = false;

                    while let Some(token) = former.peek_ahead(lookahead) {
                        match &token.kind {
                            TokenKind::Punctuation(PunctuationKind::LeftBracket) => {
                                bracket_count += 1;
                            }
                            TokenKind::Punctuation(PunctuationKind::RightBracket) => {
                                bracket_count -= 1;
                                if bracket_count == 0 {
                                    break;
                                }
                            }
                            TokenKind::Punctuation(PunctuationKind::Comma) if bracket_count == 1 => {
                                found_comma = true;
                            }
                            _ => {}
                        }
                        lookahead += 1;
                    }

                    found_comma
                }),
                Pattern::sequence([
                    Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                        token.kind == TokenKind::Punctuation(PunctuationKind::LeftBracket)
                    }))),
                    Pattern::lazy(|| Self::pattern()).optional_self(),
                    Pattern::repeat(
                        Pattern::sequence([
                            Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                                token.kind == TokenKind::Punctuation(PunctuationKind::Comma)
                            }))),
                            Pattern::lazy(|| Self::pattern()).optional_self(),
                        ]),
                        0,
                        None
                    ),
                    Pattern::required(
                        Pattern::predicate(Arc::new(|token: &Token| {
                            token.kind == TokenKind::Punctuation(PunctuationKind::RightBracket)
                        })),
                        Action::Error(Arc::new(|span| {
                            ParseError::new(
                                ErrorKind::UnclosedDelimiter(Token::new(TokenKind::Punctuation(
                                    PunctuationKind::LeftBracket,
                                ), Span::default())),
                                span,
                            )
                        })),
                    ),
                ])
            ),
            Arc::new(|_, form| {
                let elements = form.outputs();

                Ok(Element::new(ElementKind::Collection(elements), form.span))
            }),
        )
    }

    pub fn series() -> Pattern<Token, Element, ParseError> {
        Pattern::transform(
            Pattern::guard(
                Arc::new(|former: &dyn Peekable<Token>| {
                    let mut lookahead = 0;
                    let mut bracket_count = 0;
                    let mut found_semicolon = false;

                    while let Some(token) = former.peek_ahead(lookahead) {
                        match &token.kind {
                            TokenKind::Punctuation(PunctuationKind::LeftBracket) => {
                                bracket_count += 1;
                            }
                            TokenKind::Punctuation(PunctuationKind::RightBracket) => {
                                bracket_count -= 1;
                                if bracket_count == 0 {
                                    break;
                                }
                            }
                            TokenKind::Punctuation(PunctuationKind::Semicolon) if bracket_count == 1 => {
                                found_semicolon = true;
                            }
                            _ => {}
                        }
                        lookahead += 1;
                    }

                    found_semicolon
                }),
                Pattern::sequence([
                    Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                        token.kind == TokenKind::Punctuation(PunctuationKind::LeftBracket)
                    }))),
                    Pattern::lazy(|| Self::pattern()).optional_self(),
                    Pattern::repeat(
                        Pattern::sequence([
                            Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                                token.kind == TokenKind::Punctuation(PunctuationKind::Semicolon)
                            }))),
                            Pattern::lazy(|| Self::pattern()).optional_self(),
                        ]),
                        0,
                        None
                    ),
                    Pattern::required(
                        Pattern::predicate(Arc::new(|token: &Token| {
                            token.kind == TokenKind::Punctuation(PunctuationKind::RightBracket)
                        })),
                        Action::Error(Arc::new(|span| {
                            ParseError::new(
                                ErrorKind::UnclosedDelimiter(Token::new(TokenKind::Punctuation(
                                    PunctuationKind::LeftBracket,
                                ), Span::default())),
                                span,
                            )
                        })),
                    ),
                ])
            ),
            Arc::new(|_, form| {
                let elements = form.outputs();

                Ok(Element::new(ElementKind::Series(elements), form.span))
            }),
        )
    }

    pub fn brace() -> Pattern<Token, Element, ParseError> {
        Pattern::transform(
            Pattern::sequence([
                Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::LeftBrace)
                }))),
                Pattern::lazy(|| Self::pattern()).optional_self(),
                Pattern::required(
                    Pattern::predicate(Arc::new(|token: &Token| {
                        token.kind == TokenKind::Punctuation(PunctuationKind::RightBrace)
                    })),
                    Action::Error(Arc::new(|span| {
                        ParseError::new(
                            ErrorKind::UnclosedDelimiter(Token::new(TokenKind::Punctuation(
                                PunctuationKind::LeftBrace,
                            ), Span::default())),
                            span,
                        )
                    })),
                ),
            ]),
            Arc::new(|_, form| {
                let elements = form.outputs();

                Ok(Element::new(ElementKind::Bundle(elements), form.span))
            }),
        )
    }

    pub fn parenthesis() -> Pattern<Token, Element, ParseError> {
        Pattern::transform(
            Pattern::sequence([
                Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::LeftParenthesis)
                }))),
                Pattern::lazy(|| Self::pattern()).optional_self(),
                Pattern::required(
                    Pattern::predicate(Arc::new(|token: &Token| {
                        token.kind == TokenKind::Punctuation(PunctuationKind::RightParenthesis)
                    })),
                    Action::Error(Arc::new(|span| {
                        ParseError::new(
                            ErrorKind::UnclosedDelimiter(Token::new(TokenKind::Punctuation(
                                PunctuationKind::LeftParenthesis,
                            ), Span::default())),
                            span,
                        )
                    })),
                ),
            ]),
            Arc::new(|_, form| {
                let elements = form.outputs();

                Ok(Element::new(ElementKind::Group(elements), form.span))
            }),
        )
    }

    pub fn bracket() -> Pattern<Token, Element, ParseError> {
        Pattern::transform(
            Pattern::sequence([
                Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::LeftBracket)
                }))),
                Pattern::lazy(|| Self::pattern()).optional_self(),
                Pattern::required(
                    Pattern::predicate(Arc::new(|token: &Token| {
                        token.kind == TokenKind::Punctuation(PunctuationKind::RightBracket)
                    })),
                    Action::Error(Arc::new(|span| {
                        ParseError::new(
                            ErrorKind::UnclosedDelimiter(Token::new(TokenKind::Punctuation(
                                PunctuationKind::LeftBracket,
                            ), Span::default())),
                            span,
                        )
                    })),
                ),
            ]),
            Arc::new(|_, form| {
                let elements = form.outputs();

                Ok(Element::new(ElementKind::Collection(elements), form.span))
            }),
        )
    }

    pub fn delimited() -> Pattern<Token, Element, ParseError> {
        Pattern::alternative([
            Self::scope(),
            Self::bundle(),
            Self::group(),
            Self::sequence(),
            Self::collection(),
            Self::series(),
            Self::parenthesis(),
            Self::brace(),
            Self::bracket(),
        ])
    }
}
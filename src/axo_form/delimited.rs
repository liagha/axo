use crate::thread::Arc;
use crate::axo_form::{Form, Pattern};
use crate::axo_parser::{Element, ElementKind, ParseError};
use crate::{PunctuationKind, Token, TokenKind};
use crate::axo_form::action::Action;
use crate::axo_form::parser::token;
use crate::axo_parser::error::ErrorKind;
use crate::axo_span::Span;

pub fn group() -> Pattern<Token, Element, ParseError> {
    Pattern::transform(
        Pattern::sequence([
            Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                token.kind == TokenKind::Punctuation(PunctuationKind::LeftParenthesis)
            }))),
            Pattern::optional(token()),
            Pattern::optional(Pattern::repeat(
                Pattern::sequence([
                    Pattern::required(
                        Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                            token.kind == TokenKind::Punctuation(PunctuationKind::Comma)
                        }))),
                        Action::Error(Arc::new(|span| {
                            ParseError::new(
                                ErrorKind::MissingSeparator(TokenKind::Punctuation(
                                    PunctuationKind::Comma,
                                )),
                                span,
                            )
                        })),
                    ),
                    token(),
                ]),
                0,
                None,
            )),
            Pattern::ignore(Pattern::required(
                Pattern::predicate(Arc::new(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::RightParenthesis)
                })),
                Action::Error(Arc::new(|span| {
                    ParseError::new(
                        ErrorKind::UnclosedDelimiter(Token::new(
                            TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                            span.clone(),
                        )),
                        span,
                    )
                })),
            )),
        ]),
        Arc::new(|forms, span: Span| {
            let elements = Form::expand_outputs(forms);

            Ok(Element::new(ElementKind::Group(elements), span))
        }),
    )
}

pub fn sequence() -> Pattern<Token, Element, ParseError> {
    Pattern::transform(
        Pattern::sequence([
            Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                token.kind == TokenKind::Punctuation(PunctuationKind::LeftParenthesis)
            }))),
            Pattern::optional(token()),
            Pattern::optional(Pattern::repeat(
                Pattern::sequence([
                    Pattern::required(
                        Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                            token.kind == TokenKind::Punctuation(PunctuationKind::SemiColon)
                        }))),
                        Action::Error(Arc::new(|span| {
                            ParseError::new(
                                ErrorKind::MissingSeparator(TokenKind::Punctuation(
                                    PunctuationKind::SemiColon,
                                )),
                                span,
                            )
                        })),
                    ),
                    token(),
                ]),
                0,
                None,
            )),
            Pattern::ignore(Pattern::required(
                Pattern::predicate(Arc::new(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::RightParenthesis)
                })),
                Action::Error(Arc::new(|span| {
                    ParseError::new(
                        ErrorKind::UnclosedDelimiter(Token::new(
                            TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                            span.clone(),
                        )),
                        span,
                    )
                })),
            )),
        ]),
        Arc::new(|forms, span: Span| {
            let elements = Form::expand_outputs(forms);
            Ok(Element::new(ElementKind::Sequence(elements), span))
        }),
    )
}

pub fn collection() -> Pattern<Token, Element, ParseError> {
    Pattern::transform(
        Pattern::sequence([
            Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                token.kind == TokenKind::Punctuation(PunctuationKind::LeftBracket)
            }))),
            Pattern::optional(token()),
            Pattern::optional(Pattern::repeat(
                Pattern::sequence([
                    Pattern::required(
                        Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                            token.kind == TokenKind::Punctuation(PunctuationKind::Comma)
                        }))),
                        Action::Error(Arc::new(|span| {
                            ParseError::new(
                                ErrorKind::MissingSeparator(TokenKind::Punctuation(
                                    PunctuationKind::Comma,
                                )),
                                span,
                            )
                        })),
                    ),
                    token(),
                ]),
                0,
                None,
            )),
            Pattern::ignore(Pattern::required(
                Pattern::predicate(Arc::new(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::RightBracket)
                })),
                Action::Error(Arc::new(|span| {
                    ParseError::new(
                        ErrorKind::UnclosedDelimiter(Token::new(
                            TokenKind::Punctuation(PunctuationKind::LeftBracket),
                            span.clone(),
                        )),
                        span,
                    )
                })),
            )),
        ]),
        Arc::new(|forms, span: Span| {
            let elements = Form::expand_outputs(forms);
            Ok(Element::new(ElementKind::Collection(elements), span))
        }),
    )
}

pub fn series() -> Pattern<Token, Element, ParseError> {
    Pattern::transform(
        Pattern::sequence([
            Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                token.kind == TokenKind::Punctuation(PunctuationKind::LeftBracket)
            }))),
            Pattern::optional(token()),
            Pattern::optional(Pattern::repeat(
                Pattern::sequence([
                    Pattern::required(
                        Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                            token.kind == TokenKind::Punctuation(PunctuationKind::SemiColon)
                        }))),
                        Action::Error(Arc::new(|span| {
                            ParseError::new(
                                ErrorKind::MissingSeparator(TokenKind::Punctuation(
                                    PunctuationKind::SemiColon,
                                )),
                                span,
                            )
                        })),
                    ),
                    token(),
                ]),
                0,
                None,
            )),
            Pattern::ignore(Pattern::required(
                Pattern::predicate(Arc::new(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::RightBracket)
                })),
                Action::Error(Arc::new(|span| {
                    ParseError::new(
                        ErrorKind::UnclosedDelimiter(Token::new(
                            TokenKind::Punctuation(PunctuationKind::LeftBracket),
                            span.clone(),
                        )),
                        span,
                    )
                })),
            )),
        ]),
        Arc::new(|forms, span: Span| {
            let elements = Form::expand_outputs(forms);
            Ok(Element::new(ElementKind::Series(elements), span))
        }),
    )
}

pub fn bundle() -> Pattern<Token, Element, ParseError> {
    Pattern::transform(
        Pattern::sequence([
            Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                token.kind == TokenKind::Punctuation(PunctuationKind::LeftBrace)
            }))),
            Pattern::optional(token()),
            Pattern::optional(Pattern::repeat(
                Pattern::sequence([
                    Pattern::required(
                        Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                            token.kind == TokenKind::Punctuation(PunctuationKind::Comma)
                        }))),
                        Action::Error(Arc::new(|span| {
                            ParseError::new(
                                ErrorKind::MissingSeparator(TokenKind::Punctuation(
                                    PunctuationKind::Comma,
                                )),
                                span,
                            )
                        })),
                    ),
                    token(),
                ]),
                0,
                None,
            )),
            Pattern::ignore(Pattern::required(
                Pattern::predicate(Arc::new(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::RightBrace)
                })),
                Action::Error(Arc::new(|span| {
                    ParseError::new(
                        ErrorKind::UnclosedDelimiter(Token::new(
                            TokenKind::Punctuation(PunctuationKind::LeftBrace),
                            span.clone(),
                        )),
                        span,
                    )
                })),
            )),
        ]),
        Arc::new(|forms, span: Span| {
            let elements = Form::expand_outputs(forms);
            Ok(Element::new(ElementKind::Bundle(elements), span))
        }),
    )
}

pub fn scope() -> Pattern<Token, Element, ParseError> {
    Pattern::transform(
        Pattern::sequence([
            Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                token.kind == TokenKind::Punctuation(PunctuationKind::LeftBrace)
            }))),
            Pattern::optional(token()),
            Pattern::optional(Pattern::repeat(
                Pattern::sequence([
                    Pattern::required(
                        Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                            token.kind == TokenKind::Punctuation(PunctuationKind::SemiColon)
                        }))),
                        Action::Error(Arc::new(|span| {
                            ParseError::new(
                                ErrorKind::MissingSeparator(TokenKind::Punctuation(
                                    PunctuationKind::SemiColon,
                                )),
                                span,
                            )
                        })),
                    ),
                    token(),
                ]),
                0,
                None,
            )),
            Pattern::ignore(Pattern::required(
                Pattern::predicate(Arc::new(|token: &Token| {
                    token.kind == TokenKind::Punctuation(PunctuationKind::RightBrace)
                })),
                Action::Error(Arc::new(|span| {
                    ParseError::new(
                        ErrorKind::UnclosedDelimiter(Token::new(
                            TokenKind::Punctuation(PunctuationKind::LeftBrace),
                            span.clone(),
                        )),
                        span,
                    )
                })),
            )),
        ]),
        Arc::new(|forms, span: Span| {
            let elements = Form::expand_outputs(forms);
            Ok(Element::new(ElementKind::Sequence(elements), span))
        }),
    )
}

pub fn delimited() -> Pattern<Token, Element, ParseError> {
    Pattern::alternative([
        group(),
        sequence(),
        collection(),
        series(),
        bundle(),
        scope(),
    ])
}

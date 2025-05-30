use crate::thread::Arc;
use crate::axo_form::{Form, Pattern};
use crate::axo_parser::{Element, ElementKind, ParseError};
use crate::{PunctuationKind, Token, TokenKind};
use crate::axo_form::action::Action;
use crate::axo_form::parser::{pattern};
use crate::axo_parser::error::ErrorKind;
use crate::axo_span::Span;

pub fn scope() -> Pattern<Token, Element, ParseError> {
    Pattern::transform(
        Pattern::guard(
            Arc::new(|former: &dyn crate::axo_data::peekable::Peekable<Token>| {
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
                        TokenKind::Punctuation(PunctuationKind::SemiColon) if brace_count == 1 => {
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
                Pattern::lazy(|| pattern()).optional_self(),
                Pattern::repeat(
                    Pattern::sequence([
                        Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                            token.kind == TokenKind::Punctuation(PunctuationKind::SemiColon)
                        }))),
                        Pattern::lazy(|| pattern()).optional_self(),
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
        Arc::new(|forms, span: Span| {
            let elements = Form::expand_outputs(forms);

            Ok(Element::new(ElementKind::Scope(elements), span))
        }),
    )
}

pub fn bundle() -> Pattern<Token, Element, ParseError> {
    Pattern::transform(
        Pattern::guard(
            Arc::new(|former: &dyn crate::axo_data::peekable::Peekable<Token>| {
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
                Pattern::lazy(|| pattern()).optional_self(),
                Pattern::repeat(
                    Pattern::sequence([
                        Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                            token.kind == TokenKind::Punctuation(PunctuationKind::Comma)
                        }))),
                        Pattern::lazy(|| pattern()).optional_self(),
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
        Arc::new(|forms, span: Span| {
            let elements = Form::expand_outputs(forms);

            Ok(Element::new(ElementKind::Bundle(elements), span))
        }),
    )
}

pub fn single() -> Pattern<Token, Element, ParseError> {
    Pattern::transform(
        Pattern::sequence([
            Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                token.kind == TokenKind::Punctuation(PunctuationKind::LeftBrace)
            }))),
            Pattern::lazy(|| pattern()).optional_self(),
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
        Arc::new(|forms, span: Span| {
            let elements = Form::expand_outputs(forms);

            Ok(Element::new(ElementKind::Bundle(elements), span))
        }),
    )
}

pub fn delimited() -> Pattern<Token, Element, ParseError> {
    Pattern::alternative([
        scope(),
        bundle(),
        single(),
    ])
}
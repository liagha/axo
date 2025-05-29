use crate::thread::Arc;
use crate::axo_form::{Form, Pattern};
use crate::axo_parser::{Element, ElementKind, ParseError};
use crate::{PunctuationKind, Token, TokenKind};
use crate::axo_form::action::Action;
use crate::axo_form::parser::{pattern, token};
use crate::axo_parser::error::ErrorKind;
use crate::axo_span::Span;

pub fn bundle() -> Pattern<Token, Element, ParseError> {
    Pattern::transform(
        Pattern::sequence([
            Pattern::ignore(Pattern::predicate(Arc::new(|token: &Token| {
                token.kind == TokenKind::Punctuation(PunctuationKind::LeftBrace)
            }))),
            Pattern::optional(Pattern::lazy(|| pattern())),
            Pattern::repeat(
                Pattern::sequence([
                    Pattern::conditional(
                        Pattern::predicate(Arc::new(|token: &Token| {
                            token.kind == TokenKind::Punctuation(PunctuationKind::Comma)
                        })),
                        Action::Ignore,
                        Action::Error(Arc::new(|span| {
                            ParseError::new(
                                ErrorKind::MissingSeparator(TokenKind::Punctuation(
                                    PunctuationKind::Comma,
                                )),
                                span,
                            )
                        })),
                    ),
                    Pattern::optional(Pattern::lazy(|| pattern())),
                ]),
                0,
                None,
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
            Pattern::optional(Pattern::lazy(|| pattern())),
            Pattern::repeat(
                Pattern::sequence([
                    Pattern::conditional(
                        Pattern::predicate(Arc::new(|token: &Token| {
                            token.kind == TokenKind::Punctuation(PunctuationKind::SemiColon)
                        })),
                        Action::Ignore,
                        Action::Error(Arc::new(|span| {
                            ParseError::new(
                                ErrorKind::MissingSeparator(TokenKind::Punctuation(
                                    PunctuationKind::Comma,
                                )),
                                span,
                            )
                        })),
                    ),
                    Pattern::optional(Pattern::lazy(|| pattern())),
                ]),
                0,
                None,
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
    ])
}
use crate::axo_form::form::Form;
use crate::axo_form::order::Order;
use crate::axo_form::pattern::Classifier;
use crate::axo_parser::{Element, ElementKind, ParseError, Parser};
use crate::axo_parser::error::ErrorKind;
use crate::axo_scanner::{Token, TokenKind};
use crate::axo_schema::{Conditional, Repeat};

impl Parser {
    pub fn conditional() -> Classifier<Token, Element, ParseError> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    if let TokenKind::Identifier(identifier) = &token.kind {
                        identifier == "if"
                    } else {
                        false
                    }
                })
                    .with_ignore(),
                Classifier::with_fallback(
                    Classifier::deferred(|| Self::element()),
                    Order::fail(|_, form| {
                        ParseError::new(ErrorKind::ExpectedCondition, form.span)
                    }),
                ),
                Classifier::with_fallback(
                    Classifier::deferred(|| Self::element()),
                    Order::fail(|_, form| ParseError::new(ErrorKind::ExpectedBody, form.span)),
                ),
                Classifier::optional(Classifier::sequence([
                    Classifier::predicate(|token: &Token| {
                        if let TokenKind::Identifier(identifier) = &token.kind {
                            identifier == "else"
                        } else {
                            false
                        }
                    })
                        .with_ignore(),
                    Classifier::deferred(|| Self::element()),
                ])),
            ]),
            |_, form| {
                let sequence = form.outputs();
                let condition = sequence[0].clone();
                let then = sequence[1].clone();

                if let Some(alternate) = sequence.get(2).cloned() {
                    let span = condition.span.mix(&alternate.span);
                    Ok(Form::output(
                        Element::new(
                            ElementKind::Conditional(Conditional::new(condition.into(), then.into(), Some(alternate.into()))),
                            span,
                        )
                    ))
                } else {
                    let span = condition.span.mix(&then.span);
                    Ok(Form::output(
                        Element::new(
                            ElementKind::Conditional(Conditional::new(condition.into(), then.into(), None)),
                            span,
                        )
                    ))
                }
            },
        )
    }

    pub fn cycle() -> Classifier<Token, Element, ParseError> {
        Classifier::choice([
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    if let TokenKind::Identifier(identifier) = &token.kind {
                        identifier == "while"
                    } else {
                        false
                    }
                }).with_ignore(),
                Classifier::deferred(Self::element).with_fallback(
                    Order::fail(|_, form| {
                        ParseError::new(ErrorKind::ExpectedCondition, form.span)
                    })
                ),
                Classifier::deferred(Self::element).with_fallback(
                    Order::fail(|_, form| {
                        ParseError::new(ErrorKind::ExpectedBody, form.span)
                    })
                )
            ]),
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    if let TokenKind::Identifier(identifier) = &token.kind {
                        identifier == "loop"
                    } else {
                        false
                    }
                }).with_ignore(),
                Classifier::deferred(Self::element).with_fallback(
                    Order::fail(|_, form| {
                        ParseError::new(ErrorKind::ExpectedBody, form.span)
                    })
                ),
            ]),
        ], vec![1, 0]).with_transform(
            |_, form| {
                let sequence = form.outputs();

                if sequence.len() == 1 {
                    let body = sequence[0].clone();
                    let span = body.span.clone();
                    Ok(Form::output(
                        Element::new(
                            ElementKind::Repeat(Repeat::new(None, body.into())),
                            span,
                        )
                    ))
                } else if sequence.len() == 2 {
                    let condition = sequence[0].clone();
                    let body = sequence[1].clone();
                    let span = condition.span.mix(&body.span);
                    Ok(Form::output(
                        Element::new(
                            ElementKind::Repeat(Repeat::new(Some(condition.into()), body.into())),
                            span,
                        )
                    ))
                } else {
                    unreachable!()
                }
            }
        )
    }

    pub fn statement() -> Classifier<Token, Element, ParseError> {
        Classifier::alternative([Self::conditional(), Self::cycle(), Self::binding()])
    }
}
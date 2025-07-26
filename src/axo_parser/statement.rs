use {
    super::{
        Element, ElementKind, ParseError, Parser,
        error::ErrorKind,
    },
    crate::{
        axo_cursor::{Span, Spanned},
        axo_form::{
            form::Form,
            order::Order,
            pattern::Classifier,
        },
        axo_scanner::{Token, TokenKind},
        axo_schema::{Conditional, Repeat},
    }
};

impl<'parser> Parser<'parser> {
    pub fn conditional() -> Classifier<Token, Element, ParseError> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    if let TokenKind::Identifier(identifier) = &token.kind {
                        identifier == "if"
                    } else {
                        false
                    }
                }),
                Classifier::with_fallback(
                    Classifier::deferred(Self::element),
                    Order::fail(|_, form: Form<Token, Element, ParseError>| {
                        let span = form.unwrap_input().span();

                        ParseError::new(ErrorKind::ExpectedCondition, span)
                    }),
                ),
                Classifier::with_fallback(
                    Classifier::deferred(Self::element),
                    Order::fail(|_, form: Form<Token, Element, ParseError>| {
                        let span = form.unwrap_input().span();

                        ParseError::new(ErrorKind::ExpectedBody, span)
                    }),
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
                    Classifier::deferred(Self::element),
                ])),
            ]),
            |_, form| {
                let sequence = form.as_forms();
                let keyword = sequence[0].unwrap_input();
                let condition = sequence[1].unwrap_output().clone();
                let then = sequence[2].unwrap_output().clone();

                if let Some(alternate) = sequence.get(3).cloned() {
                    let alternate = alternate.unwrap_output().clone();
                    let span = Span::merge(&keyword.span(), &alternate.span());

                    Ok(Form::output(
                        Element::new(
                            ElementKind::Conditional(Conditional::new(Box::new(condition), Box::new(then), Some(alternate.into()))),
                            span,
                        )
                    ))
                } else {
                    let span = condition.span().merge(&then.span());

                    Ok(Form::output(
                        Element::new(
                            ElementKind::Conditional(Conditional::new(Box::new(condition), Box::new(then), None)),
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
                    Order::fail(|_, form: Form<Token, Element, ParseError>| {
                        let span = form.unwrap_input().span();

                        ParseError::new(ErrorKind::ExpectedCondition, span)
                    })
                ),
                Classifier::deferred(Self::element).with_fallback(
                    Order::fail(|_, form: Form<Token, Element, ParseError>| {
                        let span = form.unwrap_input().span();

                        ParseError::new(ErrorKind::ExpectedBody, span)
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
                    Order::fail(|_, form: Form<Token, Element, ParseError>| {
                        let span = form.unwrap_input().span();

                        ParseError::new(ErrorKind::ExpectedBody, span)
                    })
                ),
            ]),
        ], vec![1, 0]).with_transform(
            |_, form| {
                let sequence = form.as_forms();
                let keyword = sequence[0].unwrap_input();

                if sequence.len() == 1 {
                    let body = sequence[0].unwrap_output().clone();
                    let span = Span::merge(&keyword.span(), &body.span());

                    Ok(Form::output(
                        Element::new(
                            ElementKind::Repeat(Repeat::new(None, body.into())),
                            span,
                        )
                    ))
                } else if sequence.len() == 2 {
                    let condition = sequence[0].unwrap_output().clone();
                    let body = sequence[1].unwrap_output().clone();
                    let span = Span::merge(&keyword.span, &body.span);

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
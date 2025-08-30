use {
    super::{
        Element, ElementKind, ParseError, Parser,
        ErrorKind,
    },
    crate::{
        tracker::{Span, Spanned},
        formation::{
            form::Form,
            classifier::Classifier,
        },
        scanner::{Token, TokenKind},
        schema::{Conditional, While},
    }
};

impl<'parser> Parser<'parser> {
    pub fn conditional() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
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
                    Classifier::fail(|form: Form<Token, Element, ParseError>| {
                        let span = form.unwrap_input().borrow_span();

                        ParseError::new(ErrorKind::ExpectedCondition, span)
                    }),
                ),
                Classifier::with_fallback(
                    Classifier::deferred(Self::element),
                    Classifier::fail(|form: Form<Token, Element, ParseError>| {
                        let span = form.unwrap_input().borrow_span();

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
            |form: Form<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>| {
                let sequence = form.as_forms();
                let keyword = sequence[0].unwrap_input();
                let condition = sequence[1].unwrap_output().clone();
                let then = sequence[2].unwrap_output().clone();

                if let Some(alternate) = sequence.get(3).cloned() {
                    let alternate = alternate.unwrap_output().clone();
                    let span = Span::merge(&keyword.borrow_span(), &alternate.borrow_span());

                    Ok(Form::output(
                        Element::new(
                            ElementKind::Conditional(Conditional::new(Box::new(condition), Box::new(then), Some(Box::new(alternate)))),
                            span,
                        )
                    ))
                } else {
                    let span = condition.borrow_span().merge(&then.borrow_span());

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

    pub fn cycle() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
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
                    Classifier::fail(|form: Form<Token, Element, ParseError>| {
                        let span = form.unwrap_input().borrow_span();

                        ParseError::new(ErrorKind::ExpectedCondition, span)
                    })
                ),
                Classifier::deferred(Self::element).with_fallback(
                    Classifier::fail(|form: Form<Token, Element, ParseError>| {
                        let span = form.unwrap_input().borrow_span();

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
                    Classifier::fail(|form: Form<Token, Element, ParseError>| {
                        let span = form.unwrap_input().borrow_span();

                        ParseError::new(ErrorKind::ExpectedBody, span)
                    })
                ),
            ]),
        ], vec![1, 0]).with_transform(
            |form: Form<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>>| {
                let sequence: &[Form<Token<'_>, Element, ParseError>] = form.as_forms();
                let keyword = sequence[0].unwrap_input();

                if sequence.len() == 1 {
                    let body = sequence[0].unwrap_output().clone();
                    let span = Span::merge(&keyword.borrow_span(), &body.borrow_span());

                    Ok(Form::output(
                        Element::new(
                            ElementKind::While(While::new(None, Box::new(body))),
                            span,
                        )
                    ))
                } else if sequence.len() == 2 {
                    let condition = sequence[0].unwrap_output().clone();
                    let body = sequence[1].unwrap_output().clone();
                    let span = Span::merge(&keyword.span, &body.span);

                    Ok(Form::output(
                        Element::new(
                            ElementKind::While(While::new(Some(Box::new(condition)), Box::new(body))),
                            span,
                        )
                    ))
                } else {
                    unreachable!()
                }
            }
        )
    }

    pub fn statement() -> Classifier<'parser, Token<'parser>, Element<'parser>, ParseError<'parser>> {
        Classifier::alternative([Self::conditional(), Self::cycle(), Self::binding()])
    }
}
use {
    super::{
        error::ErrorKind, 
        Element, ElementKind,
        SymbolKind,
        ParseError, Parser
    },
    crate::{
        axo_cursor::{
            Span,
        },
        axo_form::{
            form::Form,
            former::Former,
            order::Order,
            pattern::Classifier,
        },
        axo_scanner::{OperatorKind, PunctuationKind, Token, TokenKind},
        axo_schema::{
            Access, Assign,
            Binary, Binding, Conditional,
            Construct, Enumeration,
            Index, Invoke,
            Label,
            Repeat, Structure, Unary,
        },
    },
};

impl Parser {
    pub fn identifier() -> Classifier<Token, Element, ParseError> {
        Classifier::with_transform(
            Classifier::predicate(|token: &Token| matches!(token.kind, TokenKind::Identifier(_))),
            |_, form| {
                let input = form.inputs()[0].clone();
                let identifier = input.kind.unwrap_identifier();

                Ok(Form::output(
                    Element::new(ElementKind::identifier(identifier), input.span)
                ))
            },
        )
    }

    pub fn literal() -> Classifier<Token, Element, ParseError> {
        Classifier::with_transform(
            Classifier::predicate(|token: &Token| {
                matches!(
                    token.kind,
                    TokenKind::String(_)
                        | TokenKind::Character(_)
                        | TokenKind::Boolean(_)
                        | TokenKind::Float(_)
                        | TokenKind::Integer(_)
                )
            }),
            |_, form| {
                let input = form.inputs()[0].clone();

                Ok(Form::output(
                    Element::new(ElementKind::literal(input.kind), input.span)
                ))
            },
        )
    }

    pub fn token() -> Classifier<Token, Element, ParseError> {
        Classifier::alternative([Self::identifier(), Self::literal()])
    }

    pub fn whitespace() -> Classifier<Token, Element, ParseError> {
        Classifier::alternative([Classifier::predicate(
            |token: &Token| {
                matches!(
                    token.kind,
                    TokenKind::Comment(_)
                        | TokenKind::Punctuation(PunctuationKind::Newline)
                        | TokenKind::Punctuation(PunctuationKind::Tab)
                        | TokenKind::Punctuation(PunctuationKind::Indentation(_))
                        | TokenKind::Punctuation(PunctuationKind::Space)
                )
            },
        )])
    }

    pub fn primary() -> Classifier<Token, Element, ParseError> {
        Classifier::alternative([Self::delimited(), Self::token()])
    }

    pub fn prefixed() -> Classifier<Token, Element, ParseError> {
        Classifier::with_order(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    if let TokenKind::Operator(operator) = &token.kind {
                        operator.is_prefix()
                    } else {
                        false
                    }
                }),
                Self::primary(),
            ]),
            Order::convert(|_, form: Form<Token, Element, ParseError>| {
                let prefixes = form.inputs();
                let operand = form.outputs()[0].clone();
                let mut unary = operand.clone();

                for prefix in prefixes {
                    let span = Span::mix(&prefix.span, &unary.span);

                    unary = Element::new(
                        ElementKind::unary(Unary::new(
                            prefix,
                            unary.into(),
                        )),
                        span,
                    );
                }

                Ok(Form::output(unary))
            })
        )
    }

    pub fn suffixed() -> Classifier<Token, Element, ParseError> {
        Classifier::with_transform(
            Classifier::sequence([
                Self::primary(),
                Classifier::repetition(
                    Classifier::alternative([
                        Self::group(Classifier::lazy(Self::element)),
                        Self::collection(Classifier::lazy(Self::element)),
                        Self::bundle(Classifier::lazy(Self::element)),
                        Classifier::predicate(|token: &Token| {
                            if let TokenKind::Operator(operator) = &token.kind {
                                operator.is_suffix()
                            } else {
                                false
                            }
                        })
                    ]),
                    0,
                    None
                ),
            ]),
            |_, form| {
                let sequence = form.unwrap().clone();
                let operand = sequence[0].unwrap_output();
                let suffixes = sequence[1].unwrap();
                let mut unary = operand.clone();

                for suffix in suffixes {
                    let span = Span::mix(&unary.span, &suffix.span);

                    if let Some(token) = suffix.get_input() {
                        unary = Element::new(
                            ElementKind::Unary(Unary::new(token, unary.into())),
                            span,
                        );
                    } else if let Some(element) = suffix.get_output() {
                        match element.kind {
                            ElementKind::Group(group) => {
                                unary = Element::new(
                                    ElementKind::invoke(Invoke::new(unary.into(), group.items)),
                                    span,
                                )
                            }
                            ElementKind::Collection(collection) => {
                                unary = Element::new(
                                    ElementKind::index(Index::new(unary.into(), collection.items)),
                                    span,
                                )
                            }
                            ElementKind::Bundle(bundle) => {
                                unary = Element::new(
                                    ElementKind::construct(Construct::new(unary.into(), bundle.items)),
                                    span,
                                )
                            }
                            _ => {}
                        }
                    }
                }

                Ok(Form::output(unary))
            },
        )
    }


    pub fn unary() -> Classifier<Token, Element, ParseError> {
        Classifier::alternative([
            Self::prefixed(),
            Self::suffixed(),
            Self::primary(),
        ])
    }

    pub fn access() -> Classifier<Token, Element, ParseError> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::lazy(|| Self::primary()),
                Classifier::predicate(|token: &Token| {
                    matches!(token.kind, TokenKind::Operator(ref op) if op == &OperatorKind::Dot)
                }).with_ignore(),
                Classifier::lazy(|| Self::primary()),
            ]),
            |_, form| {
                let sequence = form.unwrap();
                let object = sequence[0].unwrap_output();
                let target = sequence[1].unwrap_output();
                let span = Span::mix(&object.span, &target.span);

                Ok(Form::output(
                    Element::new(
                        ElementKind::access(Access::new(object.into(), target.into())),
                        span,
                    )
                ))
            },
        )
    }

    pub fn label() -> Classifier<Token, Element, ParseError> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::lazy(|| Self::primary()),
                Classifier::predicate(|token: &Token| {
                    matches!(token.kind, TokenKind::Operator(ref op) if op.as_slice() == [OperatorKind::Colon])
                }).with_ignore(),
                Classifier::lazy(|| Self::element()),
            ]),
            |_, form| {
                let sequence = form.unwrap();
                let label = sequence[0].unwrap_output();
                let element = sequence[1].unwrap_output();
                let span = Span::mix(&label.span, &element.span);

                Ok(Form::output(
                    Element::new(
                        ElementKind::label(Label::new(label.into(), element.into())),
                        span,
                    )
                ))
            },
        )
    }

    pub fn assign() -> Classifier<Token, Element, ParseError> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::lazy(|| Self::unary()),
                Classifier::predicate(|token: &Token| {
                    matches!(token.kind, TokenKind::Operator(ref op) if op.as_slice() == [OperatorKind::Equal])
                }).with_ignore(),
                Classifier::lazy(|| Self::element()),
            ]),
            |_, form| {
                let sequence = form.unwrap();
                let target = sequence[0].unwrap_output();
                let value = sequence[1].unwrap_output();
                let span = Span::mix(&target.span, &value.span);

                Ok(Form::output(
                    Element::new(
                        ElementKind::Assign(Assign::new(target.into(), value.into())),
                        span,
                    )
                ))
            },
        )
    }


    pub fn compound() -> Classifier<Token, Element, ParseError> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::lazy(|| Self::unary()),
                Classifier::predicate(|token: &Token| {
                    if let TokenKind::Operator(op) = &token.kind {
                        if let OperatorKind::Composite(compound) = op {
                            OperatorKind::Composite(compound.clone()).decompound().is_some()
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }),
                Classifier::lazy(|| Self::element()),
            ]),
            |_, form| {
                let sequence = form.unwrap();
                let target = sequence[0].unwrap_output();
                let raw = sequence[1].unwrap_input();
                let operator = raw.kind.unwrap_operator();
                let value = sequence[2].unwrap_output();
                let span = Span::mix(&target.span, &value.span);

                if let Some(base) = operator.decompound() {
                    let operation = Token {
                        kind: TokenKind::Operator(base),
                        span: raw.span.clone(),
                    };

                    let right = Element::new(
                        ElementKind::Binary(Binary::new(
                            target.clone().into(),
                            operation,
                            value.into())
                        ),
                        span.clone(),
                    );

                    return Ok(Form::output(
                        Element::new(
                            ElementKind::Assign(Assign::new(target.into(), right.into())),
                            span,
                        )
                    ));
                }

                unreachable!()
            },
        )
    }

    pub fn binary() -> Classifier<Token, Element, ParseError> {
        Classifier::alternative([
            Self::access(),
            Self::assign(),
            Self::label(),
            Self::binding(),
            Self::compound(),
            Classifier::with_transform(
                Classifier::sequence([
                    Classifier::alternative([
                        Self::statement(),
                        Self::unary(),
                    ]),
                    Classifier::repetition(
                        Classifier::sequence([
                            Classifier::predicate(move |token: &Token| {
                                if let TokenKind::Operator(operator) = &token.kind {
                                    if let Some(_) = operator.precedence() {
                                        true
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                }
                            }),
                            Classifier::alternative([
                                Self::statement(),
                                Self::unary(),
                            ])
                        ]),
                        1,
                        None,
                    ),
                ]),
                move |_, form| {
                    let sequence = form.unwrap();
                    let mut left = sequence[0].unwrap_output();
                    let operations = sequence[1].unwrap();
                    let mut pairs = Vec::new();

                    for operation in operations {
                        let sequence = operation.unwrap();
                        if sequence.len() >= 2 {
                            let operator = sequence[0].unwrap_input();
                            let operand = sequence[1].unwrap_output();
                            let precedence = if let TokenKind::Operator(op) = &operator.kind {
                                op.precedence().unwrap_or(0)
                            } else {
                                0
                            };
                            pairs.push((operator, operand, precedence));
                        }
                    }

                    left = Self::climb(left, pairs, 0);

                    Ok(Form::output(left))
                },
            )
        ])
    }

    fn climb(mut left: Element, pairs: Vec<(Token, Element, u8)>, threshold: u8) -> Element {
        let mut current = 0;

        while current < pairs.len() {
            let (operator, operand, precedence) = &pairs[current];

            if *precedence < threshold {
                break;
            }

            let mut right = operand.clone();
            let mut lookahead = current + 1;

            while lookahead < pairs.len() {
                let (_, _, priority) = &pairs[lookahead];

                if *priority > *precedence {
                    let mut higher = Vec::new();
                    while lookahead < pairs.len() && pairs[lookahead].2 > *precedence {
                        higher.push(pairs[lookahead].clone());
                        lookahead += 1;
                    }

                    right = Self::climb(right, higher, *precedence + 1);
                    break;
                } else {
                    break;
                }
            }

            let start = left.span.start.clone();
            let end = right.span.end.clone();
            let span = Span::new(start, end);

            left = Element::new(
                ElementKind::Binary(Binary::new(left.into(), operator.clone(), right.into())),
                span,
            );

            current = lookahead;
        }

        left
    }

    pub fn expression() -> Classifier<Token, Element, ParseError> {
        Classifier::alternative([Self::binary(), Self::unary(), Self::primary()])
    }

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
                    Classifier::lazy(|| Self::element()),
                    Order::fail(|_, form| {
                        ParseError::new(ErrorKind::ExpectedCondition, form.span)
                    }),
                ),
                Classifier::with_fallback(
                    Classifier::lazy(|| Self::element()),
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
                    Classifier::lazy(|| Self::element()),
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
        Classifier::with_transform(
            Classifier::alternative([
                Classifier::sequence([
                    Classifier::predicate(|token: &Token| {
                        if let TokenKind::Identifier(identifier) = &token.kind {
                            identifier == "loop"
                        } else {
                            false
                        }
                    })
                        .with_ignore(),
                    Classifier::with_fallback(
                        Classifier::lazy(|| Self::element()),
                        Order::fail(|_, form| {
                            ParseError::new(ErrorKind::ExpectedBody, form.span)
                        }),
                    ),
                ]),
                Classifier::sequence([
                    Classifier::predicate(|token: &Token| {
                        if let TokenKind::Identifier(identifier) = &token.kind {
                            identifier == "while"
                        } else {
                            false
                        }
                    })
                        .with_ignore(),
                    Classifier::with_fallback(
                        Classifier::lazy(|| Self::element()),
                        Order::fail(|_, form| {
                            ParseError::new(ErrorKind::ExpectedCondition, form.span)
                        }),
                    ),
                    Classifier::with_fallback(
                        Classifier::lazy(|| Self::element()),
                        Order::fail(|_, form| {
                            ParseError::new(ErrorKind::ExpectedBody, form.span)
                        }),
                    ),
                ]),
            ]),
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
            },
        )
    }



    pub fn statement() -> Classifier<Token, Element, ParseError> {
        Classifier::alternative([Self::conditional(), Self::cycle(), Self::binding()])
    }

    pub fn element() -> Classifier<Token, Element, ParseError> {
        Classifier::alternative([
            Self::statement(),
            Self::expression()
        ])
    }

    pub fn fallback() -> Classifier<Token, Element, ParseError> {
        Classifier::with_order(
            Classifier::predicate(|_token| true),
            Order::fail(
                |_, form: Form<Token, Element, ParseError>| {
                    ParseError::new(
                        ErrorKind::UnexpectedToken(form.unwrap_input().kind),
                        form.span,
                    )
                },
            ),
        )
    }

    pub fn parser() -> Classifier<Token, Element, ParseError> {
        Classifier::repetition(
            Classifier::alternative([
                Self::symbolization(),
                Self::element(),
                Self::fallback()
            ]),
            0,
            None,
        )
    }
}
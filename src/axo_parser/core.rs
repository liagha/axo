use {
    super::{error::ErrorKind, Element, ElementKind, ParseError, Parser},
    crate::{
        axo_cursor::Span,
        axo_form::{
            order::Order,
            form::{Form, FormKind},
            former::Former,
            pattern::Classifier,
        },
        axo_parser::{Symbol, SymbolKind},
        axo_scanner::{PunctuationKind, Token, TokenKind},
        artifact::Artifact,
        axo_cursor::Spanned,
        thread::Arc,
    },
    log::trace,
};


impl Parser {
    // Basic Token Patterns

    pub fn identifier() -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
            Classifier::predicate(|token: &Token| matches!(token.kind, TokenKind::Identifier(_))),
            |_, form| {
                let input = form.inputs()[0].clone();

                if let Token {
                    kind: TokenKind::Identifier(identifier),
                    span,
                } = input
                {
                    Ok(Element::new(ElementKind::Identifier(identifier), span))
                } else {
                    unreachable!()
                }
            },
        )
    }

    pub fn literal() -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
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
                form.expand()
                    .first()
                    .and_then(|token| match token.kind.clone() {
                        FormKind::Input(Token { kind, span }) => {
                            Some(Element::new(ElementKind::Literal(kind), span))
                        }
                        _ => None,
                    })
                    .ok_or_else(|| unreachable!())
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

    // Primary Elements

    pub fn primary() -> Classifier<Token, Element, ParseError> {
        Classifier::alternative([Self::delimited(), Self::token()])
    }

    // Unary Operations

    pub fn prefixed() -> Classifier<Token, Element, ParseError> {
        Classifier::ordered(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    if let TokenKind::Operator(operator) = &token.kind {
                        operator.is_prefix()
                    } else {
                        false
                    }
                })
                    .as_repeat(1, None),
                Self::primary(),
            ]),
            Order::map(|_, form: Form<Token, Element, ParseError>| {
                let prefixes = form.inputs();
                let operand = form.outputs()[0].clone();
                let mut unary = operand.clone();

                for prefix in prefixes {
                    unary = Element::new(
                        ElementKind::Unary {
                            operand: unary.into(),
                            operator: prefix,
                        },
                        Span::default(),
                    );
                }

                Ok(unary)
            })
        )
    }

    pub fn postfixed() -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
            Classifier::sequence([
                Self::primary(),
                Classifier::alternative([
                    Self::group(),
                    Self::collection(),
                    Self::bundle(),
                    Classifier::predicate(|token: &Token| {
                        if let TokenKind::Operator(operator) = &token.kind {
                            operator.is_postfix()
                        } else {
                            false
                        }
                    })
                ]).as_repeat(1, None),
            ]),
            |_, form| {
                let sequence = form.unwrap().clone();
                let operand = sequence[0].unwrap_output().unwrap();
                let postfixes = sequence[1].unwrap();
                let mut unary = operand.clone();

                for postfix in postfixes {
                    if let Some(token) = postfix.unwrap_input() {
                        unary = Element::new(
                            ElementKind::Unary {
                                operand: unary.into(),
                                operator: token,
                            },
                            Span::default(),
                        );
                    } else if let Some(element) = postfix.unwrap_output() {
                        match element.kind {
                            ElementKind::Group(_) => {
                                unary = Element::new(
                                    ElementKind::Invoke {
                                        target: unary.into(),
                                        parameters: element.into(),
                                    },
                                    Span::default(),
                                )
                            }
                            ElementKind::Collection(_) => {
                                unary = Element::new(
                                    ElementKind::Index {
                                        element: unary.into(),
                                        index: element.into(),
                                    },
                                    Span::default(),
                                )
                            }
                            ElementKind::Bundle(_) => {
                                unary = Element::new(
                                    ElementKind::Constructor {
                                        name: unary.into(),
                                        body: element.into(),
                                    },
                                    Span::default(),
                                )
                            }
                            _ => {}
                        }
                    }
                }

                Ok(unary)
            },
        )
    }


    pub fn unary() -> Classifier<Token, Element, ParseError> {
        Classifier::alternative([
            Self::prefixed(),
            Self::postfixed(),
            Self::primary(),
        ])
    }

    // Binary Operations

    pub fn binary(minimum: u8) -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::alternative([
                    Self::statement(),
                    Self::unary(),
                ]),
                Classifier::repeat(
                    Classifier::sequence([
                        Classifier::predicate(move |token: &Token| {
                            if let TokenKind::Operator(operator) = &token.kind {
                                if let Some(precedence) = operator.precedence() {
                                    precedence >= minimum
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
                let mut left = sequence[0].unwrap_output().unwrap();
                let operations = sequence[1].unwrap();
                let mut pairs = Vec::new();

                for operation in operations {
                    let sequence = operation.unwrap();
                    if sequence.len() >= 2 {
                        let operator = sequence[0].unwrap_input().unwrap();
                        let operand = sequence[1].unwrap_output().unwrap();
                        let precedence = if let TokenKind::Operator(op) = &operator.kind {
                            op.precedence().unwrap_or(0)
                        } else {
                            0
                        };
                        pairs.push((operator, operand, precedence));
                    }
                }

                left = Self::climb(left, pairs, minimum);
                Ok(left)
            },
        )
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
                ElementKind::Binary {
                    left: Box::new(left),
                    operator: operator.clone(),
                    right: Box::new(right),
                },
                span,
            );

            current = lookahead;
        }

        left
    }

    // Expressions

    pub fn expression(minimum: u8) -> Classifier<Token, Element, ParseError> {
        Classifier::alternative([Self::binary(minimum), Self::unary(), Self::primary()])
    }

    // Statements

    pub fn conditional() -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    if let TokenKind::Identifier(identifier) = &token.kind {
                        identifier == "if"
                    } else {
                        false
                    }
                })
                    .with_ignore(),
                Classifier::required(
                    Classifier::lazy(|| Self::element()),
                    Order::failure(|_, form| {
                        ParseError::new(ErrorKind::ExpectedCondition, form.span)
                    }),
                ),
                Classifier::required(
                    Classifier::lazy(|| Self::element()),
                    Order::failure(|_, form| ParseError::new(ErrorKind::ExpectedBody, form.span)),
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
                    Ok(Element::new(
                        ElementKind::Conditional {
                            condition: condition.into(),
                            then: then.into(),
                            alternate: Some(alternate.into()),
                        },
                        span,
                    ))
                } else {
                    let span = condition.span.mix(&then.span);
                    Ok(Element::new(
                        ElementKind::Conditional {
                            condition: condition.into(),
                            then: then.into(),
                            alternate: None,
                        },
                        span,
                    ))
                }
            },
        )
    }

    pub fn cycle() -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
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
                    Classifier::required(
                        Classifier::lazy(|| Self::element()),
                        Order::failure(|_, form| {
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
                    Classifier::required(
                        Classifier::lazy(|| Self::element()),
                        Order::failure(|_, form| {
                            ParseError::new(ErrorKind::ExpectedCondition, form.span)
                        }),
                    ),
                    Classifier::required(
                        Classifier::lazy(|| Self::element()),
                        Order::failure(|_, form| {
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
                    Ok(Element::new(
                        ElementKind::Cycle {
                            condition: None,
                            body: body.into(),
                        },
                        span,
                    ))
                } else if sequence.len() == 2 {
                    let condition = sequence[0].clone();
                    let body = sequence[1].clone();
                    let span = condition.span.mix(&body.span);
                    Ok(Element::new(
                        ElementKind::Cycle {
                            condition: Some(condition.into()),
                            body: body.into(),
                        },
                        span,
                    ))
                } else {
                    unreachable!()
                }
            },
        )
    }

    pub fn variable() -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    if let TokenKind::Identifier(identifier) = &token.kind {
                        identifier == "var"
                    } else {
                        false
                    }
                })
                    .with_ignore(),
                Classifier::alternative([
                    Self::token(),
                ]),
            ]),
            move |_, form| {
                let body = form.outputs()[0].clone();

                let (target, value) = if let ElementKind::Assignment { target, value } = body.kind {
                    (*target, Some(value))
                } else {
                    (body, None)
                };

                Ok(Element::new(
                    ElementKind::Symbolization(
                        SymbolKind::Variable {
                            target: target.into(),
                            value,
                            ty: None,
                            mutable: false,
                        }
                    ),
                    form.span,
                ))
            },
        )
    }

    pub fn structure() -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    if let TokenKind::Identifier(identifier) = &token.kind {
                        identifier == "struct"
                    } else {
                        false
                    }
                }),
                Self::token(),
                Self::bundle(),
            ]),
            |_, form| {
                let outputs = form.outputs().clone();

                let name = outputs[0].clone();

                let fields = if let ElementKind::Bundle(fields) = outputs[1].kind.clone() {
                    fields
                } else {
                    unreachable!()
                };

                Ok(Element::new(
                    ElementKind::Symbolization(
                        SymbolKind::Structure {
                            name: name.into(),
                            fields: fields.into(),
                        }
                    ),
                    outputs.span()
                ))
            }
        )
    }

    pub fn statement() -> Classifier<Token, Element, ParseError> {
        Classifier::alternative([Self::conditional(), Self::cycle(), Self::variable()])
    }

    // Top-Level Elements

    pub fn element() -> Classifier<Token, Element, ParseError> {
        Classifier::alternative([
            Self::statement(),
            Self::expression(0)
        ])
    }

    pub fn symbolization() -> Classifier<Token, Element, ParseError> {
        Classifier::alternative([
            Self::structure(),
            Self::variable(),
        ])
    }

    pub fn fallback() -> Classifier<Token, Element, ParseError> {
        Classifier::ordered(
            Classifier::predicate(|_token| true),
            Order::failure(
                |_, form: Form<Token, Element, ParseError>| {
                    ParseError::new(
                        ErrorKind::UnexpectedToken(form.unwrap_input().unwrap().kind),
                        form.span,
                    )
                },
            ),
        )
    }

    pub fn parser() -> Classifier<Token, Element, ParseError> {
        Classifier::repeat(
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
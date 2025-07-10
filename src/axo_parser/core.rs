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
use crate::axo_scanner::OperatorKind;
use crate::tree::{Node, Tree};

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
                    let span = Span::mix(&prefix.span, &unary.span);

                    unary = Element::new(
                        ElementKind::Unary {
                            operand: unary.into(),
                            operator: prefix,
                        },
                        span,
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
                    let span = Span::mix(&unary.span, &postfix.span);

                    if let Some(token) = postfix.unwrap_input() {
                        unary = Element::new(
                            ElementKind::Unary {
                                operand: unary.into(),
                                operator: token,
                            },
                            span,
                        );
                    } else if let Some(element) = postfix.unwrap_output() {
                        match element.kind {
                            ElementKind::Group(elements) => {
                                unary = Element::new(
                                    ElementKind::Invoke {
                                        target: unary.into(),
                                        arguments: elements,
                                    },
                                    span,
                                )
                            }
                            ElementKind::Collection(elements) => {
                                unary = Element::new(
                                    ElementKind::Index {
                                        target: unary.into(),
                                        indexes: elements,
                                    },
                                    span,
                                )
                            }
                            ElementKind::Bundle(elements) => {
                                unary = Element::new(
                                    ElementKind::Constructor {
                                        name: unary.into(),
                                        fields: elements,
                                    },
                                    span,
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

    /// Binary Operations

    // Transformation Classifiers

    pub fn member_access() -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::lazy(|| Self::primary()),
                Classifier::predicate(|token: &Token| {
                    matches!(token.kind, TokenKind::Operator(ref op) if op.as_slice() == [OperatorKind::Dot])
                }),
                Classifier::lazy(|| Self::primary()),
            ]),
            |_, form| {
                let sequence = form.unwrap();
                let object = sequence[0].unwrap_output().unwrap();
                let member = sequence[2].unwrap_output().unwrap();
                let span = Span::mix(&object.span, &member.span);

                Ok(Element::new(
                    ElementKind::Member {
                        object: object.into(),
                        member: member.into(),
                    },
                    span,
                ))
            },
        )
    }

    pub fn labeled_element() -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::lazy(|| Self::primary()),
                Classifier::predicate(|token: &Token| {
                    matches!(token.kind, TokenKind::Operator(ref op) if op.as_slice() == [OperatorKind::Colon])
                }),
                Classifier::lazy(|| Self::element()),
            ]),
            |_, form| {
                let sequence = form.unwrap();
                let label = sequence[0].unwrap_output().unwrap();
                let element = sequence[2].unwrap_output().unwrap();
                let span = Span::mix(&label.span, &element.span);

                Ok(Element::new(
                    ElementKind::Labeled {
                        label: label.into(),
                        element: element.into(),
                    },
                    span,
                ))
            },
        )
    }

    pub fn assignment() -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::lazy(|| Self::unary()),
                Classifier::predicate(|token: &Token| {
                    matches!(token.kind, TokenKind::Operator(ref op) if op.as_slice() == [OperatorKind::Equal])
                }),
                Classifier::lazy(|| Self::element()),
            ]),
            |_, form| {
                let sequence = form.unwrap();
                let target = sequence[0].unwrap_output().unwrap();
                let value = sequence[2].unwrap_output().unwrap();
                let span = Span::mix(&target.span, &value.span);

                Ok(Element::new(
                    ElementKind::Assignment {
                        target: target.into(),
                        value: value.into(),
                    },
                    span,
                ))
            },
        )
    }

    pub fn variable_declaration() -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::lazy(|| Self::primary()),
                Classifier::predicate(|token: &Token| {
                    matches!(token.kind, TokenKind::Operator(ref op) if op.as_slice() == [OperatorKind::Colon, OperatorKind::Equal])
                }),
                Classifier::lazy(|| Self::element()),
            ]),
            |_, form| {
                let sequence = form.unwrap();
                let target = sequence[0].unwrap_output().unwrap();
                let value = sequence[2].unwrap_output().unwrap();
                let span = Span::mix(&target.span, &value.span);

                let symbol = SymbolKind::Binding {
                    target: target.into(),
                    value: Some(value.into()),
                    ty: None,
                    mutable: false,
                };

                Ok(Element::new(
                    ElementKind::Symbolization(symbol),
                    span,
                ))
            },
        )
    }

    pub fn path_extension() -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::lazy(|| Self::primary()),
                Classifier::predicate(|token: &Token| {
                    matches!(token.kind, TokenKind::Operator(ref op) if op.as_slice() == [OperatorKind::Colon, OperatorKind::Colon])
                }),
                Classifier::lazy(|| Self::primary()),
            ]),
            |_, form| {
                let sequence = form.unwrap();
                let left = sequence[0].unwrap_output().unwrap();
                let right = sequence[2].unwrap_output().unwrap();
                let span = Span::mix(&left.span, &right.span);

                let kind = match &left.kind {
                    ElementKind::Path { tree } => {
                        // Extend existing path
                        let mut new_tree = tree.clone();

                        if let Some(root) = new_tree.root_mut() {
                            let mut current = root;

                            // Navigate to the deepest node
                            while current.has_children() {
                                let last_idx = current.child_count() - 1;
                                current = current.get_child_mut(last_idx).unwrap();
                            }

                            // Add the new path segment
                            current.add_value(right.into());
                        }

                        ElementKind::Path { tree: new_tree }
                    }
                    _ => {
                        // Create new path from two elements
                        let node = Node::with_children(
                            left.into(),
                            vec![Node::new(right.into())],
                        );

                        let tree = Tree::with_root_node(node);
                        ElementKind::Path { tree }
                    }
                };

                Ok(Element::new(kind, span))
            },
        )
    }

    pub fn compound_assignment() -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::lazy(|| Self::unary()),
                Classifier::predicate(|token: &Token| {
                    if let TokenKind::Operator(op) = &token.kind {
                        if let Some(OperatorKind::Composite(compound)) = op.as_slice().first() {
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
                let target = sequence[0].unwrap_output().unwrap();
                let operator_token = sequence[1].unwrap_input().unwrap();
                let value = sequence[2].unwrap_output().unwrap();
                let span = Span::mix(&target.span, &value.span);

                if let TokenKind::Operator(op) = &operator_token.kind {
                    if let Some(OperatorKind::Composite(compound)) = op.as_slice().first() {
                        if let Some(base_op) = OperatorKind::Composite(compound.clone()).decompound() {
                            let operation_token = Token {
                                kind: TokenKind::Operator(base_op),
                                span: operator_token.span.clone(),
                            };

                            let operation = Element::new(
                                ElementKind::Binary {
                                    left: target.clone().into(),
                                    operator: operation_token,
                                    right: value.into(),
                                },
                                span.clone(),
                            );

                            return Ok(Element::new(
                                ElementKind::Assignment {
                                    target: target.into(),
                                    value: operation.into(),
                                },
                                span,
                            ));
                        }
                    }
                }

                unreachable!()
            },
        )
    }

    pub fn binary() -> Classifier<Token, Element, ParseError> {
        Classifier::alternative([
            Self::member_access(),
            Self::assignment(),
            Self::labeled_element(),
            Self::variable_declaration(),
            Self::path_extension(),
            Self::compound_assignment(),
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

                    left = Self::climb(left, pairs, 0);
                    Ok(left)
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

    pub fn expression() -> Classifier<Token, Element, ParseError> {
        Classifier::alternative([Self::binary(), Self::unary(), Self::primary()])
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
                Self::token(),
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
                        SymbolKind::Binding {
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

                let fields = if let ElementKind::Bundle(elements) = outputs[1].kind.clone() {
                    elements
                } else {
                    unreachable!()
                };

                Ok(Element::new(
                    ElementKind::Symbolization(
                        SymbolKind::Structure {
                            name: name.into(),
                            entries: fields,
                        }
                    ),
                    outputs.span()
                ))
            }
        )
    }

    pub fn enumeration() -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    if let TokenKind::Identifier(identifier) = &token.kind {
                        identifier == "enum"
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

                let variants = if let ElementKind::Bundle(elements) = outputs[1].kind.clone() {
                    elements
                } else {
                    unreachable!()
                };

                Ok(Element::new(
                    ElementKind::Symbolization(
                        SymbolKind::Enumeration {
                            name: name.into(),
                            variants,
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
            Self::expression()
        ])
    }

    pub fn symbolization() -> Classifier<Token, Element, ParseError> {
        Classifier::alternative([
            Self::structure(),
            Self::enumeration(),
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
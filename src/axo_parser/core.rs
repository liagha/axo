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
        axo_schema::{
            Group, Sequence,
            Collection, Series,
            Bundle, Scope,
            Binary, Unary,
            Index, Invoke, Construct,
            Conditioned, Repeat, Walk, Map,
            Structure, Enumeration,
            Binding, Function, Interface, Implementation, Formation, Inclusion,
            Label, Access, Assign,
        },
        axo_parser::{Symbol, SymbolKind},
        axo_scanner::{Token, TokenKind, OperatorKind, PunctuationKind},
        axo_cursor::Spanned,
        artifact::Artifact,
        tree::{Node, Tree},
        thread::Arc,
    },
    log::trace,
};

impl Parser {
    pub fn identifier() -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
            Classifier::predicate(|token: &Token| matches!(token.kind, TokenKind::Identifier(_))),
            |_, form| {
                let input = form.inputs()[0].clone();
                let identifier = input.kind.unwrap_identifier();

                Ok(Element::new(ElementKind::Identifier(identifier), input.span))
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
                let input = form.inputs()[0].clone();

                Ok(Element::new(ElementKind::Literal(input.kind), input.span))
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
                        ElementKind::Unary(Unary::new(
                            prefix,
                            unary.into(),
                        )),
                        span,
                    );
                }

                Ok(unary)
            })
        )
    }

    pub fn suffixed() -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
            Classifier::sequence([
                Self::primary(),
                Classifier::alternative([
                    Self::group(),
                    Self::collection(),
                    Self::bundle(),
                    Classifier::predicate(|token: &Token| {
                        if let TokenKind::Operator(operator) = &token.kind {
                            operator.is_suffix()
                        } else {
                            false
                        }
                    })
                ]).as_repeat(1, None),
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
                            ElementKind::Group(elements) => {
                                unary = Element::new(
                                    ElementKind::Invoke(Invoke::new(unary.into(), elements.items)),
                                    span,
                                )
                            }
                            ElementKind::Collection(elements) => {
                                unary = Element::new(
                                    ElementKind::Index(Index::new(unary.into(), elements.items)),
                                    span,
                                )
                            }
                            ElementKind::Bundle(elements) => {
                                unary = Element::new(
                                    ElementKind::Construct(Construct::new(unary.into(), elements.items)),
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
            Self::suffixed(),
            Self::primary(),
        ])
    }

    pub fn access() -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
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
                let member = sequence[1].unwrap_output();
                let span = Span::mix(&object.span, &member.span);

                Ok(Element::new(
                    ElementKind::Access(Access::new(object.into(), member.into())),
                    span,
                ))
            },
        )
    }

    pub fn label() -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
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

                Ok(Element::new(
                    ElementKind::Label(Label::new(label.into(), element.into())),
                    span,
                ))
            },
        )
    }

    pub fn assign() -> Classifier<Token, Element, ParseError> {
        Classifier::transform(
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

                Ok(Element::new(
                    ElementKind::Assign(Assign::new(target.into(), value.into())),
                    span,
                ))
            },
        )
    }


    pub fn locate() -> Classifier<Token, Element, ParseError> {
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
                let left = sequence[0].unwrap_output();
                let right = sequence[2].unwrap_output();
                let span = Span::mix(&left.span, &right.span);

                let kind = match &left.kind {
                    ElementKind::Locate(tree) => {
                        let mut new_tree = tree.clone();

                        if let Some(root) = new_tree.root_mut() {
                            let mut current = root;

                            while current.has_children() {
                                let last_idx = current.child_count() - 1;
                                current = current.get_child_mut(last_idx).unwrap();
                            }

                            current.add_value(right.into());
                        }

                        ElementKind::Locate(new_tree)
                    }
                    _ => {
                        let node = Node::with_children(
                            left.into(),
                            vec![Node::new(right.into())],
                        );

                        let tree = Tree::with_root_node(node);
                        ElementKind::Locate(tree)
                    }
                };

                Ok(Element::new(kind, span))
            },
        )
    }

    pub fn compound() -> Classifier<Token, Element, ParseError> {
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
                let target = sequence[0].unwrap_output();
                let operator = sequence[1].unwrap_input();
                let value = sequence[2].unwrap_output();
                let span = Span::mix(&target.span, &value.span);

                if let TokenKind::Operator(op) = &operator.kind {
                    if let Some(OperatorKind::Composite(compound)) = op.as_slice().first() {
                        if let Some(base_op) = OperatorKind::Composite(compound.clone()).decompound() {
                            let operation_token = Token {
                                kind: TokenKind::Operator(base_op),
                                span: operator.span.clone(),
                            };

                            let operation = Element::new(
                                ElementKind::Binary(Binary::new(
                                    target.clone().into(),
                                    operation_token,
                                    value.into())
                                ),
                                span.clone(),
                            );

                            return Ok(Element::new(
                                ElementKind::Assign(Assign::new(target.into(), operation.into())),
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
            Self::access(),
            Self::assign(),
            Self::label(),
            Self::variable(),
            Self::locate(),
            Self::compound(),
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
                        ElementKind::Conditioned(Conditioned::new(condition.into(), then.into(), Some(alternate.into()))),
                        span,
                    ))
                } else {
                    let span = condition.span.mix(&then.span);
                    Ok(Element::new(
                        ElementKind::Conditioned(Conditioned::new(condition.into(), then.into(), None)),
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
                        ElementKind::Repeat(Repeat::new(None, body.into())),
                        span,
                    ))
                } else if sequence.len() == 2 {
                    let condition = sequence[0].clone();
                    let body = sequence[1].clone();
                    let span = condition.span.mix(&body.span);
                    Ok(Element::new(
                        ElementKind::Repeat(Repeat::new(Some(condition.into()), body.into())),
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
                        identifier == "var" || identifier == "const"
                    } else {
                        false
                    }
                }),
                Classifier::lazy(|| Self::primary()),
                Classifier::repeat(
                    Classifier::alternative([
                        Classifier::sequence([
                            Classifier::predicate(|token: &Token| {
                                matches!(token.kind, TokenKind::Operator(ref op) if op == &OperatorKind::Colon)
                            }),
                            Classifier::lazy(|| Self::element()),
                        ]),
                        Classifier::sequence([
                            Classifier::predicate(|token: &Token| {
                                matches!(token.kind, TokenKind::Operator(ref op) if op == &OperatorKind::Equal)
                            }),
                            Classifier::lazy(|| Self::element()),
                        ]),
                    ]),
                    0,
                    None,
                ),
            ]),
            |_, form| {
                let sequence = form.unwrap();

                let keyword = sequence[0].unwrap_input();
                let mutable = if let TokenKind::Identifier(identifier) = &keyword.kind {
                    identifier == "var"
                } else {
                    false
                };

                let target = sequence[1].unwrap_output();
                let operations = sequence[2].unwrap();

                let mut ty : Option<Box<Element>> = None;
                let mut value : Option<Box<Element>> = None;

                // Process the operations to extract type annotation and initialization
                for operation in operations {
                    let op_sequence = operation.unwrap();
                    if op_sequence.len() >= 2 {
                        let operator = op_sequence[0].unwrap_input();
                        let operand = op_sequence[1].unwrap_output();

                        if let TokenKind::Operator(op) = &operator.kind {
                            match op {
                                OperatorKind::Colon => {
                                    ty = Some(operand.into());
                                }
                                OperatorKind::Equal => {
                                    value = Some(operand.into());
                                }
                                _ => {}
                            }
                        }
                    }
                }

                let span = if let Some(ref val) = value {
                    Span::mix(&target.span, &val.span)
                } else if let Some(ref type_ann) = ty {
                    Span::mix(&target.span, &type_ann.span)
                } else {
                    target.span.clone()
                };

                let symbol = SymbolKind::Binding(Binding::new(target.into(), value, ty, mutable));

                Ok(Element::new(
                    ElementKind::Symbolize(symbol),
                    span,
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
                let body = outputs[1].clone().kind.unwrap_bundle();

                Ok(Element::new(
                    ElementKind::Symbolize(
                        SymbolKind::Structure(Structure::new(name.into(), body.items)),                    
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

                let body = outputs[1].clone().kind.unwrap_bundle();

                Ok(Element::new(
                    ElementKind::Symbolize(
                        SymbolKind::Enumeration(Enumeration::new(name.into(), body.items)),
                    ),
                    outputs.span()
                ))
            }
        )
    }

    pub fn statement() -> Classifier<Token, Element, ParseError> {
        Classifier::alternative([Self::conditional(), Self::cycle(), Self::variable()])
    }

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
                        ErrorKind::UnexpectedToken(form.unwrap_input().kind),
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
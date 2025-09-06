use {
    crate::{
        data::Str,
        internal::hash::Set,
        parser::{Element, ElementKind, Symbol, SymbolKind},
        scanner::{OperatorKind, Token, TokenKind},
        schema::*,
        tracker::{Location, Position, Span},
        resolver::{
            Resolver,
            scope::Scope,
        },
    }
};

impl<'resolver> Resolver<'resolver> {
    pub fn desugar(&mut self, element: Element<'resolver>) -> Element<'resolver> {
        match element.kind {
            ElementKind::Literal(literal) => {
                self.desugar_literal(literal)
            },
            ElementKind::Unary(unary) => {
                self.desugar_unary(unary, element.span)
            },
            ElementKind::Binary(binary) => {
                self.desugar_binary(binary, element.span)
            }
            ElementKind::Access(access) => {
                let target = Box::new(self.desugar(*access.target));
                let member = Box::new(self.desugar(*access.member));
                Element::new(
                    ElementKind::Access(Access::new(target, member)),
                    element.span,
                )
            }
            ElementKind::Index(index) => {
                let target = Box::new(self.desugar(*index.target));
                let indexes = index.members.into_iter().map(|i| self.desugar(i)).collect();
                Element::new(
                    ElementKind::Index(Index::new(target, indexes)),
                    element.span,
                )
            }
            ElementKind::Invoke(invoke) => {
                let target = Box::new(self.desugar(*invoke.target));
                let arguments = invoke
                    .members
                    .into_iter()
                    .map(|a| self.desugar(a))
                    .collect();
                Element::new(
                    ElementKind::Invoke(Invoke::new(target, arguments)),
                    element.span,
                )
            }
            ElementKind::Construct(construct) => {
                let target = Box::new(self.desugar(*construct.target));
                let fields = construct
                    .members
                    .into_iter()
                    .map(|f| self.desugar(f))
                    .collect();
                Element::new(
                    ElementKind::Construct(Structure::new(target, fields)),
                    element.span,
                )
            }
            ElementKind::Delimited(delimited) => {
                let items = delimited
                    .items
                    .into_iter()
                    .map(|i| self.desugar(i))
                    .collect();
                Element::new(
                    ElementKind::Delimited(
                        Delimited::new(
                            delimited.start,
                            items,
                            delimited.separator,
                            delimited.end,
                        )
                    ),
                    element.span
                )
            }
            ElementKind::Label(label) => {
                let label_val = Box::new(self.desugar(*label.label));
                let element_val = Box::new(self.desugar(*label.element));
                Element::new(
                    ElementKind::Label(Label::new(label_val, element_val)),
                    element.span,
                )
            }
            ElementKind::Assign(assign) => {
                let target = Box::new(self.desugar(*assign.target));
                let value = Box::new(self.desugar(*assign.value));
                Element::new(
                    ElementKind::Assign(Assign::new(target, value)),
                    element.span,
                )
            }
            ElementKind::Conditional(conditional) => {
                let condition = Box::new(self.desugar(*conditional.condition));
                let then = Box::new(self.desugar(*conditional.then));
                let alternate = conditional.alternate.map(|a| Box::new(self.desugar(*a)));
                Element::new(
                    ElementKind::Conditional(Conditional::new(condition, then, alternate)),
                    element.span,
                )
            }
            ElementKind::While(repeat) => {
                let condition = repeat.condition.map(|c| Box::new(self.desugar(*c)));
                let body = Box::new(self.desugar(*repeat.body));
                Element::new(
                    ElementKind::While(While::new(condition, body)),
                    element.span,
                )
            }
            ElementKind::Cycle(cycle) => {
                let clause = Box::new(self.desugar(*cycle.clause));
                let body = Box::new(self.desugar(*cycle.body));
                Element::new(ElementKind::Cycle(Cycle::new(clause, body)), element.span)
            }
            ElementKind::Return(value) => {
                let value = value.map(|v| Box::new(self.desugar(*v)));
                Element::new(ElementKind::Return(value), element.span)
            }
            ElementKind::Break(value) => {
                let value = value.map(|v| Box::new(self.desugar(*v)));
                Element::new(ElementKind::Break(value), element.span)
            }
            ElementKind::Continue(value) => {
                let value = value.map(|v| Box::new(self.desugar(*v)));
                Element::new(ElementKind::Continue(value), element.span)
            }
            ElementKind::Symbolize(mut symbol) => {
                self.desugar_symbol(&mut symbol);
                Element::new(ElementKind::Symbolize(symbol), element.span)
            }
            ElementKind::Procedural(procedural) => {
                let body = Box::new(self.desugar(*procedural.body));
                Element::new(ElementKind::Procedural(Procedural::new(body)), element.span)
            }
            _ => element,
        }
    }

    fn desugar_literal(&self, literal: Token<'resolver>) -> Element<'resolver> {
        let span = literal.span;

        match literal.kind {
            TokenKind::Float(_) => {
                let target = Element::new(
                    ElementKind::Literal(Token::new(TokenKind::Identifier(Str::from("Float")), span)),
                    span,
                );

                Element::new(
                    ElementKind::Construct(
                        Structure::new(
                            Box::new(target),
                            vec![
                                Element::new(ElementKind::Literal(literal), span),
                                Element::new(ElementKind::Literal(Token::new(TokenKind::Integer(32), span)), span)
                            ],
                        )
                    ),
                    span,
                )
            },
            TokenKind::Integer(_) => {
                let target = Element::new(
                    ElementKind::Literal(Token::new(TokenKind::Identifier(Str::from("Integer")), span)),
                    span,
                );

                Element::new(
                    ElementKind::Construct(
                        Structure::new(
                            Box::new(target),
                            vec![
                                Element::new(ElementKind::Literal(literal), span),
                                Element::new(ElementKind::Literal(Token::new(TokenKind::Integer(32), span)), span),
                                Element::new(ElementKind::Literal(Token::new(TokenKind::Boolean(true), span)), span),
                            ],
                        )
                    ),
                    span,
                )
            },
            TokenKind::Boolean(_) => {
                let target = Element::new(
                    ElementKind::Literal(Token::new(TokenKind::Identifier(Str::from("Boolean")), span)),
                    span,
                );

                Element::new(
                    ElementKind::Construct(
                        Structure::new(
                            Box::new(target),
                            vec![
                                Element::new(ElementKind::Literal(literal), span),
                            ],
                        )
                    ),
                    span,
                )
            },
            TokenKind::String(_) => {
                let target = Element::new(
                    ElementKind::Literal(Token::new(TokenKind::Identifier(Str::from("String")), span)),
                    span,
                );

                Element::new(
                    ElementKind::Construct(
                        Structure::new(
                            Box::new(target),
                            vec![
                                Element::new(ElementKind::Literal(literal), span),
                            ],
                        )
                    ),
                    span,
                )
            },
            TokenKind::Character(_) => {
                let target = Element::new(
                    ElementKind::Literal(Token::new(TokenKind::Identifier(Str::from("Character")), span)),
                    span,
                );

                Element::new(
                    ElementKind::Construct(
                        Structure::new(
                            Box::new(target),
                            vec![
                                Element::new(ElementKind::Literal(literal), span),
                            ],
                        )
                    ),
                    span,
                )
            },
            _ => Element::new(ElementKind::Literal(literal.clone()), span),
        }
    }

    fn desugar_unary(
        &mut self,
        unary: Unary<Token<'resolver>, Box<Element<'resolver>>>,
        span: Span<'resolver>,
    ) -> Element<'resolver> {
        let operand = Box::new(self.desugar(*unary.operand));

        if let TokenKind::Operator(operator) = &unary.operator.kind {
            match operator.as_slice() {
                [OperatorKind::Minus] => {
                    let member = Element::new(
                        ElementKind::Invoke(Invoke::new(
                            Box::new(Element::new(
                                ElementKind::Literal(Token::new(
                                    TokenKind::Identifier(Str::from("negate")),
                                    span,
                                )),
                                span,
                            )),
                            Vec::new(),
                        )),
                        span,
                    );

                    return Element::new(
                        ElementKind::Access(Access::new(operand, Box::new(member))),
                        span,
                    );
                }
                [OperatorKind::Exclamation] => {
                    let member = Element::new(
                        ElementKind::Invoke(Invoke::new(
                            Box::new(Element::new(
                                ElementKind::Literal(Token::new(
                                    TokenKind::Identifier(Str::from("not")),
                                    span,
                                )),
                                span,
                            )),
                            Vec::new(),
                        )),
                        span,
                    );

                    return Element::new(
                        ElementKind::Access(Access::new(operand, Box::new(member))),
                        span,
                    );
                }
                [OperatorKind::Tilde] => {
                    let member = Element::new(
                        ElementKind::Invoke(Invoke::new(
                            Box::new(Element::new(
                                ElementKind::Literal(Token::new(
                                    TokenKind::Identifier(Str::from("bitwise_not")),
                                    span,
                                )),
                                span,
                            )),
                            Vec::new(),
                        )),
                        span,
                    );

                    return Element::new(
                        ElementKind::Access(Access::new(operand, Box::new(member))),
                        span,
                    );
                }
                _ => {}
            }
        }

        Element::new(
            ElementKind::Unary(Unary::new(unary.operator, operand)),
            span,
        )
    }

    fn desugar_binary(
        &mut self,
        binary: Binary<Box<Element<'resolver>>, Token<'resolver>, Box<Element<'resolver>>>,
        span: Span<'resolver>,
    ) -> Element<'resolver> {
        let left = Box::new(self.desugar(*binary.left));
        let right = Box::new(self.desugar(*binary.right));

        if let TokenKind::Operator(operator) = &binary.operator.kind {
            let method_name = match operator.as_slice() {
                [OperatorKind::Plus] => "add",
                [OperatorKind::Minus] => "subtract",
                [OperatorKind::Star] => "multiply",
                [OperatorKind::Slash] => "divide",
                [OperatorKind::Percent] => "modulus",
                [OperatorKind::Ampersand, OperatorKind::Ampersand] => "and",
                [OperatorKind::Pipe, OperatorKind::Pipe] => "or",
                [OperatorKind::Caret] => "xor",
                [OperatorKind::Ampersand] => "bitwise_and",
                [OperatorKind::Pipe] => "bitwise_or",
                [OperatorKind::LeftAngle, OperatorKind::LeftAngle] => "shift_left",
                [OperatorKind::RightAngle, OperatorKind::RightAngle] => "shift_right",
                [OperatorKind::Equal, OperatorKind::Equal] => "equal",
                [OperatorKind::Exclamation, OperatorKind::Equal] => "not_equal",
                [OperatorKind::LeftAngle] => "less",
                [OperatorKind::LeftAngle, OperatorKind::Equal] => "less_or_equal",
                [OperatorKind::RightAngle] => "greater",
                [OperatorKind::RightAngle, OperatorKind::Equal] => "greater_or_equal",
                _ => {
                    return Element::new(
                        ElementKind::Binary(Binary::new(left, binary.operator, right)),
                        span,
                    )
                }
            };

            let method = Box::new(Element::new(
                ElementKind::Invoke(Invoke::new(
                    Box::new(Element::new(
                        ElementKind::Literal(Token::new(
                            TokenKind::Identifier(Str::from(method_name)),
                            span,
                        )),
                        span,
                    )),
                    vec![*right],
                )),
                span,
            ));

            Element::new(
                ElementKind::Access(Access::new(left, method)),
                span,
            )
        } else {
            Element::new(
                ElementKind::Binary(Binary::new(left, binary.operator, right)),
                span,
            )
        }
    }

    fn desugar_symbol(&mut self, symbol: &mut Symbol<'resolver>) {
        match &mut symbol.kind {
            SymbolKind::Inclusion(inclusion) => {
                *inclusion.target = self.desugar(*inclusion.target.clone());
            }
            SymbolKind::Extension(extension) => {
                extension.target = Box::new(self.desugar(*extension.target.clone()));
                extension.members.iter_mut().for_each(|member| {
                    self.desugar_symbol(member);
                });
            }
            SymbolKind::Binding(binding) => {
                if let Some(value) = &binding.value {
                    binding.value = Some(Box::new(self.desugar(*value.clone())));
                }
            }
            SymbolKind::Structure(structure) => {
                structure.members.iter_mut().for_each(|member| {
                    self.desugar_symbol(member);
                });
            }
            SymbolKind::Enumeration(enumeration) => {
                enumeration.members.iter_mut().for_each(|member| {
                    self.desugar_symbol(member);
                });
            }
            SymbolKind::Method(method) => {
                method.members.iter_mut().for_each(|member| {
                    self.desugar_symbol(member);
                });

                method.body = Box::new(self.desugar(*method.body.clone()));

                if let Some(output) = &method.output {
                    method.output = Some(Box::new(self.desugar(*output.clone())));
                }
            }
            _ => {}
        }
    }
}
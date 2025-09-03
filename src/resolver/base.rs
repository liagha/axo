use crate::data::Str;
use crate::internal::hash::Set;
use crate::parser::{Element, ElementKind, Symbol, Symbolic};
use crate::resolver::Resolver;
use crate::resolver::scope::Scope;
use crate::scanner::{OperatorKind, Token, TokenKind};
use crate::schema::*;
use crate::tracker::{Location, Position, Span};

impl<'resolver> Resolver<'resolver> {
    pub fn desugar(&mut self, element: Element<'resolver>) -> Element<'resolver> {
        match element.kind {
            ElementKind::Literal(literal) => {
                match literal.kind {
                    TokenKind::Float(_) => {
                        let target = Element::new(
                            ElementKind::Literal(
                                Token::new(
                                    TokenKind::Identifier(Str::from("Float")),
                                    Span::void()
                                )
                            ),
                            Span::void()
                        );

                        let member = Element::new(
                            ElementKind::Invoke(
                                Invoke::new(
                                    Box::new(Element::new(
                                        ElementKind::Literal(
                                            Token::new(
                                                TokenKind::Identifier(Str::from("new")),
                                                Span::void()
                                            )
                                        ),
                                        Span::void()
                                    )),
                                    vec![
                                        Element::new(ElementKind::Literal(literal), element.span),
                                    ]
                                )
                            ),
                            Span::void()
                        );

                        Element::new(
                            ElementKind::Access(
                                Access::new(
                                    Box::new(target),
                                    Box::new(member)
                                )
                            ),
                            element.span
                        )
                    }
                    TokenKind::Integer(_) => {
                        let target = Element::new(
                            ElementKind::Literal(
                                Token::new(
                                    TokenKind::Identifier(Str::from("Integer")),
                                    Span::void()
                                )
                            ),
                            Span::void()
                        );

                        let member = Element::new(
                            ElementKind::Invoke(
                                Invoke::new(
                                    Box::new(Element::new(
                                        ElementKind::Literal(
                                            Token::new(
                                                TokenKind::Identifier(Str::from("new")),
                                                Span::void()
                                            )
                                        ),
                                        Span::void()
                                    )),
                                    vec![
                                        Element::new(ElementKind::Literal(literal), element.span),
                                    ]
                                )
                            ),
                            Span::void()
                        );

                        Element::new(
                            ElementKind::Access(
                                Access::new(
                                    Box::new(target),
                                    Box::new(member)
                                )
                            ),
                            element.span
                        )
                    }
                    TokenKind::Boolean(_) => {
                        let target = Element::new(
                            ElementKind::Literal(
                                Token::new(
                                    TokenKind::Identifier(Str::from("Boolean")),
                                    Span::void()
                                )
                            ),
                            Span::void()
                        );

                        let member = Element::new(
                            ElementKind::Invoke(
                                Invoke::new(
                                    Box::new(Element::new(
                                        ElementKind::Literal(
                                            Token::new(
                                                TokenKind::Identifier(Str::from("new")),
                                                Span::void()
                                            )
                                        ),
                                        Span::void()
                                    )),
                                    vec![
                                        Element::new(ElementKind::Literal(literal), element.span),
                                    ]
                                )
                            ),
                            Span::void()
                        );

                        Element::new(
                            ElementKind::Access(
                                Access::new(
                                    Box::new(target),
                                    Box::new(member)
                                )
                            ),
                            element.span
                        )
                    }
                    TokenKind::String(_) => {
                        let target = Element::new(
                            ElementKind::Literal(
                                Token::new(
                                    TokenKind::Identifier(Str::from("String")),
                                    Span::void()
                                )
                            ),
                            Span::void()
                        );

                        let member = Element::new(
                            ElementKind::Invoke(
                                Invoke::new(
                                    Box::new(Element::new(
                                        ElementKind::Literal(
                                            Token::new(
                                                TokenKind::Identifier(Str::from("new")),
                                                Span::void()
                                            )
                                        ),
                                        Span::void()
                                    )),
                                    vec![
                                        Element::new(ElementKind::Literal(literal), element.span),
                                    ]
                                )
                            ),
                            Span::void()
                        );

                        Element::new(
                            ElementKind::Access(
                                Access::new(
                                    Box::new(target),
                                    Box::new(member)
                                )
                            ),
                            element.span
                        )
                    }
                    TokenKind::Character(_) => {
                        let target = Element::new(
                            ElementKind::Literal(
                                Token::new(
                                    TokenKind::Identifier(Str::from("Character")),
                                    Span::void()
                                )
                            ),
                            Span::void()
                        );

                        let member = Element::new(
                            ElementKind::Invoke(
                                Invoke::new(
                                    Box::new(Element::new(
                                        ElementKind::Literal(
                                            Token::new(
                                                TokenKind::Identifier(Str::from("new")),
                                                Span::void()
                                            )
                                        ),
                                        Span::void()
                                    )),
                                    vec![
                                        Element::new(ElementKind::Literal(literal), element.span),
                                    ]
                                )
                            ),
                            Span::void()
                        );

                        Element::new(
                            ElementKind::Access(
                                Access::new(
                                    Box::new(target),
                                    Box::new(member)
                                )
                            ),
                            element.span
                        )
                    }
                    _ => {
                        Element::new(ElementKind::Literal(literal), element.span)
                    }
                }
            }
            ElementKind::Unary(unary) => {
                let operand = Box::new(self.desugar(*unary.get_operand().clone()));

                if let TokenKind::Operator(operator) = &unary.get_operator().kind {
                    match operator.as_slice() {
                        [OperatorKind::Exclamation] => {
                            let member = Element::new(
                                ElementKind::Invoke(
                                    Invoke::new(
                                        Box::new(Element::new(
                                            ElementKind::Literal(
                                                Token::new(
                                                    TokenKind::Identifier(Str::from("not")),
                                                    Span::void()
                                                )
                                            ),
                                            Span::void()
                                        )),
                                        Vec::new()
                                    )
                                ),
                                Span::void()
                            );

                            Element::new(
                                ElementKind::Access(
                                    Access::new(
                                        operand,
                                        Box::new(member)
                                    )
                                ),
                                element.span
                            )
                        }
                        _ => {
                            Element::new(
                                ElementKind::Unary(Unary::new(unary.get_operator().clone(), operand)),
                                element.span
                            )
                        }
                    }
                } else {
                    Element::new(
                        ElementKind::Unary(Unary::new(unary.get_operator().clone(), operand)),
                        element.span
                    )
                }
            }
            ElementKind::Binary(binary) => {
                let left = Box::new(self.desugar(*binary.get_left().clone()));
                let right = Box::new(self.desugar(*binary.get_right().clone()));
                Element::new(
                    ElementKind::Binary(Binary::new(left, binary.get_operator().clone(), right)),
                    element.span
                )
            }
            ElementKind::Access(access) => {
                let target = Box::new(self.desugar(*access.get_target().clone()));
                let member = Box::new(self.desugar(*access.get_member().clone()));
                Element::new(
                    ElementKind::Access(Access::new(target, member)),
                    element.span
                )
            }
            ElementKind::Index(index) => {
                let target = Box::new(self.desugar(*index.get_target().clone()));
                let indexes = index.get_indexes().into_iter().map(|i| self.desugar(i.clone())).collect();
                Element::new(
                    ElementKind::Index(Index::new(target, indexes)),
                    element.span
                )
            }
            ElementKind::Invoke(invoke) => {
                let target = Box::new(self.desugar(*invoke.get_target().clone()));
                let arguments = invoke.get_arguments().into_iter().map(|a| self.desugar(a.clone())).collect();
                Element::new(
                    ElementKind::Invoke(Invoke::new(target, arguments)),
                    element.span
                )
            }
            ElementKind::Construct(construct) => {
                let target = Box::new(self.desugar(*construct.get_target().clone()));
                let fields = construct.get_fields().into_iter().map(|f| self.desugar(f.clone())).collect();
                Element::new(
                    ElementKind::Construct(Structure::new(target, fields)),
                    element.span
                )
            }
            ElementKind::Group(group) => {
                let items = group.items.into_iter().map(|i| self.desugar(i)).collect();
                Element::new(
                    ElementKind::Group(Group::new(items)),
                    element.span
                )
            }
            ElementKind::Sequence(sequence) => {
                let items = sequence.items.into_iter().map(|i| self.desugar(i)).collect();
                Element::new(
                    ElementKind::Sequence(Sequence::new(items)),
                    element.span
                )
            }
            ElementKind::Collection(collection) => {
                let items = collection.items.into_iter().map(|i| self.desugar(i)).collect();
                Element::new(
                    ElementKind::Collection(Collection::new(items)),
                    element.span
                )
            }
            ElementKind::Series(series) => {
                let items = series.items.into_iter().map(|i| self.desugar(i)).collect();
                Element::new(
                    ElementKind::Series(Series::new(items)),
                    element.span
                )
            }
            ElementKind::Bundle(bundle) => {
                let items = bundle.items.into_iter().map(|i| self.desugar(i)).collect();
                Element::new(
                    ElementKind::Bundle(Bundle::new(items)),
                    element.span
                )
            }
            ElementKind::Block(block) => {
                let items = block.items.into_iter().map(|i| self.desugar(i)).collect();
                Element::new(
                    ElementKind::Block(Block::new(items)),
                    element.span
                )
            }
            ElementKind::Label(label) => {
                let label_val = Box::new(self.desugar(*label.get_label().clone()));
                let element_val = Box::new(self.desugar(*label.get_element().clone()));
                Element::new(
                    ElementKind::Label(Label::new(label_val, element_val)),
                    element.span
                )
            }
            ElementKind::Assign(assign) => {
                let target = Box::new(self.desugar(*assign.get_target().clone()));
                let value = Box::new(self.desugar(*assign.get_value().clone()));
                Element::new(
                    ElementKind::Assign(Assign::new(target, value)),
                    element.span
                )
            }
            ElementKind::Conditional(conditional) => {
                let condition = Box::new(self.desugar(*conditional.get_condition().clone()));
                let then = Box::new(self.desugar(*conditional.get_then().clone()));
                let alternate = conditional.get_alternate().map(|a| Box::new(self.desugar(*a.clone())));
                Element::new(
                    ElementKind::Conditional(Conditional::new(condition, then, alternate)),
                    element.span
                )
            }
            ElementKind::While(repeat) => {
                let condition = repeat.get_condition().map(|c| Box::new(self.desugar(*c.clone())));
                let body = Box::new(self.desugar(*repeat.get_body().clone()));
                Element::new(
                    ElementKind::While(While::new(condition, body)),
                    element.span
                )
            }
            ElementKind::Cycle(cycle) => {
                let clause = Box::new(self.desugar(*cycle.get_clause().clone()));
                let body = Box::new(self.desugar(*cycle.get_body().clone()));
                Element::new(
                    ElementKind::Cycle(Cycle::new(clause, body)),
                    element.span
                )
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
            ElementKind::Symbolize(_) => {
                element
            }
            ElementKind::Procedural(procedural) => {
                let body = Box::new(self.desugar(*procedural.body));
                Element::new(
                    ElementKind::Procedural(Procedural::new(body)),
                    element.span
                )
            }
            _ => element,
        }
    }
}
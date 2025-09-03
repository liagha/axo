use crate::data::Str;
use crate::internal::hash::Set;
use crate::parser::{Element, ElementKind, Symbol, Symbolic};
use crate::resolver::Resolver;
use crate::resolver::scope::Scope;
use crate::scanner::{OperatorKind, Token, TokenKind};
use crate::schema::{Access, Invoke, Label, Structure};
use crate::tracker::{Location, Position, Span};

impl<'resolver> Resolver<'resolver> {
    pub fn desugar(&mut self, element: Element<'resolver>) -> Element<'resolver> {
        match &element.kind {
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
                                        element.clone(),
                                    ]
                                )
                            ),
                            Span::void()
                        );

                        Element::new(
                            ElementKind::access(
                                Access::new(
                                    Box::new(target),
                                    Box::new(member)
                                )
                            ),
                            Span::void()
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
                                        element.clone(),
                                    ]
                                )
                            ),
                            Span::void()
                        );

                        Element::new(
                            ElementKind::access(
                                Access::new(
                                    Box::new(target),
                                    Box::new(member)
                                )
                            ),
                            Span::void()
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
                                        element.clone(),
                                    ]
                                )
                            ),
                            Span::void()
                        );

                        Element::new(
                            ElementKind::access(
                                Access::new(
                                    Box::new(target),
                                    Box::new(member)
                                )
                            ),
                            Span::void()
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
                                        element.clone(),
                                    ]
                                )
                            ),
                            Span::void()
                        );

                        Element::new(
                            ElementKind::access(
                                Access::new(
                                    Box::new(target),
                                    Box::new(member)
                                )
                            ),
                            Span::void()
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
                                        element.clone(),
                                    ]
                                )
                            ),
                            Span::void()
                        );

                        Element::new(
                            ElementKind::access(
                                Access::new(
                                    Box::new(target),
                                    Box::new(member)
                                )
                            ),
                            Span::void()
                        )
                    }
                    _ => {
                        element
                    }
                }
            }
            ElementKind::Unary(unary) => {
                let operand = self.desugar(*unary.get_operand().clone());

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
                                ElementKind::access(
                                    Access::new(
                                        Box::new(operand),
                                        Box::new(member)
                                    )
                                ),
                                Span::void()
                            )
                        }
                        _ => {
                            element
                        }
                    }
                } else { 
                    element
                }
            }
            ElementKind::Binary(_) => {
                element
            }
            _ => {
                element
            }
        }
    }
}
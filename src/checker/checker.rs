use crate::checker::primitive;
use crate::checker::types::Type;
use crate::parser::{Element, ElementKind};
use crate::scanner::TokenKind;
use crate::tracker::{Location, Position, Span};

pub struct Checker {
    
}

impl Checker {
    pub fn new() -> Checker {
        Checker {}
    }
    
    pub fn check(&mut self, element: Element) -> Type {
        match element.kind {
            ElementKind::Literal(literal) => {
                match literal {
                    TokenKind::Float(value) => {
                        Type::new(
                            primitive::Float {
                                value,
                                size: 64,
                            },
                            Span::point(Position::new(Location::Void))
                        )
                    }
                    TokenKind::Integer(value) => {
                        Type::new(
                            primitive::Integer {
                                value,
                                size: 64,
                            },
                            Span::point(Position::new(Location::Void))
                        )
                    }
                    TokenKind::Boolean(value) => {
                        Type::new(
                            primitive::Boolean {
                                value,
                            },
                            Span::point(Position::new(Location::Void))
                        )
                    }
                    TokenKind::String(_) => {
                        unimplemented!()
                    }
                    TokenKind::Character(_) => {
                        unimplemented!()
                    }
                    TokenKind::Operator(_) => {
                        unimplemented!()
                    }
                    TokenKind::Identifier(_) => {
                        unimplemented!()
                    }
                    TokenKind::Punctuation(_) => {
                        unimplemented!()
                    }
                    TokenKind::Comment(_) => {
                        unimplemented!()
                    }
                }
            }
            _ => {
                unreachable!()
            }
        }
    }
}
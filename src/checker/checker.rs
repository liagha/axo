use {
    super::{
        CheckError,
    },
    crate::{
        checker::{
            primitive,
            types::Type,
        },
        parser::{Element, ElementKind},
        scanner::TokenKind,
        tracker::{Location, Position, Span},
        internal::hash::Set,
        schema::Group,
    }
};

#[derive(Debug)]
pub struct Checker<'checker: 'static> {
    types: Set<Type<'checker>>,
    errors: Vec<CheckError<'checker>>,
}

impl<'checker: 'static> Clone for Checker<'checker> {
    fn clone(&self) -> Self {
        Self {
            types: self.types.clone(),
            errors: vec![],
        }
    }
}

impl<'checker: 'static> Checker<'checker> {
    pub fn new() -> Checker<'checker> {
        Checker {
            types: Set::new(),
            errors: vec![],
        }
    }

    pub fn unit() -> Type<'checker> {
        Type::new(
            Group::new(vec![]),
            Span::point(Position::new(Location::Void))
        )
    }
    
    pub fn check(&mut self, element: &Element<'checker>) -> Type<'checker> {
        match &element.kind {
            ElementKind::Literal(literal) => {
                match literal {
                    TokenKind::Float(value) => {
                        Type::new(
                            value.clone(),
                            Span::point(Position::new(Location::Void))
                        )
                    }
                    TokenKind::Integer(value) => {
                        Type::new(
                            value.clone(),
                            Span::point(Position::new(Location::Void))
                        )
                    }
                    TokenKind::Boolean(value) => {
                        Type::new(
                            value.clone(),
                            Span::point(Position::new(Location::Void))
                        )
                    }
                    TokenKind::String(value) => {
                        Type::new(
                            value.clone(),
                            Span::point(Position::new(Location::Void))
                        )
                    }
                    TokenKind::Character(value) => {
                        Type::new(
                            value.clone(),
                            Span::point(Position::new(Location::Void))
                        )
                    }
                    TokenKind::Operator(_) => {
                        Self::unit()
                    }
                    TokenKind::Identifier(_) => {
                        Self::unit()
                    }
                    TokenKind::Punctuation(_) => {
                        Self::unit()
                    }
                    TokenKind::Comment(_) => {
                        Self::unit()
                    }
                }
            }
            ElementKind::Group(group) => {
                let types = group.items.iter().map(|item| self.check(item)).collect::<Vec<_>>();

                Type::new(
                    Group::new(types),
                    Span::point(Position::new(Location::Void))
                )
            }
            ElementKind::Block(block) => {
                if let Some(output) = block.items.last() {
                    Type::new(
                        self.check(output),
                        Span::point(Position::new(Location::Void))
                    )
                } else {
                    Self::unit()
                }
            }
            ElementKind::Unary(unary) => {
                self.check(unary.get_operand())
            }
            ElementKind::Binary(binary) => {
                self.check(binary.get_left())
            }

            ElementKind::Identifier(_) => {
                Self::unit()
            }
            ElementKind::Procedural(procedural) => {
                self.check(&*procedural.body)
            }
            ElementKind::Sequence(_) => {
                Self::unit()
            }
            ElementKind::Collection(_) => {
                Self::unit()
            }
            ElementKind::Series(_) => {
                Self::unit()
            }
            ElementKind::Bundle(_) => {
                Self::unit()
            }
            ElementKind::Label(_) => {
                Self::unit()
            }
            ElementKind::Access(_) => {
                Self::unit()
            }
            ElementKind::Index(_) => {
                Self::unit()
            }
            ElementKind::Invoke(_) => {
                Self::unit()
            }
            ElementKind::Construct(structure) => {
                Type::new(
                    structure.clone(),
                    Span::point(Position::new(Location::Void))
                )
            }
            ElementKind::Conditional(conditional) => {
                self.check(conditional.get_then())
            }
            ElementKind::Repeat(repeat) => {
                self.check(repeat.get_body())
            }
            ElementKind::Iterate(iterate) => {
                self.check(iterate.get_body())
            }
            ElementKind::Assign(assign) => {
                self.check(assign.get_value())
            }
            ElementKind::Symbolize(_) => {
                Self::unit()
            }
            ElementKind::Produce(_) => {
                Self::unit()
            }
            ElementKind::Abort(_) => {
                Self::unit()
            }
            ElementKind::Pass(_) => {
                Self::unit()
            }
        }
    }
}
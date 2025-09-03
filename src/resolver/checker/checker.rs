use crate::{
    data::Str,
    internal::hash::Map,
    parser::{
        Element, ElementKind, Symbol,
    },
};
use crate::resolver::checker::types::TypeKind;
use crate::parser::Symbolic;
use crate::resolver::checker::{
    types::Type,
    CheckError,
};
use crate::resolver::Resolver;
use crate::scanner::{Token, TokenKind};
use crate::schema::Structure;

impl<'resolver> Resolver<'resolver> {
    pub fn infer_element(&mut self, element: &Element<'resolver>) -> Type<'resolver> {
        match &element.kind {
            ElementKind::Literal(literal) => {
                self.infer_literal(literal)
            }
            ElementKind::Procedural(_) => {
                Type::unit()
            }
            ElementKind::Group(group) => {
                Type::new(
                    TypeKind::Tuple {
                        items: group.items
                            .iter()
                            .map(|item| {
                                self.infer_element(item)
                            })
                            .collect::<Vec<_>>()
                    }
                )
            }
            ElementKind::Sequence(_) => {
                Type::unit()
            }
            ElementKind::Collection(_) => {
                Type::unit()
            }
            ElementKind::Series(_) => {
                Type::unit()
            }
            ElementKind::Bundle(_) => {
                Type::unit()
            }
            ElementKind::Block(block) => {
                if let Some(element) = block.items.last() {
                    self.infer_element(element)
                } else {
                    Type::unit()
                }
            }
            ElementKind::Unary(unary) => {
                self.infer_element(unary.get_operand())
            }
            ElementKind::Binary(binary) => {
                self.infer_element(binary.get_left())
            }
            ElementKind::Label(_) => {
                Type::unit()
            }
            ElementKind::Access(_) => {
                Type::unit()
            }
            ElementKind::Index(_) => {
                Type::unit()
            }
            ElementKind::Invoke(_) => {
                Type::unit()
            }
            ElementKind::Construct(construct) => {
                let structure = Structure::new(
                    Str::from(construct.get_target().brand().unwrap().to_string()),
                    construct.get_fields().iter().map(|field| Box::new(self.infer_element(field))).collect::<Vec<_>>(),
                );

                Type::new(TypeKind::Structure(structure))
            }
            ElementKind::Conditional(_) => {
                Type::unit()
            }
            ElementKind::While(_) => {
                Type::unit()
            }
            ElementKind::Cycle(_) => {
                Type::unit()
            }
            ElementKind::Symbolize(_) => {
                Type::unit()
            }
            ElementKind::Assign(_) => {
                Type::unit()
            }
            ElementKind::Return(_) => {
                Type::unit()
            }
            ElementKind::Break(_) => {
                Type::unit()
            }
            ElementKind::Continue(_) => {
                Type::unit()
            }
        }
    }

    pub fn infer_symbol(&mut self, symbol: Symbol<'resolver>) -> Type<'resolver> {
        match &symbol.kind {
            Symbolic::Inclusion(_) => {
                Type::unit()
            }
            Symbolic::Extension(_) => {
                Type::unit()
            }
            Symbolic::Binding(binding) => {
                if let Some(annotation) = binding.get_annotation() {
                    if let Some(ty) = self.get(annotation) {
                        self.infer_symbol(ty)
                    } else {
                        Type::unit()
                    }
                } else if let Some(value) = binding.get_value() {
                    self.infer_element(value)
                } else {
                    Type::unit()
                }
            }
            Symbolic::Structure(structure) => {
                let structure = Structure::new(
                    Str::from(structure.get_target().brand().unwrap().to_string()),
                    structure.get_fields()
                        .iter()
                        .map(|field| {
                            Box::new(self.infer_symbol(field.clone()))
                        })
                        .collect::<Vec<_>>(),
                );

                Type::new(
                    TypeKind::Structure(
                        structure,
                    )
                )
            }
            Symbolic::Enumeration(_) => {
                Type::unit()
            }
            Symbolic::Method(_) => {
                Type::unit()
            }
            Symbolic::Module(_) => {
                Type::unit()
            }
            Symbolic::Preference(_) => {
                Type::unit()
            }
        }
    }

    pub fn infer_literal(&mut self, literal: &Token<'resolver>) -> Type<'resolver> {
        match literal.kind {
            TokenKind::Float(_) => {
                Type::new(TypeKind::Float { size: 64 })
            }
            TokenKind::Integer(_) => {
                Type::new(TypeKind::Integer { size: 64 })
            }
            TokenKind::Boolean(_) => {
                Type::new(TypeKind::Boolean)
            }
            TokenKind::String(_) => {
                Type::unit()
            }
            TokenKind::Character(_) => {
                Type::unit()
            }
            TokenKind::Operator(_) => {
                Type::unit()
            }
            TokenKind::Identifier(_) => {
                Type::unit()
            }
            TokenKind::Punctuation(_) => {
                Type::unit()
            }
            TokenKind::Comment(_) => {
                Type::unit()
            }
        }
    }
}
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
use crate::resolver::{ResolveError, Resolver};
use crate::scanner::{Token, TokenKind};
use crate::schema::Structure;

impl<'resolver> Resolver<'resolver> {
    pub fn check(&mut self, target: Type<'resolver>, source: Type<'resolver>) {
        if target != source {
            let error = ResolveError::new(
                crate::resolver::ErrorKind::Check { 
                    error: CheckError::new(
                        crate::resolver::checker::ErrorKind::Mismatch(
                            target, source.clone()
                        ),
                        source.span
                    ),
                },
                source.span
            );
            
            self.errors.push(error);
        }    
    }
    
    pub fn infer_element(&mut self, element: &Element<'resolver>) -> Type<'resolver> {
        match &element.kind {
            ElementKind::Literal(literal) => {
                self.infer_literal(literal)
            }
            ElementKind::Procedural(_) => {
                Type::unit(element.span)
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
                    },
                    element.span,
                )
            }
            ElementKind::Sequence(_) => {
                Type::unit(element.span)
            }
            ElementKind::Collection(_) => {
                Type::unit(element.span)
            }
            ElementKind::Series(_) => {
                Type::unit(element.span)
            }
            ElementKind::Bundle(_) => {
                Type::unit(element.span)
            }
            ElementKind::Block(block) => {
                if let Some(element) = block.items.last() {
                    self.infer_element(element)
                } else {
                    Type::unit(element.span)
                }
            }
            ElementKind::Unary(unary) => {
                self.infer_element(&*unary.operand)
            }
            ElementKind::Binary(binary) => {
                self.infer_element(&*binary.left)
            }
            ElementKind::Label(_) => {
                Type::unit(element.span)
            }
            ElementKind::Access(_) => {
                Type::unit(element.span)
            }
            ElementKind::Index(_) => {
                Type::unit(element.span)
            }
            ElementKind::Invoke(_) => {
                Type::unit(element.span)
            }
            ElementKind::Construct(construct) => {
                let structure = Structure::new(
                    Str::from(construct.target.brand().unwrap().to_string()),
                    construct.fields.iter().map(|field| Box::new(self.infer_element(field))).collect::<Vec<_>>(),
                );

                Type::new(TypeKind::Structure(structure), element.span)
            }
            ElementKind::Conditional(_) => {
                Type::unit(element.span)
            }
            ElementKind::While(_) => {
                Type::unit(element.span)
            }
            ElementKind::Cycle(_) => {
                Type::unit(element.span)
            }
            ElementKind::Symbolize(_) => {
                Type::unit(element.span)
            }
            ElementKind::Assign(_) => {
                Type::unit(element.span)
            }
            ElementKind::Return(_) => {
                Type::unit(element.span)
            }
            ElementKind::Break(_) => {
                Type::unit(element.span)
            }
            ElementKind::Continue(_) => {
                Type::unit(element.span)
            }
        }
    }

    pub fn infer_symbol(&mut self, symbol: Symbol<'resolver>) -> Type<'resolver> {
        match &symbol.kind {
            Symbolic::Inclusion(_) => {
                Type::unit(symbol.span)
            }
            Symbolic::Extension(_) => {
                Type::unit(symbol.span)
            }
            Symbolic::Binding(binding) => {
                if let Some(annotation) = &binding.annotation {
                    if let Some(ty) = self.get(annotation) {
                        self.infer_symbol(ty)
                    } else {
                        Type::unit(symbol.span)
                    }
                } else if let Some(value) = &binding.value {
                    self.infer_element(value)
                } else {
                    Type::unit(symbol.span)
                }
            }
            Symbolic::Structure(structure) => {
                let structure = Structure::new(
                    Str::from(structure.target.brand().unwrap().to_string()),
                    structure.fields
                        .iter()
                        .map(|field| {
                            Box::new(self.infer_symbol(field.clone()))
                        })
                        .collect::<Vec<_>>(),
                );

                Type::new(
                    TypeKind::Structure(
                        structure,
                    ),
                    symbol.span
                )
            }
            Symbolic::Enumeration(_) => {
                Type::unit(symbol.span)
            }
            Symbolic::Method(_) => {
                Type::unit(symbol.span)
            }
            Symbolic::Module(_) => {
                Type::unit(symbol.span)
            }
            Symbolic::Preference(_) => {
                Type::unit(symbol.span)
            }
        }
    }

    pub fn infer_literal(&mut self, literal: &Token<'resolver>) -> Type<'resolver> {
        match literal.kind {
            TokenKind::Float(_) => {
                Type::new(TypeKind::Float { size: 64 }, literal.span)
            }
            TokenKind::Integer(_) => {
                Type::new(TypeKind::Integer { size: 64 }, literal.span)
            }
            TokenKind::Boolean(_) => {
                Type::new(TypeKind::Boolean, literal.span)
            }
            TokenKind::String(_) => {
                Type::unit(literal.span)
            }
            TokenKind::Character(_) => {
                Type::unit(literal.span)
            }
            TokenKind::Operator(_) => {
                Type::unit(literal.span)
            }
            TokenKind::Identifier(_) => {
                Type::unit(literal.span)
            }
            TokenKind::Punctuation(_) => {
                Type::unit(literal.span)
            }
            TokenKind::Comment(_) => {
                Type::unit(literal.span)
            }
        }
    }
}
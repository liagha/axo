use crate::{
    data::Str,
    internal::hash::Map,
    parser::{
        Element, ElementKind, Symbol,
    },
};
use crate::resolver::checker::types::TypeKind;
use crate::parser::SymbolKind;
use crate::resolver::checker::{
    types::Type,
    CheckError,
};
use crate::resolver::{ResolveError, Resolver};
use crate::scanner::{PunctuationKind, Token, TokenKind};
use crate::schema::{Index, Invoke, Structure};

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
        match element.kind.clone() {
            ElementKind::Literal(literal) => {
                Type::unit(literal.span)
            }
            ElementKind::Procedural(_) => {
                Type::unit(element.span)
            }
            ElementKind::Delimited(delimited) => {
                match (delimited.start.kind, delimited.separator.map(|token| token.kind), delimited.end.kind) {
                    (
                        TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                        None,
                        TokenKind::Punctuation(PunctuationKind::RightParenthesis),
                    ) | (
                        TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                        Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                        TokenKind::Punctuation(PunctuationKind::RightParenthesis),
                    ) => {
                        Type::new(
                            TypeKind::Tuple {
                                items: delimited.items
                                    .iter()
                                    .map(|item| {
                                        self.infer_element(item)
                                    })
                                    .collect::<Vec<_>>()
                            },
                            element.span,
                        )
                    }

                    (
                        TokenKind::Punctuation(PunctuationKind::LeftBrace),
                        None,
                        TokenKind::Punctuation(PunctuationKind::RightBrace),
                    ) | (
                        TokenKind::Punctuation(PunctuationKind::LeftBrace),
                        Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                        TokenKind::Punctuation(PunctuationKind::RightBrace),
                    ) => {
                        if let Some(element) = delimited.items.last() {
                            self.infer_element(element)
                        } else {
                            Type::unit(element.span)
                        }
                    }

                    _ => {
                        Type::unit(element.span)
                    }
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
                    construct.members.iter().map(|field| Box::new(self.infer_element(field))).collect::<Vec<_>>(),
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
            SymbolKind::Inclusion(_) => {
                Type::unit(symbol.span)
            }
            SymbolKind::Extension(_) => {
                Type::unit(symbol.span)
            }
            SymbolKind::Binding(binding) => {
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
            SymbolKind::Structure(structure) => {
                let structure = Structure::new(
                    Str::from(structure.target.brand().unwrap().to_string()),
                    structure.members
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
            SymbolKind::Enumeration(_) => {
                Type::unit(symbol.span)
            }
            SymbolKind::Method(_) => {
                Type::unit(symbol.span)
            }
            SymbolKind::Module(_) => {
                Type::unit(symbol.span)
            }
            SymbolKind::Preference(_) => {
                Type::unit(symbol.span)
            }
        }
    }
}
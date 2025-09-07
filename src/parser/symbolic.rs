use {
    super::{
        Element, ElementKind,
        Symbol,
    },
    crate::{
        scanner::{
            Token, TokenKind
        },
        format::{
            Debug, Formatter, Result as FormatResult,
        },
        schema::{
            Binding, Enumeration, Extension, Inclusion, Method, Structure, Module,
        },
        initial::{
            Preference,
        },
        internal::{
            hash::{Hash, Hasher},
        },
        data::{
            any::{Any, TypeId},
            memory,
        },
    }
};
use crate::scanner::OperatorKind;

#[derive(Clone, PartialEq, Hash)]
pub enum SymbolKind<'symbol> {
    Inclusion(Inclusion<Box<Element<'symbol>>>),
    Extension(Extension<Box<Element<'symbol>>, Box<Element<'symbol>>, Symbol<'symbol>>),
    Binding(Binding<Box<Element<'symbol>>, Box<Element<'symbol>>, Box<Element<'symbol>>>),
    Structure(Structure<Box<Element<'symbol>>, Symbol<'symbol>>),
    Enumeration(Enumeration<Box<Element<'symbol>>, Symbol<'symbol>>),
    Method(Method<Box<Element<'symbol>>, Symbol<'symbol>, Box<Element<'symbol>>, Option<Box<Element<'symbol>>>>),
    Module(Module<Box<Element<'symbol>>>),
    Preference(Preference<'symbol>),
}

impl<'symbol> SymbolKind<'symbol> {
    pub fn brand(&self) -> Option<Token<'symbol>> {
        match self {
            SymbolKind::Inclusion(inclusion) => inclusion.target.clone().brand(),
            SymbolKind::Extension(extension) => extension.target.clone().brand(),
            SymbolKind::Binding(binding) => binding.target.clone().brand(),
            SymbolKind::Structure(structure) => structure.target.clone().brand(),
            SymbolKind::Enumeration(enumeration) => enumeration.target.clone().brand(),
            SymbolKind::Method(method) => method.target.clone().brand(),
            SymbolKind::Module(module) => module.target.brand().clone(),
            SymbolKind::Preference(preference) => Some(preference.target.clone()),
        }
    }
}

impl<'symbol> Element<'symbol> {
    pub fn brand(&self) -> Option<Token<'symbol>> {
        match &self.kind {
            ElementKind::Literal(literal) => Some(literal.clone()),
            ElementKind::Construct(construct) => construct.target.brand(),
            ElementKind::Index(index) => index.target.brand(),
            ElementKind::Invoke(invoke) => invoke.target.brand(),
            ElementKind::Symbolize(symbol) => symbol.brand(),
            ElementKind::Binary(binary) => {
                match binary.operator.kind {
                    TokenKind::Operator(OperatorKind::Colon) => binary.left.brand().clone(),
                    TokenKind::Operator(OperatorKind::Equal) => binary.left.brand().clone(),
                    TokenKind::Operator(OperatorKind::Dot) => binary.right.brand().clone(),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}
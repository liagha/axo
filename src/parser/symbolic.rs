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

#[derive(Clone, PartialEq, Hash)]
pub enum Symbolic<'symbol> {
    Inclusion(Inclusion<Box<Element<'symbol>>>),
    Extension(Extension<Box<Element<'symbol>>, Box<Element<'symbol>>, Symbol<'symbol>>),
    Binding(Binding<Box<Element<'symbol>>, Box<Element<'symbol>>, Box<Element<'symbol>>>),
    Structure(Structure<Box<Element<'symbol>>, Symbol<'symbol>>),
    Enumeration(Enumeration<Box<Element<'symbol>>, Symbol<'symbol>>),
    Method(Method<Box<Element<'symbol>>, Symbol<'symbol>, Box<Element<'symbol>>, Option<Box<Element<'symbol>>>>),
    Module(Module<Box<Element<'symbol>>>),
    Preference(Preference<'symbol>),
}

impl<'symbol> Symbolic<'symbol> {
    pub fn brand(&self) -> Option<Token<'symbol>> {
        match self {
            Symbolic::Inclusion(inclusion) => inclusion.target.clone().brand(),
            Symbolic::Extension(extension) => extension.target.clone().brand(),
            Symbolic::Binding(binding) => binding.target.clone().brand(),
            Symbolic::Structure(structure) => structure.target.clone().brand(),
            Symbolic::Enumeration(enumeration) => enumeration.target.clone().brand(),
            Symbolic::Method(method) => method.target.clone().brand(),
            Symbolic::Module(module) => module.target.brand().clone(),
            Symbolic::Preference(preference) => Some(preference.target.clone()),
        }
    }
}

impl<'symbol> Element<'symbol> {
    pub fn brand(&self) -> Option<Token<'symbol>> {
        match &self.kind {
            ElementKind::Literal(literal) => Some(literal.clone()),
            ElementKind::Construct(construct) => construct.target.brand(),
            ElementKind::Label(label) => label.label.brand(),
            ElementKind::Index(index) => index.target.brand(),
            ElementKind::Invoke(invoke) => invoke.target.brand(),
            ElementKind::Access(access) => access.member.brand(),
            ElementKind::Symbolize(symbol) => symbol.brand(),
            ElementKind::Assign(assign) => assign.target.brand(),
            _ => None,
        }
    }
}
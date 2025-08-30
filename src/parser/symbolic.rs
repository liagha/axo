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

#[derive(Clone, Debug, PartialEq, Hash)]
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
            Symbolic::Inclusion(inclusion) => inclusion.get_target().clone().brand(),
            Symbolic::Extension(extension) => extension.get_target().clone().brand(),
            Symbolic::Binding(binding) => binding.get_target().clone().brand(),
            Symbolic::Structure(structure) => structure.get_target().clone().brand(),
            Symbolic::Enumeration(enumeration) => enumeration.get_target().clone().brand(),
            Symbolic::Method(method) => method.get_target().clone().brand(),
            Symbolic::Module(module) => module.get_target().brand().clone(),
            Symbolic::Preference(preference) => Some(preference.target.clone()),
        }
    }
}

impl<'symbol> Element<'symbol> {
    pub fn brand(&self) -> Option<Token<'symbol>> {
        match &self.kind {
            ElementKind::Literal(literal) => Some(literal.clone()),
            ElementKind::Construct(construct) => construct.get_target().brand(),
            ElementKind::Label(label) => label.get_label().brand(),
            ElementKind::Index(index) => index.get_target().brand(),
            ElementKind::Invoke(invoke) => invoke.get_target().brand(),
            ElementKind::Access(access) => access.get_member().brand(),
            ElementKind::Symbolize(symbol) => symbol.brand(),
            ElementKind::Assign(assign) => assign.get_target().brand(),
            _ => None,
        }
    }
}
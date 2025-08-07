use crate::format::Debug;
use crate::hash::{Hash, Hasher};
use std::any::Any;
use std::collections::hash_map::DefaultHasher;
use {
    crate::{
        axo_scanner::{
            Token, TokenKind
        },
        axo_parser::{
            Element, ElementKind,
        }
    }
};

use crate::axo_parser::Symbol;
use crate::axo_schema::{Binding, Enumeration, Implementation, Inclusion, Interface, Method, Structure};

pub trait Symbolic<'input>: Debug + 'input {
    fn brand(&self) -> Option<Token<'input>>;
    
    fn as_any(&self) -> &dyn Any where Self: 'static;
    
    fn dyn_clone(&self) -> Box<dyn Symbolic<'input> + 'input>;
    
    fn dyn_eq(&self, other: &dyn Symbolic<'input>) -> bool;
    
    fn dyn_hash(&self, state: &mut dyn Hasher);
}

impl<'input> Clone for Box<dyn Symbolic<'input>> {
    fn clone(&self) -> Self {
        (**self).dyn_clone()
    }
}

impl<'input> Clone for Box<dyn Symbolic<'input> + Send> {
    fn clone(&self) -> Self {
        let cloned: Box<dyn Symbolic<'input>> = (**self).dyn_clone();
        unsafe { std::mem::transmute(cloned) }
    }
}

impl<'input> Clone for Box<dyn Symbolic<'input> + Sync> {
    fn clone(&self) -> Self {
        let cloned: Box<dyn Symbolic<'input>> = (**self).dyn_clone();
        unsafe { std::mem::transmute(cloned) }
    }
}

impl<'input> Clone for Box<dyn Symbolic<'input> + Send + Sync> {
    fn clone(&self) -> Self {
        let cloned: Box<dyn Symbolic<'input>> = (**self).dyn_clone();
        unsafe { std::mem::transmute(cloned) }
    }
}

impl<'input> PartialEq for dyn Symbolic<'input> + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.dyn_eq(other)
    }
}

impl<'input> PartialEq for dyn Symbolic<'input> + Send + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.dyn_eq(other)
    }
}

impl<'input> PartialEq for dyn Symbolic<'input> + Sync + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.dyn_eq(other)
    }
}

impl<'input> PartialEq for dyn Symbolic<'input> + Send + Sync + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.dyn_eq(other)
    }
}

impl<'input> Eq for dyn Symbolic<'input> + '_ {}
impl<'input> Eq for dyn Symbolic<'input> + Send + '_ {}
impl<'input> Eq for dyn Symbolic<'input> + Sync + '_ {}
impl<'input> Eq for dyn Symbolic<'input> + Send + Sync + '_ {}

impl<'input> Hash for dyn Symbolic<'input> + '_ {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dyn_hash(state);
    }
}

impl<'input> Hash for dyn Symbolic<'input> + Send + '_ {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dyn_hash(state);
    }
}

impl<'input> Hash for dyn Symbolic<'input> + Sync + '_ {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dyn_hash(state);
    }
}

impl<'input> Hash for dyn Symbolic<'input> + Send + Sync + '_ {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dyn_hash(state);
    }
}

// Symbol implementation
impl<'symbol> Symbolic<'symbol> for Symbol<'symbol> {
    fn brand(&self) -> Option<Token<'symbol>> {
        self.value.brand()
    }
    
    fn as_any(&self) -> &dyn Any where Self: 'static {
        self
    }
    
    fn dyn_clone(&self) -> Box<dyn Symbolic<'symbol>> {
        Box::new(Self {
            value: self.value.clone(),
            span: self.span.clone(),
            members: self.members.clone(),
        })
    }
    
    fn dyn_eq(&self, other: &dyn Symbolic<'symbol>) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.value == other.value.clone()
        } else {
            false
        }
    }
    
    fn dyn_hash(&self, state: &mut dyn Hasher) {
        let mut hasher = DefaultHasher::new();
        std::hash::Hash::hash(&std::any::TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());
        
        let mut hasher = DefaultHasher::new();
        self.value.dyn_hash(&mut hasher);
        state.write_u64(hasher.finish());
    }
}

// Inclusion implementation
impl<'symbol> Symbolic<'symbol> for Inclusion<Box<Element<'symbol>>> {
    fn brand(&self) -> Option<Token<'symbol>> {
        self.get_target().clone().brand()
    }
    
    fn as_any(&self) -> &dyn Any where Self: 'static {
        self
    }
    
    fn dyn_clone(&self) -> Box<dyn Symbolic<'symbol>> {
        Box::new(self.clone())
    }
    
    fn dyn_eq(&self, other: &dyn Symbolic<'symbol>) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }
    
    fn dyn_hash(&self, state: &mut dyn Hasher) {
        let mut hasher = DefaultHasher::new();
        std::hash::Hash::hash(&std::any::TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());
        
        let mut hasher = DefaultHasher::new();
        std::hash::Hash::hash(&self, &mut hasher);
        state.write_u64(hasher.finish());
    }
}

// Implementation implementation
impl<'symbol> Symbolic<'symbol> for Implementation<Box<Element<'symbol>>, Box<Element<'symbol>>, Symbol<'symbol>> {
    fn brand(&self) -> Option<Token<'symbol>> {
        self.get_target().clone().brand()
    }
    
    fn as_any(&self) -> &dyn Any where Self: 'static {
        self
    }
    
    fn dyn_clone(&self) -> Box<dyn Symbolic<'symbol>> {
        Box::new(self.clone())
    }
    
    fn dyn_eq(&self, other: &dyn Symbolic<'symbol>) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }
    
    fn dyn_hash(&self, state: &mut dyn Hasher) {
        let mut hasher = DefaultHasher::new();
        std::hash::Hash::hash(&std::any::TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());
        
        let mut hasher = DefaultHasher::new();
        std::hash::Hash::hash(&self, &mut hasher);
        state.write_u64(hasher.finish());
    }
}

// Interface implementation
impl<'symbol> Symbolic<'symbol> for Interface<Box<Element<'symbol>>, Symbol<'symbol>> {
    fn brand(&self) -> Option<Token<'symbol>> {
        self.get_target().clone().brand()
    }
    
    fn as_any(&self) -> &dyn Any where Self: 'static {
        self
    }
    
    fn dyn_clone(&self) -> Box<dyn Symbolic<'symbol>> {
        Box::new(self.clone())
    }
    
    fn dyn_eq(&self, other: &dyn Symbolic<'symbol>) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }
    
    fn dyn_hash(&self, state: &mut dyn Hasher) {
        let mut hasher = DefaultHasher::new();
        std::hash::Hash::hash(&std::any::TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());
        
        let mut hasher = DefaultHasher::new();
        std::hash::Hash::hash(&self, &mut hasher);
        state.write_u64(hasher.finish());
    }
}

// Binding implementation
impl<'symbol> Symbolic<'symbol> for Binding<Box<Element<'symbol>>, Box<Element<'symbol>>, Box<Element<'symbol>>> {
    fn brand(&self) -> Option<Token<'symbol>> {
        self.get_target().clone().brand()
    }
    
    fn as_any(&self) -> &dyn Any where Self: 'static {
        self
    }
    
    fn dyn_clone(&self) -> Box<dyn Symbolic<'symbol>> {
        Box::new(self.clone())
    }
    
    fn dyn_eq(&self, other: &dyn Symbolic<'symbol>) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }
    
    fn dyn_hash(&self, state: &mut dyn Hasher) {
        let mut hasher = DefaultHasher::new();
        std::hash::Hash::hash(&std::any::TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());
        
        let mut hasher = DefaultHasher::new();
        std::hash::Hash::hash(&self, &mut hasher);
        state.write_u64(hasher.finish());
    }
}

// Structure implementation
impl<'symbol> Symbolic<'symbol> for Structure<Box<Element<'symbol>>, Symbol<'symbol>> {
    fn brand(&self) -> Option<Token<'symbol>> {
        self.get_target().clone().brand()
    }
    
    fn as_any(&self) -> &dyn Any where Self: 'static {
        self
    }
    
    fn dyn_clone(&self) -> Box<dyn Symbolic<'symbol>> {
        Box::new(self.clone())
    }
    
    fn dyn_eq(&self, other: &dyn Symbolic<'symbol>) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }
    
    fn dyn_hash(&self, state: &mut dyn Hasher) {
        let mut hasher = DefaultHasher::new();
        std::hash::Hash::hash(&std::any::TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());
        
        let mut hasher = DefaultHasher::new();
        std::hash::Hash::hash(&self, &mut hasher);
        state.write_u64(hasher.finish());
    }
}

// Enumeration implementation
impl<'symbol> Symbolic<'symbol> for Enumeration<Box<Element<'symbol>>, Element<'symbol>> {
    fn brand(&self) -> Option<Token<'symbol>> {
        self.get_target().clone().brand()
    }
    
    fn as_any(&self) -> &dyn Any where Self: 'static {
        self
    }
    
    fn dyn_clone(&self) -> Box<dyn Symbolic<'symbol>> {
        Box::new(self.clone())
    }
    
    fn dyn_eq(&self, other: &dyn Symbolic<'symbol>) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }
    
    fn dyn_hash(&self, state: &mut dyn Hasher) {
        let mut hasher = DefaultHasher::new();
        std::hash::Hash::hash(&std::any::TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());
        
        let mut hasher = DefaultHasher::new();
        std::hash::Hash::hash(&self, &mut hasher);
        state.write_u64(hasher.finish());
    }
}

// Method implementation
impl<'symbol> Symbolic<'symbol> for Method<Box<Element<'symbol>>, Symbol<'symbol>, Box<Element<'symbol>>, Option<Box<Element<'symbol>>>> {
    fn brand(&self) -> Option<Token<'symbol>> {
        self.get_target().clone().brand()
    }
    
    fn as_any(&self) -> &dyn Any where Self: 'static {
        self
    }
    
    fn dyn_clone(&self) -> Box<dyn Symbolic<'symbol>> {
        Box::new(self.clone())
    }
    
    fn dyn_eq(&self, other: &dyn Symbolic<'symbol>) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }
    
    fn dyn_hash(&self, state: &mut dyn Hasher) {
        let mut hasher = DefaultHasher::new();
        std::hash::Hash::hash(&std::any::TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());
        
        let mut hasher = DefaultHasher::new();
        std::hash::Hash::hash(&self, &mut hasher);
        state.write_u64(hasher.finish());
    }
}

// Element implementation
impl<'symbol> Symbolic<'symbol> for Element<'symbol> {
    fn brand(&self) -> Option<Token<'symbol>> {
        match &self.kind {
            ElementKind::Literal(literal) => Some(Token {
                kind: literal.clone(),
                span: self.span,
            }),
            ElementKind::Identifier(identifier) => Some(Token {
                kind: TokenKind::Identifier(identifier.clone()),
                span: self.span,
            }),
            ElementKind::Construct(construct) => construct.get_target().brand(),
            ElementKind::Label(label) => label.get_label().brand(),
            ElementKind::Index(index) => index.get_target().brand(),
            ElementKind::Invoke(invoke) => invoke.get_target().brand(),
            ElementKind::Access(access) => access.get_object().brand(),
            ElementKind::Symbolize(symbol) => symbol.brand(),
            ElementKind::Assign(assign) => assign.get_target().brand(),
            _ => None,
        }
    }
    
    fn as_any(&self) -> &dyn Any where Self: 'static {
        self
    }
    
    fn dyn_clone(&self) -> Box<dyn Symbolic<'symbol>> {
        Box::new(self.clone())
    }
    
    fn dyn_eq(&self, other: &dyn Symbolic<'symbol>) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }
    
    fn dyn_hash(&self, state: &mut dyn Hasher) {
        let mut hasher = DefaultHasher::new();
        std::hash::Hash::hash(&std::any::TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());
        
        let mut hasher = DefaultHasher::new();
        std::hash::Hash::hash(&self, &mut hasher);
        state.write_u64(hasher.finish());
    }
}
use crate::any::Any;
use crate::hash::{Hash, Hasher};
use crate::{
    axo_scanner::{Token, TokenKind},
    axo_parser::{Element, ElementKind, Symbol},
    axo_initial::initializer::Preference,
    axo_schema::{Binding, Enumeration, Implementation, Inclusion, Interface, Method, Structure},
};

pub trait Symbolic<'input>: Any + std::fmt::Debug + Send + Sync + 'input {
    fn brand<'brand>(&self) -> Option<Token<'brand>>;
    fn as_any(&self) -> &dyn Any;
    fn dyn_eq(&self, other: &dyn Any) -> bool;
    fn dyn_hash(&self, state: &mut dyn Hasher);
    fn dyn_clone(&self) -> Box<dyn Symbolic<'input>>;
}

// Trait object implementations
impl<'input> Clone for Box<dyn Symbolic<'input>> {
    fn clone(&self) -> Self {
        self.dyn_clone()
    }
}

impl<'input> PartialEq for dyn Symbolic<'input> {
    fn eq(&self, other: &Self) -> bool {
        self.dyn_eq(other.as_any())
    }
}

impl<'input> Eq for dyn Symbolic<'input> {}

impl<'input> Hash for dyn Symbolic<'input> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dyn_hash(state);
    }
}

// Implementation for Symbol
impl<'input> Symbolic<'input> for Symbol<'input> {
    fn brand<'brand>(&self) -> Option<Token<'brand>> {
        self.value.brand()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_eq(&self, other: &dyn Any) -> bool {
        if let Some(other) = other.downcast_ref::<Symbol<'input>>() {
            self == other
        } else {
            false
        }
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        Hash::hash(self, state);
    }

    fn dyn_clone(&self) -> Box<dyn Symbolic<'input>> {
        Box::new(self.clone())
    }
}

// Implementation for Preference
impl<'input> Symbolic<'input> for Preference<'input> {
    fn brand<'brand>(&self) -> Option<Token<'brand>> {
        Some(Token::new(self.target.kind.clone(), self.target.span.clone()))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_eq(&self, other: &dyn Any) -> bool {
        if let Some(other) = other.downcast_ref::<Preference<'input>>() {
            self == other
        } else {
            false
        }
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        Hash::hash(self, state);
    }

    fn dyn_clone(&self) -> Box<dyn Symbolic<'input>> {
        Box::new(self.clone())
    }
}

// Implementation for Inclusion
impl<'input> Symbolic<'input> for Inclusion<Box<Element<'input>>> {
    fn brand<'brand>(&self) -> Option<Token<'brand>> {
        self.get_target().brand()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_eq(&self, other: &dyn Any) -> bool {
        if let Some(other) = other.downcast_ref::<Inclusion<Box<Element<'input>>>>() {
            self == other
        } else {
            false
        }
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        Hash::hash(self, state);
    }

    fn dyn_clone(&self) -> Box<dyn Symbolic<'input>> {
        Box::new(self.clone())
    }
}

// Implementation for Implementation
impl<'input> Symbolic<'input> for Implementation<Box<Element<'input>>, Box<Element<'input>>, Symbol<'input>> {
    fn brand<'brand>(&self) -> Option<Token<'brand>> {
        self.get_target().brand()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_eq(&self, other: &dyn Any) -> bool {
        if let Some(other) = other.downcast_ref::<Implementation<Box<Element<'input>>, Box<Element<'input>>, Symbol<'input>>>() {
            self == other
        } else {
            false
        }
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        Hash::hash(self, state);
    }

    fn dyn_clone(&self) -> Box<dyn Symbolic<'input>> {
        Box::new(self.clone())
    }
}

// Implementation for Interface
impl<'input> Symbolic<'input> for Interface<Box<Element<'input>>, Symbol<'input>> {
    fn brand<'brand>(&self) -> Option<Token<'brand>> {
        self.get_target().brand()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_eq(&self, other: &dyn Any) -> bool {
        if let Some(other) = other.downcast_ref::<Interface<Box<Element<'input>>, Symbol<'input>>>() {
            self == other
        } else {
            false
        }
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        Hash::hash(self, state);
    }

    fn dyn_clone(&self) -> Box<dyn Symbolic<'input>> {
        Box::new(self.clone())
    }
}

// Implementation for Binding
impl<'input> Symbolic<'input> for Binding<Box<Element<'input>>, Box<Element<'input>>, Box<Element<'input>>> {
    fn brand<'brand>(&self) -> Option<Token<'brand>> {
        self.get_target().brand()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_eq(&self, other: &dyn Any) -> bool {
        if let Some(other) = other.downcast_ref::<Binding<Box<Element<'input>>, Box<Element<'input>>, Box<Element<'input>>>>() {
            self == other
        } else {
            false
        }
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        Hash::hash(self, state);
    }

    fn dyn_clone(&self) -> Box<dyn Symbolic<'input>> {
        Box::new(self.clone())
    }
}

// Implementation for Structure
impl<'input> Symbolic<'input> for Structure<Box<Element<'input>>, Symbol<'input>> {
    fn brand<'brand>(&self) -> Option<Token<'brand>> {
        self.get_target().brand()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_eq(&self, other: &dyn Any) -> bool {
        if let Some(other) = other.downcast_ref::<Structure<Box<Element<'input>>, Symbol<'input>>>() {
            self == other
        } else {
            false
        }
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        Hash::hash(self, state);
    }

    fn dyn_clone(&self) -> Box<dyn Symbolic<'input>> {
        Box::new(self.clone())
    }
}

// Implementation for Enumeration
impl<'input> Symbolic<'input> for Enumeration<Box<Element<'input>>, Element<'input>> {
    fn brand<'brand>(&self) -> Option<Token<'brand>> {
        self.get_target().brand()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_eq(&self, other: &dyn Any) -> bool {
        if let Some(other) = other.downcast_ref::<Enumeration<Box<Element<'input>>, Element<'input>>>() {
            self == other
        } else {
            false
        }
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        Hash::hash(self, state);
    }

    fn dyn_clone(&self) -> Box<dyn Symbolic<'input>> {
        Box::new(self.clone())
    }
}

// Implementation for Method
impl<'input> Symbolic<'input> for Method<Box<Element<'input>>, Symbol<'input>, Box<Element<'input>>, Option<Box<Element<'input>>>> {
    fn brand<'brand>(&self) -> Option<Token<'brand>> {
        self.get_target().brand()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_eq(&self, other: &dyn Any) -> bool {
        if let Some(other) = other.downcast_ref::<Method<Box<Element<'input>>, Symbol<'input>, Box<Element<'input>>, Option<Box<Element<'input>>>>>() {
            self == other
        } else {
            false
        }
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        Hash::hash(self, state);
    }

    fn dyn_clone(&self) -> Box<dyn Symbolic<'input>> {
        Box::new(self.clone())
    }
}

// Implementation for Element
impl<'input> Symbolic<'input> for Element<'input> {
    fn brand<'brand>(&self) -> Option<Token<'brand>> {
        match &self.kind {
            ElementKind::Literal(literal) => Some(Token {
                kind: literal.clone(),
                span: self.span.clone(),
            }),
            ElementKind::Identifier(identifier) => Some(Token {
                kind: TokenKind::Identifier(identifier.clone()),
                span: self.span.clone(),
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

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_eq(&self, other: &dyn Any) -> bool {
        if let Some(other) = other.downcast_ref::<Element<'input>>() {
            self == other
        } else {
            false
        }
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        Hash::hash(self, state);
    }

    fn dyn_clone(&self) -> Box<dyn Symbolic<'input>> {
        Box::new(self.clone())
    }
}
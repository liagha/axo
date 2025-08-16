use {
    crate::{
        format::{
            self, Show,
            Debug, Display, Formatter,
        },
        data::{
            any::{Any, TypeId},
        },
        internal::{
            hash::{DefaultHasher, Hash, Hasher},
        },
        parser::Symbol,
        resolver::scope::Scope,
        scanner::Token,
        tracker::Span,
    }
};

pub struct Type {
    pub value: Box<dyn Typable>,
    pub span: Span<'static>,
}

impl Type {
    pub fn new(value: impl Typable + 'static, span: Span<'static>) -> Self {
        Self {
            value: Box::new(value),
            span,
        }
    }

    pub fn cast<Type: 'static>(&self) -> Option<&Type> {
        self.value.as_ref().as_any().downcast_ref::<Type>()
    }
}

impl Clone for Type {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            span: self.span.clone(),
        }
    }
}

impl Debug for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> format::Result {
        write!(f, "{:?}", self.value)
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Eq for Type {}

impl Hash for Type {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value.clone()
    }
}

pub trait Typable: Debug + 'static {
    fn brand(&self) -> Option<Token<'static>>;

    fn as_any(&self) -> &dyn Any where Self: 'static;

    fn dyn_clone(&self) -> Box<dyn Typable>;

    fn dyn_eq(&self, other: &dyn Typable) -> bool;

    fn dyn_hash(&self, state: &mut dyn Hasher);
}

impl Clone for Box<dyn Typable> {
    fn clone(&self) -> Self {
        (**self).dyn_clone()
    }
}

impl PartialEq for dyn Typable + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.dyn_eq(other)
    }
}

impl Eq for dyn Typable + '_ {}

impl Hash for dyn Typable + '_ {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dyn_hash(state);
    }
}

impl Typable for Type {
    fn brand(&self) -> Option<Token<'static>> {
        self.value.brand()
    }

    fn as_any(&self) -> &dyn Any where Self: 'static {
        self
    }

    fn dyn_clone(&self) -> Box<dyn Typable> {
        Box::new(Self {
            value: self.value.clone(),
            span: self.span.clone(),
        })
    }

    fn dyn_eq(&self, other: &dyn Typable) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.value == other.value.clone()
        } else {
            false
        }
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        let mut hasher = DefaultHasher::new();
        Hash::hash(&TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());

        let mut hasher = DefaultHasher::new();
        self.value.dyn_hash(&mut hasher);
        state.write_u64(hasher.finish());
    }
}
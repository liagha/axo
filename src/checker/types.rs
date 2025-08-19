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

pub struct Type<'ty: 'static> {
    pub value: Box<dyn Typable<'ty>>,
    pub span: Span<'ty>,
}

impl<'ty> Type<'ty> {
    pub fn new(value: impl Typable<'ty> + 'ty, span: Span<'ty>) -> Self {
        Self {
            value: Box::new(value),
            span,
        }
    }

    pub fn cast<Type: 'ty>(&self) -> Option<&Type> {
        self.value.as_ref().as_any().downcast_ref::<Type>()
    }
}

impl<'ty> Clone for Type<'ty> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            span: self.span.clone(),
        }
    }
}

impl<'ty> Debug for Type<'ty> {
    fn fmt(&self, f: &mut Formatter<'_>) -> format::Result {
        write!(f, "{:?}", self.value)
    }
}

impl<'ty> Display for Type<'ty> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'ty> Eq for Type<'ty> {}

impl<'ty> Hash for Type<'ty> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<'ty> PartialEq for Type<'ty> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value.clone()
    }
}

pub trait Typable<'ty: 'static>: Debug + 'ty {
    fn brand(&self) -> Option<Token<'ty>>;

    fn as_any(&self) -> &dyn Any where Self: 'ty;

    fn dyn_clone(&self) -> Box<dyn Typable<'ty>>;

    fn dyn_eq(&self, other: &dyn Typable<'ty>) -> bool;

    fn dyn_hash(&self, state: &mut dyn Hasher);
}

impl<'ty: 'static> Clone for Box<dyn Typable<'ty>> {
    fn clone(&self) -> Self {
        (**self).dyn_clone()
    }
}

impl<'ty: 'static> PartialEq for dyn Typable<'ty> + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.dyn_eq(other)
    }
}

impl<'ty: 'static> Eq for dyn Typable<'ty> + '_ {}

impl<'ty: 'static> Hash for dyn Typable<'ty> + '_ {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dyn_hash(state);
    }
}

impl<'ty> Typable<'ty> for Type<'ty> {
    fn brand(&self) -> Option<Token<'ty>> {
        self.value.brand()
    }

    fn as_any(&self) -> &dyn Any where Self: 'ty {
        self
    }

    fn dyn_clone(&self) -> Box<dyn Typable<'ty>> {
        Box::new(Self {
            value: self.value.clone(),
            span: self.span.clone(),
        })
    }

    fn dyn_eq(&self, other: &dyn Typable<'ty>) -> bool {
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
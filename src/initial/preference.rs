use {
    crate::{
        data::{
            any::{Any, TypeId},
            memory,
        },
        internal::{
            hash::{Hash, Hasher, DefaultHasher},
        },
        parser::Symbolic,
        scanner::Token,
        tracker::{Span, Spanned},
    }
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Preference<'preference> {
    pub target: Token<'preference>,
    pub value: Token<'preference>,
    pub span: Span<'preference>,
}

impl<'preference> Spanned<'preference> for Preference<'preference> {
    fn borrow_span(&self) -> Span<'preference> {
        self.span.clone()
    }

    fn span(self) -> Span<'preference> {
        self.span
    }
}

impl<'preference> Preference<'preference> {
    pub fn new(target: Token<'preference>, value: Token<'preference>) -> Self {
        let span = Span::merge(&target.borrow_span(), &value.borrow_span());

        Self {
            target,
            value,
            span
        }
    }
}

impl Symbolic for Preference<'static> {
    fn brand(&self) -> Option<Token<'static>> {
        Some(unsafe { memory::transmute(self.target.clone()) })
    }

    fn as_any(&self) -> &dyn Any where Self: 'static {
        self
    }

    fn dyn_clone(&self) -> Box<dyn Symbolic> {
        Box::new(Self {
            target: self.target.clone(),
            value: self.value.clone(),
            span: self.span.clone(),
        })
    }

    fn dyn_eq(&self, other: &dyn Symbolic) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        let mut hasher = DefaultHasher::new();
        Hash::hash(&TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());

        let mut hasher = DefaultHasher::new();
        Hash::hash(&self, &mut hasher);
        state.write_u64(hasher.finish());
    }
}


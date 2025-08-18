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
            Debug,
        },
        schema::{
            Binding, Enumeration, Implementation, Inclusion, Interface, Method, Structure, Module,
        },
        initial::{
            Preference,
        },
        internal::{
            hash::{Hash, Hasher, DefaultHasher},
        },
        data::{
            any::{Any, TypeId},
            memory,
        },
    }
};

pub trait Symbolic<'symbol: 'static>: Debug + 'symbol {
    fn brand(&self) -> Option<Token<'symbol>>;

    fn as_any(&self) -> &dyn Any where Self: 'symbol;

    fn dyn_clone(&self) -> Box<dyn Symbolic<'symbol>>;

    fn dyn_eq(&self, other: &dyn Symbolic<'symbol>) -> bool;

    fn dyn_hash(&self, state: &mut dyn Hasher);
}

impl<'symbol: 'static> Clone for Box<dyn Symbolic<'symbol>> {
    fn clone(&self) -> Self {
        (**self).dyn_clone()
    }
}

impl<'symbol: 'static> PartialEq for dyn Symbolic<'symbol> + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.dyn_eq(other)
    }
}

impl<'symbol: 'static> Eq for dyn Symbolic<'symbol> + '_ {}

impl<'symbol: 'static> Hash for dyn Symbolic<'symbol> + '_ {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dyn_hash(state);
    }
}

impl<'symbol: 'static> Symbolic<'symbol> for Symbol<'symbol> {
    fn brand(&self) -> Option<Token<'symbol>> {
        self.value.brand()
    }

    fn as_any(&self) -> &dyn Any where Self: 'symbol {
        self
    }

    fn dyn_clone(&self) -> Box<dyn Symbolic<'symbol>> {
        Box::new(Self {
            value: self.value.clone(),
            span: self.span.clone(),
            scope: self.scope.clone(),
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
        Hash::hash(&TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());

        let mut hasher = DefaultHasher::new();
        self.value.dyn_hash(&mut hasher);
        state.write_u64(hasher.finish());
    }
}

impl<'symbol: 'static> Symbolic<'symbol> for Inclusion<Box<Element<'symbol>>> {
    fn brand(&self) -> Option<Token<'symbol>> {
        self.get_target().clone().brand()
    }

    fn as_any(&self) -> &dyn Any where Self: 'symbol {
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
        Hash::hash(&TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());

        let mut hasher = DefaultHasher::new();
        Hash::hash(&self, &mut hasher);
        state.write_u64(hasher.finish());
    }
}

impl<'symbol: 'static> Symbolic<'symbol> for Implementation<Box<Element<'symbol>>, Box<Element<'symbol>>, Symbol<'symbol>> {
    fn brand(&self) -> Option<Token<'symbol>> {
        self.get_target().clone().brand()
    }

    fn as_any(&self) -> &dyn Any where Self: 'symbol {
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
        Hash::hash(&TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());

        let mut hasher = DefaultHasher::new();
        Hash::hash(&self, &mut hasher);
        state.write_u64(hasher.finish());
    }
}

impl<'symbol: 'static> Symbolic<'symbol> for Interface<Box<Element<'symbol>>, Symbol<'symbol>> {
    fn brand(&self) -> Option<Token<'symbol>> {
        self.get_target().clone().brand()
    }

    fn as_any(&self) -> &dyn Any where Self: 'symbol {
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
        Hash::hash(&TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());

        let mut hasher = DefaultHasher::new();
        Hash::hash(&self, &mut hasher);
        state.write_u64(hasher.finish());
    }
}

impl<'symbol: 'static> Symbolic<'symbol> for Binding<Box<Element<'symbol>>, Box<Element<'symbol>>, Box<Element<'symbol>>> {
    fn brand(&self) -> Option<Token<'symbol>> {
        self.get_target().clone().brand()
    }

    fn as_any(&self) -> &dyn Any where Self: 'symbol {
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
        Hash::hash(&TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());

        let mut hasher = DefaultHasher::new();
        Hash::hash(&self, &mut hasher);
        state.write_u64(hasher.finish());
    }
}

impl<'symbol: 'static> Symbolic<'symbol> for Structure<Box<Element<'symbol>>, Symbol<'symbol>> {
    fn brand(&self) -> Option<Token<'symbol>> {
        self.get_target().clone().brand()
    }

    fn as_any(&self) -> &dyn Any where Self: 'symbol {
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
        Hash::hash(&TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());

        let mut hasher = DefaultHasher::new();
        Hash::hash(&self, &mut hasher);
        state.write_u64(hasher.finish());
    }
}

impl<'symbol: 'static> Symbolic<'symbol> for Enumeration<Box<Element<'symbol>>, Element<'symbol>> {
    fn brand(&self) -> Option<Token<'symbol>> {
        self.get_target().clone().brand()
    }

    fn as_any(&self) -> &dyn Any where Self: 'symbol {
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
        Hash::hash(&TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());

        let mut hasher = DefaultHasher::new();
        Hash::hash(&self, &mut hasher);
        state.write_u64(hasher.finish());
    }
}

impl<'symbol> Symbolic<'symbol> for Method<Box<Element<'symbol>>, Symbol<'symbol>, Box<Element<'symbol>>, Option<Box<Element<'symbol>>>> {
    fn brand(&self) -> Option<Token<'symbol>> {
        self.get_target().clone().brand()
    }

    fn as_any(&self) -> &dyn Any where Self: 'symbol {
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
        Hash::hash(&TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());

        let mut hasher = DefaultHasher::new();
        Hash::hash(&self, &mut hasher);
        state.write_u64(hasher.finish());
    }
}

impl<'symbol: 'static> Symbolic<'symbol> for Module<Element<'symbol>> {
    fn brand(&self) -> Option<Token<'symbol>> {
        self.get_target().brand().clone()
    }

    fn as_any(&self) -> &dyn Any where Self: 'symbol {
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
        Hash::hash(&TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());

        let mut hasher = DefaultHasher::new();
        Hash::hash(&self, &mut hasher);
        state.write_u64(hasher.finish());
    }
}

impl<'symbol: 'static> Symbolic<'symbol> for Element<'symbol> {
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
            ElementKind::Access(access) => access.get_member().brand(),
            ElementKind::Symbolize(symbol) => symbol.brand(),
            ElementKind::Assign(assign) => assign.get_target().brand(),
            _ => None,
        }
    }

    fn as_any(&self) -> &dyn Any where Self: 'symbol {
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
        Hash::hash(&TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());

        let mut hasher = DefaultHasher::new();
        Hash::hash(&self, &mut hasher);
        state.write_u64(hasher.finish());
    }
}

impl<'preference: 'static> Symbolic<'preference> for Preference<'preference> {
    fn brand(&self) -> Option<Token<'preference>> {
        Some(self.target.clone())
    }

    fn as_any(&self) -> &dyn Any where Self: 'preference {
        self
    }

    fn dyn_clone(&self) -> Box<dyn Symbolic<'preference>> {
        Box::new(Self {
            target: self.target.clone(),
            value: self.value.clone(),
            span: self.span.clone(),
        })
    }

    fn dyn_eq(&self, other: &dyn Symbolic<'preference>) -> bool {
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
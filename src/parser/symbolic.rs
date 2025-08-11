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
        internal::hash::{Hash, Hasher, DefaultHasher},
        data::{
            any::{Any, TypeId},
            memory,
        },
    }
};

pub trait Symbolic: Debug + 'static {
    fn brand(&self) -> Option<Token<'static>>;

    fn as_any(&self) -> &dyn Any where Self: 'static;

    fn dyn_clone(&self) -> Box<dyn Symbolic>;

    fn dyn_eq(&self, other: &dyn Symbolic) -> bool;

    fn dyn_hash(&self, state: &mut dyn Hasher);
}

impl Clone for Box<dyn Symbolic> {
    fn clone(&self) -> Self {
        (**self).dyn_clone()
    }
}

impl Clone for Box<dyn Symbolic + Send> {
    fn clone(&self) -> Self {
        let cloned: Box<dyn Symbolic> = (**self).dyn_clone();
        unsafe { memory::transmute(cloned) }
    }
}

impl Clone for Box<dyn Symbolic + Sync> {
    fn clone(&self) -> Self {
        let cloned: Box<dyn Symbolic> = (**self).dyn_clone();
        unsafe { memory::transmute(cloned) }
    }
}

impl Clone for Box<dyn Symbolic + Send + Sync> {
    fn clone(&self) -> Self {
        let cloned: Box<dyn Symbolic> = (**self).dyn_clone();
        unsafe { memory::transmute(cloned) }
    }
}

impl PartialEq for dyn Symbolic + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.dyn_eq(other)
    }
}

impl PartialEq for dyn Symbolic + Send + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.dyn_eq(other)
    }
}

impl PartialEq for dyn Symbolic + Sync + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.dyn_eq(other)
    }
}

impl PartialEq for dyn Symbolic + Send + Sync + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.dyn_eq(other)
    }
}

impl Eq for dyn Symbolic + '_ {}
impl Eq for dyn Symbolic + Send + '_ {}
impl Eq for dyn Symbolic + Sync + '_ {}
impl Eq for dyn Symbolic + Send + Sync + '_ {}

impl Hash for dyn Symbolic + '_ {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dyn_hash(state);
    }
}

impl Hash for dyn Symbolic + Send + '_ {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dyn_hash(state);
    }
}

impl Hash for dyn Symbolic + Sync + '_ {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dyn_hash(state);
    }
}

impl Hash for dyn Symbolic + Send + Sync + '_ {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dyn_hash(state);
    }
}

impl Symbolic for Symbol {
    fn brand(&self) -> Option<Token<'static>> {
        self.value.brand()
    }

    fn as_any(&self) -> &dyn Any where Self: 'static {
        self
    }

    fn dyn_clone(&self) -> Box<dyn Symbolic> {
        Box::new(Self {
            value: self.value.clone(),
            span: self.span.clone(),
            members: self.members.clone(),
        })
    }

    fn dyn_eq(&self, other: &dyn Symbolic) -> bool {
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

impl Symbolic for Inclusion<Box<Element<'static>>> {
    fn brand(&self) -> Option<Token<'static>> {
        self.get_target().clone().brand()
    }

    fn as_any(&self) -> &dyn Any where Self: 'static {
        self
    }

    fn dyn_clone(&self) -> Box<dyn Symbolic> {
        Box::new(self.clone())
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

impl Symbolic for Implementation<Box<Element<'static>>, Box<Element<'static>>, Symbol> {
    fn brand(&self) -> Option<Token<'static>> {
        self.get_target().clone().brand()
    }

    fn as_any(&self) -> &dyn Any where Self: 'static {
        self
    }

    fn dyn_clone(&self) -> Box<dyn Symbolic> {
        Box::new(self.clone())
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

impl Symbolic for Interface<Box<Element<'static>>, Symbol> {
    fn brand(&self) -> Option<Token<'static>> {
        self.get_target().clone().brand()
    }

    fn as_any(&self) -> &dyn Any where Self: 'static {
        self
    }

    fn dyn_clone(&self) -> Box<dyn Symbolic> {
        Box::new(self.clone())
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

impl Symbolic for Binding<Box<Element<'static>>, Box<Element<'static>>, Box<Element<'static>>> {
    fn brand(&self) -> Option<Token<'static>> {
        self.get_target().clone().brand()
    }

    fn as_any(&self) -> &dyn Any where Self: 'static {
        self
    }

    fn dyn_clone(&self) -> Box<dyn Symbolic> {
        Box::new(self.clone())
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

impl Symbolic for Structure<Box<Element<'static>>, Symbol> {
    fn brand(&self) -> Option<Token<'static>> {
        self.get_target().clone().brand()
    }

    fn as_any(&self) -> &dyn Any where Self: 'static {
        self
    }

    fn dyn_clone(&self) -> Box<dyn Symbolic> {
        Box::new(self.clone())
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

impl Symbolic for Enumeration<Box<Element<'static>>, Element<'static>> {
    fn brand(&self) -> Option<Token<'static>> {
        self.get_target().clone().brand()
    }

    fn as_any(&self) -> &dyn Any where Self: 'static {
        self
    }

    fn dyn_clone(&self) -> Box<dyn Symbolic> {
        Box::new(self.clone())
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

impl Symbolic for Method<Box<Element<'static>>, Symbol, Box<Element<'static>>, Option<Box<Element<'static>>>> {
    fn brand(&self) -> Option<Token<'static>> {
        self.get_target().clone().brand()
    }

    fn as_any(&self) -> &dyn Any where Self: 'static {
        self
    }

    fn dyn_clone(&self) -> Box<dyn Symbolic> {
        Box::new(self.clone())
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

impl Symbolic for Module<Element<'static>> {
    fn brand(&self) -> Option<Token<'static>> {
        self.get_target().brand().clone()
    }

    fn as_any(&self) -> &dyn Any where Self: 'static {
        self
    }

    fn dyn_clone(&self) -> Box<dyn Symbolic> {
        Box::new(self.clone())
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

impl Symbolic for Element<'static> {
    fn brand(&self) -> Option<Token<'static>> {
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

    fn dyn_clone(&self) -> Box<dyn Symbolic> {
        Box::new(self.clone())
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
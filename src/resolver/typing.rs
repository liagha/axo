use crate::{
    data::{Boolean, Identity, Scale, Str, Aggregate},
};

#[derive(Clone, Debug)]
pub struct Type<'typing> {
    pub identity: Identity,
    pub kind: TypeKind<'typing>,
}

impl<'typing> Type<'typing> {
    pub fn new(identity: Identity, kind: TypeKind<'typing>) -> Self {
        Self { identity, kind }
    }
}

impl<'typing> From<TypeKind<'typing>> for Type<'typing> {
    fn from(kind: TypeKind<'typing>) -> Self {
        Self::new(0, kind)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypeKind<'typing> {
    Integer { size: Scale, signed: Boolean },
    Float { size: Scale },
    Boolean,
    String,
    Character,
    Pointer { target: Box<Type<'typing>> },
    Array { member: Box<Type<'typing>>, size: Scale },
    Tuple { members: Vec<Type<'typing>> },
    Void,
    Variable(Identity),
    Unknown,
    Structure(Aggregate<Str<'typing>, Type<'typing>>),
    Union(Aggregate<Str<'typing>, Type<'typing>>),
    Enumeration(Aggregate<Str<'typing>, Type<'typing>>),
    Function(Str<'typing>, Vec<Type<'typing>>, Option<Box<Type<'typing>>>),
}

impl<'typing> PartialEq for Type<'typing> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

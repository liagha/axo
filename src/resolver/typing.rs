use crate::{
    data::{Boolean, Identity, Scale, Str, Aggregate},
};

#[derive(Clone, Debug)]
pub struct Type<'typing> {
    pub kind: TypeKind<'typing>,
}

impl<'typing> Type<'typing> {
    pub fn new(kind: TypeKind<'typing>) -> Self {
        Self { kind }
    }

    pub fn void() -> Self {
        Self::new(TypeKind::Void)
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
    Constructor(Identity, Aggregate<Str<'typing>, Type<'typing>>),
    Structure(Identity, Aggregate<Str<'typing>, Type<'typing>>),
    Union(Identity, Aggregate<Str<'typing>, Type<'typing>>),
    Enumeration(Identity, Aggregate<Str<'typing>, Type<'typing>>),
    Function(Str<'typing>, Vec<Type<'typing>>, Option<Box<Type<'typing>>>),
}

impl<'typing> PartialEq for Type<'typing> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

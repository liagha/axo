use {
    crate::{
        data::{Str, Scale},
        schema::*,
        tracker::Span,
    }
};

#[derive(Clone, Debug)]
pub struct Type<'ty> {
    kind: TypeKind<'ty>,
    pub span: Span<'ty>,
}

impl<'ty> Type<'ty> {
    pub fn new(kind: TypeKind<'ty>, span: Span<'ty>) -> Self {
        Self { kind, span }
    }

    pub fn unit(span: Span<'ty>) -> Self {
        Self::new(TypeKind::Tuple { items: Vec::new() }, span)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypeKind<'ty> {
    Array { item: Box<Type<'ty>>, size: Scale },
    Tuple { items: Vec<Type<'ty>> },

    Type(Box<Type<'ty>>),

    Structure(Structure<Str<'ty>, Box<Type<'ty>>>),
    Enumeration(Structure<Str<'ty>, Box<Type<'ty>>>),
    Method(Method<Str<'ty>, Box<Type<'ty>>, Box<Type<'ty>>, Box<Type<'ty>>>),
}

impl<'ty> PartialEq for Type<'ty> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}
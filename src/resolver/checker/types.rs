use {
    crate::{
        scanner::Token,
        data::{Str, Scale},
        schema::{Enumeration, Method, Structure},
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
    Integer { size: Scale },
    Float { size: Scale },
    Boolean,
    Array { item: Box<Type<'ty>>, size: Scale },
    Tuple { items: Vec<Type<'ty>> },

    Type(Box<Type<'ty>>),

    Structure(Structure<Str<'ty>, Box<Type<'ty>>>),
    Enumeration(Enumeration<Str<'ty>, Box<Type<'ty>>>),
    Method(Method<Str<'ty>, Box<Type<'ty>>, Box<Type<'ty>>, Box<Type<'ty>>>),
}

impl<'ty> PartialEq for Type<'ty> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}
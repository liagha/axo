use {
    crate::{
        scanner::Token,
        data::{Str, Scale},
        schema::{Enumeration, Method, Structure},
    }
};

#[derive(Debug, PartialEq)]
pub struct Type<'ty> {
    kind: TypeKind<'ty>,
}

impl<'ty> Type<'ty> {
    pub fn new(kind: TypeKind<'ty>) -> Self {
        Self { kind }
    }

    pub fn unit() -> Self {
        Self { kind: TypeKind::Tuple { items: Vec::new() }}
    }
}

#[derive(Debug, PartialEq)]
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
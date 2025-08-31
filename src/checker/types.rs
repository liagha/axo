use {
    crate::{
        scanner::Token,
        data::{Str, Scale},
        schema::{Enumeration, Method, Structure},
    }
};

#[derive(Debug)]
pub struct Type<'ty> {
    kind: TypeKind<'ty>,
}

#[derive(Debug)]
pub enum TypeKind<'ty> {
    Integer { size: Scale },
    Float { size: Scale },
    Array { item: Box<Type<'ty>>, size: Scale },

    Type(Box<Type<'ty>>),

    Structure(Structure<Str<'ty>, Box<Type<'ty>>>),
    Enumeration(Enumeration<Str<'ty>, Box<Type<'ty>>>),
    Method(Method<Str<'ty>, Box<Type<'ty>>, Box<Type<'ty>>, Box<Type<'ty>>>),
}
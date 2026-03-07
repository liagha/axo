use crate::{
    data::{Boolean, Scale, Str},
    tracker::Span,
};
use crate::data::*;

#[derive(Clone, Debug)]
pub struct Type<'ty> {
    pub kind: TypeKind<'ty>,
    pub span: Span<'ty>,
}

impl<'ty> Type<'ty> {
    pub fn new(kind: TypeKind<'ty>, span: Span<'ty>) -> Self {
        Self { kind, span }
    }

    pub fn unit(span: Span<'ty>) -> Self {
        Self::new(TypeKind::Tuple { members: Vec::new() }, span)
    }

    pub fn integer(bits: Scale, signed: Boolean, span: Span<'ty>) -> Self {
        Self::new(TypeKind::Integer { bits, signed }, span)
    }

    pub fn float(bits: Scale, span: Span<'ty>) -> Self {
        Self::new(TypeKind::Float { bits }, span)
    }

    pub fn boolean(span: Span<'ty>) -> Self {
        Self::new(TypeKind::Boolean, span)
    }

    pub fn string(span: Span<'ty>) -> Self {
        Self::new(TypeKind::String, span)
    }

    pub fn character(span: Span<'ty>) -> Self {
        Self::new(TypeKind::Character, span)
    }

    pub fn pointer(to: Type<'ty>, span: Span<'ty>) -> Self {
        Self::new(TypeKind::Pointer { to: Box::new(to) }, span)
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self.kind, TypeKind::Integer { .. } | TypeKind::Float { .. })
    }

    pub fn is_integer(&self) -> bool {
        matches!(self.kind, TypeKind::Integer { .. })
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self.kind, TypeKind::Boolean)
    }

    pub fn is_infer(&self) -> bool {
        matches!(self.kind, TypeKind::Unknown)
    }

    pub fn is_pointer(&self) -> bool {
        matches!(self.kind, TypeKind::Pointer { .. })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypeKind<'ty> {
    Integer { bits: Scale, signed: Boolean },
    Float { bits: Scale },
    Boolean,
    String,
    Character,
    Pointer { to: Box<Type<'ty>> },
    Array { member: Box<Type<'ty>>, size: Scale },
    Tuple { members: Vec<Type<'ty>> },
    Unknown,

    Type(Box<Type<'ty>>),

    Structure(Structure<Str<'ty>, Box<Type<'ty>>>),
    Enumeration(Structure<Str<'ty>, Box<Type<'ty>>>),
    Method(Method<Str<'ty>, Box<Type<'ty>>, Box<Type<'ty>>, Box<Type<'ty>>>),
}

impl<'ty> TypeKind<'ty> {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "Int8" => Some(Self::Integer {
                bits: 8,
                signed: true,
            }),
            "Int16" => Some(Self::Integer {
                bits: 16,
                signed: true,
            }),
            "Int32" => Some(Self::Integer {
                bits: 32,
                signed: true,
            }),
            "Int64" => Some(Self::Integer {
                bits: 64,
                signed: true,
            }),
            "UInt8" => Some(Self::Integer {
                bits: 8,
                signed: false,
            }),
            "UInt16" => Some(Self::Integer {
                bits: 16,
                signed: false,
            }),
            "UInt32" => Some(Self::Integer {
                bits: 32,
                signed: false,
            }),
            "UInt64" => Some(Self::Integer {
                bits: 64,
                signed: false,
            }),
            "Float32" => Some(Self::Float { bits: 32 }),
            "Float64" => Some(Self::Float { bits: 64 }),
            "Bool" => Some(Self::Boolean),
            "Char" | "Character" => Some(Self::Character),
            "String" => Some(Self::String),
            "Integer" => Some(Self::Integer {
                bits: 64,
                signed: true,
            }),
            "Float" => Some(Self::Float { bits: 64 }),
            "Boolean" => Some(Self::Boolean),
            "Pointer" => Some(Self::Pointer { to: Box::from(Type::new(Self::Unknown, Span::void())) }),
            _ => None,
        }
    }
}

impl<'ty> PartialEq for Type<'ty> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

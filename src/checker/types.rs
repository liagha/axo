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
        Self::new(TypeKind::Integer { size: bits, signed }, span)
    }

    pub fn float(bits: Scale, span: Span<'ty>) -> Self {
        Self::new(TypeKind::Float { size: bits }, span)
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
        Self::new(TypeKind::Pointer { target: Box::new(to) }, span)
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
        matches!(self.kind, TypeKind::Void)
    }

    pub fn is_pointer(&self) -> bool {
        matches!(self.kind, TypeKind::Pointer { .. })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypeKind<'ty> {
    Integer { size: Scale, signed: Boolean },
    Float { size: Scale },
    Boolean,
    String,
    Character,
    Pointer { target: Box<Type<'ty>> },
    Array { member: Box<Type<'ty>>, size: Scale },
    Tuple { members: Vec<Type<'ty>> },
    Void,

    Type,

    Constructor(Structure<Str<'ty>, Type<'ty>>),
    Structure(Structure<Str<'ty>, Type<'ty>>),
    Enumeration(Structure<Str<'ty>, Type<'ty>>),
    Function(Function<Str<'ty>, Type<'ty>, Box<Type<'ty>>, Box<Type<'ty>>>),
}

impl<'ty> TypeKind<'ty> {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "Int8" => Some(Self::Integer {
                size: 8,
                signed: true,
            }),
            "Int16" => Some(Self::Integer {
                size: 16,
                signed: true,
            }),
            "Int32" => Some(Self::Integer {
                size: 32,
                signed: true,
            }),
            "Int64" => Some(Self::Integer {
                size: 64,
                signed: true,
            }),
            "UInt8" => Some(Self::Integer {
                size: 8,
                signed: false,
            }),
            "UInt16" => Some(Self::Integer {
                size: 16,
                signed: false,
            }),
            "UInt32" => Some(Self::Integer {
                size: 32,
                signed: false,
            }),
            "UInt64" => Some(Self::Integer {
                size: 64,
                signed: false,
            }),
            "Float32" => Some(Self::Float { size: 32 }),
            "Float64" => Some(Self::Float { size: 64 }),
            "Bool" => Some(Self::Boolean),
            "Char" | "Character" => Some(Self::Character),
            "String" => Some(Self::String),
            "Integer" => Some(Self::Integer {
                size: 64,
                signed: true,
            }),
            "Float" => Some(Self::Float { size: 64 }),
            "Boolean" => Some(Self::Boolean),
            "Pointer" => Some(Self::Pointer { target: Box::from(Type::new(Self::Void, Span::void())) }),
            _ => None,
        }
    }
}

impl<'ty> PartialEq for Type<'ty> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

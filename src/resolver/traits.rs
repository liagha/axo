// src/resolver/typing.rs

use crate::data::{Aggregate, Boolean, Identity, Scale, Str};
use crate::internal::cache::{Encode, Decode};
use crate::internal::hash::Set;
use crate::resolver::{Scope, Type, TypeKind};

impl<'typing> Encode for Type<'typing> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.identity.encode(buffer);
        self.kind.encode(buffer);
    }
}

impl<'typing> Decode<'typing> for Type<'typing> {
    fn decode(buffer: &'typing [u8], cursor: &mut usize) -> Self {
        Type {
            identity: Identity::decode(buffer, cursor),
            kind: TypeKind::decode(buffer, cursor),
        }
    }
}

impl<'typing> Encode for TypeKind<'typing> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        match self {
            TypeKind::Integer { size, signed } => {
                buffer.push(0);
                size.encode(buffer);
                signed.encode(buffer);
            }
            TypeKind::Float { size } => {
                buffer.push(1);
                size.encode(buffer);
            }
            TypeKind::Boolean => buffer.push(2),
            TypeKind::String => buffer.push(3),
            TypeKind::Character => buffer.push(4),
            TypeKind::Pointer { target } => {
                buffer.push(5);
                target.encode(buffer);
            }
            TypeKind::Array { member, size } => {
                buffer.push(6);
                member.encode(buffer);
                size.encode(buffer);
            }
            TypeKind::Tuple { members } => {
                buffer.push(7);
                members.encode(buffer);
            }
            TypeKind::Void => buffer.push(8),
            TypeKind::Variable(v) => {
                buffer.push(9);
                v.encode(buffer);
            }
            TypeKind::Unknown => buffer.push(10),
            TypeKind::Structure(v) => {
                buffer.push(11);
                v.encode(buffer);
            }
            TypeKind::Union(v) => {
                buffer.push(12);
                v.encode(buffer);
            }
            TypeKind::Function(name, args, output) => {
                buffer.push(13);
                name.encode(buffer);
                args.encode(buffer);
                output.encode(buffer);
            }
        }
    }
}

impl<'typing> Decode<'typing> for TypeKind<'typing> {
    fn decode(buffer: &'typing [u8], cursor: &mut usize) -> Self {
        let tag = buffer[*cursor];
        *cursor += 1;
        match tag {
            0 => TypeKind::Integer {
                size: Scale::decode(buffer, cursor),
                signed: Boolean::decode(buffer, cursor),
            },
            1 => TypeKind::Float {
                size: Scale::decode(buffer, cursor),
            },
            2 => TypeKind::Boolean,
            3 => TypeKind::String,
            4 => TypeKind::Character,
            5 => TypeKind::Pointer {
                target: Box::decode(buffer, cursor),
            },
            6 => TypeKind::Array {
                member: Box::decode(buffer, cursor),
                size: Scale::decode(buffer, cursor),
            },
            7 => TypeKind::Tuple {
                members: Vec::decode(buffer, cursor),
            },
            8 => TypeKind::Void,
            9 => TypeKind::Variable(Identity::decode(buffer, cursor)),
            10 => TypeKind::Unknown,
            11 => TypeKind::Structure(Aggregate::decode(buffer, cursor)),
            12 => TypeKind::Union(Aggregate::decode(buffer, cursor)),
            13 => TypeKind::Function(
                Str::decode(buffer, cursor),
                Vec::decode(buffer, cursor),
                Option::decode(buffer, cursor),
            ),
            _ => panic!(),
        }
    }
}

impl Encode for Scope {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.symbols.encode(buffer);
        self.parent.encode(buffer);
    }
}

impl<'a> Decode<'a> for Scope {
    fn decode(buffer: &'a [u8], cursor: &mut usize) -> Self {
        Scope {
            symbols: Set::decode(buffer, cursor),
            parent: Option::decode(buffer, cursor),
        }
    }
}
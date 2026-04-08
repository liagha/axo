use crate::data::{Aggregate, Boolean, Identity, Scale, Str};
use crate::internal::cache::{Decode, Encode};
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
            TypeKind::Module(name) => {
                buffer.push(0);
                name.encode(buffer);
            }
            TypeKind::Integer { size, signed } => {
                buffer.push(1);
                size.encode(buffer);
                signed.encode(buffer);
            }
            TypeKind::Float { size } => {
                buffer.push(2);
                size.encode(buffer);
            }
            TypeKind::Boolean => buffer.push(3),
            TypeKind::String => buffer.push(4),
            TypeKind::Character => buffer.push(5),
            TypeKind::Pointer { target } => {
                buffer.push(6);
                target.encode(buffer);
            }
            TypeKind::Array { member, size } => {
                buffer.push(7);
                member.encode(buffer);
                size.encode(buffer);
            }
            TypeKind::Tuple { members } => {
                buffer.push(8);
                members.encode(buffer);
            }
            TypeKind::Void => buffer.push(9),
            TypeKind::Variable(v) => {
                buffer.push(10);
                v.encode(buffer);
            }
            TypeKind::Unknown => buffer.push(11),
            TypeKind::Structure(v) => {
                buffer.push(12);
                v.encode(buffer);
            }
            TypeKind::Union(v) => {
                buffer.push(13);
                v.encode(buffer);
            }
            TypeKind::Function(name, args, output) => {
                buffer.push(14);
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
            0 => TypeKind::Module(Str::decode(buffer, cursor)),
            1 => TypeKind::Integer {
                size: Scale::decode(buffer, cursor),
                signed: Boolean::decode(buffer, cursor),
            },
            2 => TypeKind::Float {
                size: Scale::decode(buffer, cursor),
            },
            3 => TypeKind::Boolean,
            4 => TypeKind::String,
            5 => TypeKind::Character,
            6 => TypeKind::Pointer {
                target: Box::decode(buffer, cursor),
            },
            7 => TypeKind::Array {
                member: Box::decode(buffer, cursor),
                size: Scale::decode(buffer, cursor),
            },
            8 => TypeKind::Tuple {
                members: Vec::decode(buffer, cursor),
            },
            9 => TypeKind::Void,
            10 => TypeKind::Variable(Identity::decode(buffer, cursor)),
            11 => TypeKind::Unknown,
            12 => TypeKind::Structure(Aggregate::decode(buffer, cursor)),
            13 => TypeKind::Union(Aggregate::decode(buffer, cursor)),
            14 => TypeKind::Function(
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

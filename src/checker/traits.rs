use {
    crate::{
        format::Show,
    }
};
use crate::checker::{Type, TypeKind};
use crate::data::Str;

impl<'ty> Show<'ty> for Type<'ty> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'ty> {
        match verbosity {
            0 => {
                format!("{}", self.kind.format(verbosity))
            }

            1 => {
                format!("Type({})", self.kind.format(verbosity))
            }

            _ => {
                self.format(verbosity - 1).to_string()
            }
        }.into()
    }
}

impl<'ty> Show<'ty> for TypeKind<'ty> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'ty> {
        match verbosity {
            0 => {
                match self {
                    TypeKind::Integer { .. } => {
                        format!("Integer")
                    }
                    TypeKind::Float { .. } => {
                        format!("Float")
                    }
                    TypeKind::Boolean => {
                        format!("Boolean")
                    }
                    TypeKind::String => {
                        format!("String")
                    }
                    TypeKind::Character => {
                        format!("Character")
                    }
                    TypeKind::Pointer { .. } => {
                        format!("Pointer")
                    }
                    TypeKind::Array { .. } => {
                        format!("Array")
                    }
                    TypeKind::Tuple { members } => {
                        format!("Tuple({})", members.format(verbosity))
                    }
                    TypeKind::Unknown => {
                        format!("Unknown")
                    }
                    TypeKind::Type(_) => {
                        format!("Type")
                    }
                    TypeKind::Structure(_) => {
                        format!("Structure")
                    }
                    TypeKind::Enumeration(_) => {
                        format!("Enumeration")
                    }
                    TypeKind::Method(_) => {
                        format!("Method")
                    }
                }
            }

            _ => {
                self.format(verbosity - 1).to_string()
            }
        }.into()
    }
}
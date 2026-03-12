use {
    crate::{
        format::Show,
    }
};
use crate::checker::{Type, TypeKind};
use crate::data::Str;

impl<'typ> Show<'typ> for Type<'typ> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'typ> {
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

impl<'typ> Show<'typ> for TypeKind<'typ> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'typ> {
        match verbosity {
            0 => {
                match self {
                    TypeKind::Integer { size, signed } => {
                        format!("Integer[{}{}]", if *signed { "Signed | " } else { "" }, size)
                    }
                    TypeKind::Float { size } => {
                        format!("Float[{}]", size)
                    }
                    TypeKind::Boolean => {
                        "Boolean".to_string()
                    }
                    TypeKind::String => {
                        "String".to_string()
                    }
                    TypeKind::Character => {
                        "Character".to_string()
                    }
                    TypeKind::Pointer { target } => {
                        format!("Pointer({})", target.format(verbosity))
                    }
                    TypeKind::Array { member, size } => {
                        format!("Array[{}; {}]", member.format(verbosity), size)
                    }
                    TypeKind::Tuple { members } => {
                        format!("Tuple({})", members.format(verbosity))
                    }
                    TypeKind::Void => "Void".to_string(),
                    TypeKind::Constructor(_) => {
                        "Constructor".to_string()
                    }
                    TypeKind::Structure(_) => {
                        "Structure".to_string()
                    }
                    TypeKind::Union(_) => {
                        "Union".to_string()
                    }
                    TypeKind::Function(name, members, output) => {
                        format!("Function({})[{}]:{}", name.format(verbosity), members.format(verbosity), output.format(verbosity))
                    }
                    TypeKind::Variable(variable) => {
                        format!("Variable({})", variable)
                    },
                    TypeKind::Unknown => "Unknown".to_string(),
                }
            }

            _ => {
                self.format(verbosity - 1).to_string()
            }
        }.into()
    }
}
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
                    TypeKind::Pointer { .. } => {
                        "Pointer".to_string()
                    }
                    TypeKind::Array { .. } => {
                        "Array".to_string()
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
                    TypeKind::Function(_, _, _) => {
                        "Function".to_string()
                    }
                    TypeKind::Variable(_) => "Variable".to_string(),
                    TypeKind::Unknown => "Unknown".to_string(),
                }
            }

            _ => {
                self.format(verbosity - 1).to_string()
            }
        }.into()
    }
}
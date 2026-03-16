use {
    crate::{
        data::Str,
        format::{Show, Verbosity},
        parser::Symbol,
        resolver::scope::Scope,
    },
};

impl<'scope> Show<'scope> for Scope<Symbol<'scope>> {
    fn format(&self, verbosity: Verbosity) -> Str<'scope> {
        match verbosity {
            _ => {
                format!("{}", self.symbols.format(verbosity))
            }
        }.into()
    }
}

use crate::resolver::{Type, TypeKind};

impl<'typing> Show<'typing> for Type<'typing> {
    fn format(&self, verbosity: Verbosity) -> Str<'typing> {
        match verbosity {
            Verbosity::Minimal => {
                format!("{}", self.kind.format(verbosity))
            }

            Verbosity::Detailed => {
                format!("Type({})", self.kind.format(verbosity))
            }

            _ => {
                self.format(verbosity.fallback()).to_string()
            }
        }.into()
    }
}

impl<'typing> Show<'typing> for TypeKind<'typing> {
    fn format(&self, verbosity: Verbosity) -> Str<'typing> {
        match verbosity {
            Verbosity::Minimal => {
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
                    TypeKind::Constructor(_,_) => {
                        "Constructor".to_string()
                    }
                    TypeKind::Structure(_,_) => {
                        "Structure".to_string()
                    }
                    TypeKind::Union(_,_) => {
                        "Union".to_string()
                    }
                    TypeKind::Enumeration(_, _) => {
                        "Enumeration".to_string()
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
                self.format(verbosity.fallback()).to_string()
            }
        }.into()
    }
}

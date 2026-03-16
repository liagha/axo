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
            Verbosity::Off => "".into(),
            Verbosity::Minimal => self.symbols.format(verbosity),
            Verbosity::Detailed => format!("Scope({})", self.symbols.format(verbosity)).into(),
            Verbosity::Debug => format!(
                "Scope {{\n{}\n}}",
                format!("symbols: {}", self.symbols.format(verbosity)).indent(verbosity)
            ).into(),
        }
    }
}

use crate::resolver::{Type, TypeKind};

impl<'typing> Show<'typing> for Type<'typing> {
    fn format(&self, verbosity: Verbosity) -> Str<'typing> {
        match verbosity {
            Verbosity::Off => "".into(),
            Verbosity::Minimal => self.kind.format(verbosity),
            Verbosity::Detailed => format!("Type({})", self.kind.format(verbosity)).into(),
            Verbosity::Debug => format!(
                "Type {{\n{}\n}}",
                format!("kind: {}", self.kind.format(verbosity)).indent(verbosity)
            ).into(),
        }
    }
}

impl<'typing> Show<'typing> for TypeKind<'typing> {
    fn format(&self, verbosity: Verbosity) -> Str<'typing> {
        if verbosity == Verbosity::Off {
            return "".into();
        }

        match self {
            TypeKind::Integer { size, signed } => match verbosity {
                Verbosity::Minimal => format!("{}{}", if *signed { "i" } else { "u" }, size).into(),
                Verbosity::Detailed => format!("Integer[{}{}]", if *signed { "Signed | " } else { "" }, size).into(),
                Verbosity::Debug => format!("Integer {{\n    size: {},\n    signed: {}\n}}", size, signed).into(),
                _ => "".into(),
            },
            TypeKind::Float { size } => match verbosity {
                Verbosity::Minimal => format!("f{}", size).into(),
                Verbosity::Detailed => format!("Float[{}]", size).into(),
                Verbosity::Debug => format!("Float {{\n    size: {}\n}}", size).into(),
                _ => "".into(),
            },
            TypeKind::Boolean => match verbosity {
                Verbosity::Minimal => "bool".into(),
                Verbosity::Detailed => "Boolean".into(),
                Verbosity::Debug => "Boolean {}".into(),
                _ => "".into(),
            },
            TypeKind::String => match verbosity {
                Verbosity::Minimal => "str".into(),
                Verbosity::Detailed => "String".into(),
                Verbosity::Debug => "String {}".into(),
                _ => "".into(),
            },
            TypeKind::Character => match verbosity {
                Verbosity::Minimal => "char".into(),
                Verbosity::Detailed => "Character".into(),
                Verbosity::Debug => "Character {}".into(),
                _ => "".into(),
            },
            TypeKind::Pointer { target } => match verbosity {
                Verbosity::Minimal => format!("*{}", target.format(verbosity)).into(),
                Verbosity::Detailed => format!("Pointer({})", target.format(verbosity)).into(),
                Verbosity::Debug => format!("Pointer {{\n{}\n}}", format!("target: {}", target.format(verbosity)).indent(verbosity)).into(),
                _ => "".into(),
            },
            TypeKind::Array { member, size } => match verbosity {
                Verbosity::Minimal => format!("[{}; {}]", member.format(verbosity), size).into(),
                Verbosity::Detailed => format!("Array[{} | {}]", member.format(verbosity), size).into(),
                Verbosity::Debug => format!(
                    "Array {{\n{},\n{}\n}}",
                    format!("member: {}", member.format(verbosity)).indent(verbosity),
                    format!("size: {}", size).indent(verbosity)
                ).into(),
                _ => "".into(),
            },
            TypeKind::Tuple { members } => match verbosity {
                Verbosity::Minimal => format!("({})", members.format(verbosity)).into(),
                Verbosity::Detailed => format!("Tuple({})", members.format(verbosity)).into(),
                Verbosity::Debug => format!("Tuple {{\n{}\n}}", format!("members: {}", members.format(verbosity)).indent(verbosity)).into(),
                _ => "".into(),
            },
            TypeKind::Function(name, members, output) => match verbosity {
                Verbosity::Minimal => format!("fn {}({}) -> {}", name.format(verbosity), members.format(verbosity), output.format(verbosity)).into(),
                Verbosity::Detailed => format!("Function({} | [{}]) -> {}", name.format(verbosity), members.format(verbosity), output.format(verbosity)).into(),
                Verbosity::Debug => format!(
                    "Function {{\n{},\n{},\n{}\n}}",
                    format!("name: {}", name.format(verbosity)).indent(verbosity),
                    format!("members: {}", members.format(verbosity)).indent(verbosity),
                    format!("output: {}", output.format(verbosity)).indent(verbosity)
                ).into(),
                _ => "".into(),
            },
            TypeKind::Variable(variable) => match verbosity {
                Verbosity::Minimal => format!("{}", variable).into(),
                Verbosity::Detailed => format!("Variable({})", variable).into(),
                Verbosity::Debug => format!("Variable {{\n    id: {}\n}}", variable).into(),
                _ => "".into(),
            },
            TypeKind::Void => match verbosity {
                Verbosity::Minimal => "()".into(),
                Verbosity::Detailed => "Void".into(),
                Verbosity::Debug => "Void {}".into(),
                _ => "".into(),
            },
            TypeKind::Unknown => match verbosity {
                Verbosity::Minimal => "_".into(),
                Verbosity::Detailed => "Unknown".into(),
                Verbosity::Debug => "Unknown {}".into(),
                _ => "".into(),
            },
            TypeKind::Constructor(_, _) => match verbosity {
                Verbosity::Minimal => "Constructor".into(),
                Verbosity::Detailed => "Constructor(...)".into(),
                Verbosity::Debug => "Constructor {\n    ...\n}".into(),
                _ => "".into(),
            },
            TypeKind::Structure(_, _) => match verbosity {
                Verbosity::Minimal => "struct".into(),
                Verbosity::Detailed => "Structure(...)".into(),
                Verbosity::Debug => "Structure {\n    ...\n}".into(),
                _ => "".into(),
            },
            TypeKind::Union(_, _) => match verbosity {
                Verbosity::Minimal => "union".into(),
                Verbosity::Detailed => "Union(...)".into(),
                Verbosity::Debug => "Union {\n    ...\n}".into(),
                _ => "".into(),
            },
            TypeKind::Enumeration(_, _) => match verbosity {
                Verbosity::Minimal => "enum".into(),
                Verbosity::Detailed => "Enumeration(...)".into(),
                Verbosity::Debug => "Enumeration {\n    ...\n}".into(),
                _ => "".into(),
            },
        }
    }
}

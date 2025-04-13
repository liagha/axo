use core::fmt;
use std::collections::HashSet;
use crate::axo_semantic::symbol::{Symbol, SymbolKind};
use crate::axo_semantic::error::ErrorKind;
use crate::axo_semantic::Resolver;
use crate::axo_semantic::scope::Scope;

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolKind::Expression(expr) => write!(f, "{}", expr),

            SymbolKind::Field { name, field_type, default } => {
                write!(f, "{}", name)?;
                if let Some(ty) = field_type {
                    write!(f, ": {}", ty)?;
                }
                if let Some(def) = default {
                    write!(f, " = {}", def)?;
                }
                Ok(())
            },

            SymbolKind::Variable { name, value, mutable, ty } => {
                write!(f, "{}{}", if *mutable { "mut " } else { "" }, name)?;
                if let Some(t) = ty {
                    write!(f, ": {}", t)?;
                }
                if let Some(val) = value {
                    write!(f, " = {}", val)?;
                }
                Ok(())
            },

            SymbolKind::Struct { name, fields } => {
                write!(f, "struct {} {{", name)?;
                if !fields.is_empty() {
                    write!(f, "\n")?;
                    for field in fields {
                        write!(f, "    {},\n", field)?;
                    }
                }
                write!(f, "}}")
            },

            SymbolKind::Enum { name, variants } => {
                write!(f, "enum {} {{", name)?;
                if !variants.is_empty() {
                    write!(f, "\n")?;
                    for variant in variants {
                        write!(f, "    {},\n", variant)?;
                    }
                }
                write!(f, "}}")
            },

            SymbolKind::Function { name, parameters, body, return_type } => {
                write!(f, "fn {}(", name)?;

                let mut params = parameters.iter().peekable();
                while let Some(param) = params.next() {
                    write!(f, "{}", param)?;
                    if params.peek().is_some() {
                        write!(f, ", ")?;
                    }
                }

                write!(f, ")")?;

                if let Some(rt) = return_type {
                    write!(f, " -> {}", rt)?;
                }

                write!(f, " {{\n    {}\n}}", body)
            },

            SymbolKind::Macro { name, parameters, body } => {
                write!(f, "macro {}(", name)?;

                let mut params = parameters.iter().peekable();
                while let Some(param) = params.next() {
                    write!(f, "{}", param)?;
                    if params.peek().is_some() {
                        write!(f, ", ")?;
                    }
                }

                write!(f, ") {{\n    {}\n}}", body)
            },

            SymbolKind::Trait { name, body, generic_params } => {
                write!(f, "trait {}", name)?;

                if !generic_params.is_empty() {
                    write!(f, "<")?;
                    let mut params = generic_params.iter().peekable();
                    while let Some(param) = params.next() {
                        write!(f, "{}", param)?;
                        if params.peek().is_some() {
                            write!(f, ", ")?;
                        }
                    }
                    write!(f, ">")?;
                }

                write!(f, " {{\n    {}\n}}", body)
            },

            SymbolKind::Impl { trait_, target, body } => {
                write!(f, "impl ")?;

                if let Some(tr) = trait_ {
                    write!(f, "{} for ", tr)?;
                }

                write!(f, "{} {{\n    {}\n}}", target, body)
            },

            SymbolKind::Error(err) => write!(f, "Error: {}", err),
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::ImmutableAssign(name) =>
                write!(f, "Cannot assign to immutable variable `{}`", name),
            ErrorKind::InvalidAssignTarget(target) =>
                write!(f, "Invalid assignment target: `{}`", target),
            ErrorKind::InvalidVariant(name) =>
                write!(f, "Invalid enum variant: `{}`", name),
            ErrorKind::InvalidStruct(name) =>
                write!(f, "Invalid struct: `{}`", name),
            ErrorKind::UnknownField(field) =>
                write!(f, "Unknown field: `{}`", field),
            ErrorKind::UnknownVariant(variant) =>
                write!(f, "Unknown variant: `{}`", variant),
            ErrorKind::ArgCountMismatch(expected, found) =>
                write!(f, "Argument count mismatch: expected {}, found {}", expected, found),
            ErrorKind::UndefinedSymbol(name, suggestion) => {
                write!(f, "Undefined symbol: `{}`", name)?;
                if let Some(suggest) = suggestion {
                    write!(f, ", did you mean `{}`?", suggest)?;
                }
                Ok(())
            },
            ErrorKind::AlreadyDefined(name) =>
                write!(f, "Symbol `{}` already defined", name),
            ErrorKind::InvalidAssignment =>
                write!(f, "Invalid assignment"),
            ErrorKind::NotProvided =>
                write!(f, "Value not provided"),
            ErrorKind::InvalidStructField(field) =>
                write!(f, "Invalid struct field: `{}`", field),
            ErrorKind::InvalidEnumVariant(variant) =>
                write!(f, "Invalid enum variant: `{}`", variant),
            ErrorKind::TypeMismatch(expected, found) =>
                write!(f, "Type mismatch: expected `{}`, found `{}`", expected, found),
            ErrorKind::InvalidExpression(expr) =>
                write!(f, "Invalid expression: `{}`", expr),
            ErrorKind::Other(msg) =>
                write!(f, "{}", msg),
        }
    }
}
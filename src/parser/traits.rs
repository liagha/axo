use crate::format::Show;
use {
    super::{Element, ElementKind, Symbol, SymbolKind},
    crate::{
        data::memory::discriminant,
        format::{self, Debug, Display, Formatter},
        internal::{
            hash::{Hash, Hasher},
            operation::Ordering,
        },
        tracker::{Span, Spanned},
    },
};
use crate::data::Str;

impl<'element> Show<'element> for Element<'element> {
    type Verbosity = u8;
    
    fn format(&self, verbosity: Self::Verbosity) -> Str<'element> {
        match verbosity {
            0 => {
                "".to_string()
            }

            1 => {
                format!("{}", self.kind.format(verbosity))
            }
            
            2 => {
                format!("{} | {:?}", self.kind.format(verbosity), self.span)
            }

            _ => {
                unimplemented!("the verbosity `{}` wasn't implemented for Element.", verbosity);
            }
        }.into()
    }
}

impl<'element> Show<'element> for ElementKind<'element> {
    type Verbosity = u8;
    
    fn format(&self, verbosity: Self::Verbosity) -> Str<'element> {
        match verbosity {
            0 => {
                "".to_string()
            }

            1 => {
                match self {
                    ElementKind::Literal(literal) => {
                        format!("{}", literal.format(verbosity))
                    }
                    ElementKind::Delimited(delimited) => {
                        format!(
                            "Delimited({}-{},{})",
                            delimited.start.format(verbosity),
                            delimited.end.format(verbosity),
                            delimited
                                .members
                                .iter()
                                .map(|item| format!(" {}", item.format(verbosity)))
                                .collect::<Vec<_>>()
                                .join(
                                    &*delimited
                                        .clone()
                                        .separator
                                        .map(|separator| format!(" {}", separator.format(verbosity)))
                                        .unwrap_or(" ".to_string())
                                ),
                        )
                    }

                    ElementKind::Binary(binary) => {
                        format!(
                            "Binary({} {} {})",
                            binary.left.format(verbosity), binary.operator.format(verbosity), binary.right.format(verbosity)
                        )
                    }
                    ElementKind::Unary(unary) => {
                        format!("Unary({} {})", unary.operator.format(verbosity), unary.operand.format(verbosity))
                    }

                    ElementKind::Index(index) => {
                        format!("Index({}[{}])", index.target.format(verbosity), index.members.format(verbosity))
                    }
                    ElementKind::Invoke(invoke) => {
                        format!("Invoke({}({}))", invoke.target.format(verbosity), invoke.members.format(verbosity))
                    }

                    ElementKind::Construct(construct) => {
                        format!(
                            "Constructor({} | {})",
                            construct.target.format(verbosity), construct.members.format(verbosity)
                        )
                    }
 
                    ElementKind::Symbolize(symbol) => format!("{}", symbol.format(verbosity)),
                }
            }

            2 => {
                match self {
                    ElementKind::Literal(literal) => {
                        format!("{:?}", literal.format(verbosity))
                    }
                    ElementKind::Delimited(delimited) => {
                        format!(
                            "Delimited({}-{}, {})",
                            delimited.start.format(verbosity),
                            delimited.end.format(verbosity),
                            delimited
                                .members
                                .iter()
                                .map(|item| format!("{}", item.format(verbosity)))
                                .collect::<Vec<_>>()
                                .join(
                                    &*delimited
                                        .clone()
                                        .separator
                                        .map(|separator| format!("{} ", separator.format(verbosity)))
                                        .unwrap_or(" ".to_string())
                                ),
                        )
                    }

                    ElementKind::Binary(binary) => {
                        format!(
                            "Binary({} {} {})",
                            binary.left.format(verbosity), binary.operator.format(verbosity), binary.right.format(verbosity)
                        )
                    }
                    ElementKind::Unary(unary) => {
                        format!("Unary({:?} {:?})", unary.operator.format(verbosity), unary.operand.format(verbosity))
                    }

                    ElementKind::Index(index) => {
                        format!("Index({:?}[{:?}])", index.target.format(verbosity), index.members.format(verbosity))
                    }
                    ElementKind::Invoke(invoke) => {
                        format!("Invoke({:?}({:?}))", invoke.target.format(verbosity), invoke.members.format(verbosity))
                    }

                    ElementKind::Construct(construct) => {
                        format!(
                            "Constructor({:?} | {:?})",
                            construct.target.format(verbosity), construct.members.format(verbosity)
                        )
                    }

                    ElementKind::Symbolize(symbol) => format!("+ {}", symbol.format(verbosity)),
                }
            }

            _ => {
                unimplemented!("the verbosity `{}` wasn't implemented for ElementKind.", verbosity);
            }
        }.into()
    }   
}

impl<'symbol> Show<'symbol> for Symbol<'symbol> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'symbol> {
        match verbosity {
            0 => {
                "".to_string()
            }

            1 => {
                format!(
                    "{}{}{}{}",
                    self.kind.format(verbosity),
                    if self.scope.is_empty() {
                        "".into()
                    } else {
                        format!("\nScope: {}", self.scope.format(verbosity)).indent(verbosity)
                    },
                    format!("\nSpecification: {}", self.specifier.format(verbosity)).indent(verbosity),
                    if self.scope.is_empty() {
                        "".into()
                    } else {
                        format!("\nGenerics: {}", self.generic.format(verbosity)).indent(verbosity)
                    }
                )
            }

            2 => {
                format!(
                    "{} | {:?} -> {}",
                    self.kind.format(verbosity), self.span, self.specifier.format(verbosity)
                )
            }

            _ => {
                unimplemented!("the verbosity `{}` wasn't implemented for Symbol.", verbosity);
            }
        }.into()
    }
}

impl<'symbol> Show<'symbol> for SymbolKind<'symbol> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'symbol> {
        match verbosity {
            0 => {
                "".to_string()
            }

            1 => {
                match self {
                    SymbolKind::Inclusion(inclusion) => {
                        format!("Inclusion({})", inclusion.target.format(verbosity))
                    }
                    SymbolKind::Extension(extension) => {
                        format!(
                            "Extension({}{}",
                            if let Some(extension) = &extension.extension {
                                format!("{}, ", extension.format(verbosity))
                            } else {
                                "".to_string()
                            },
                            format!("{}, {})", extension.target.format(verbosity), extension.members.format(verbosity))
                        )
                    }
                    SymbolKind::Binding(binding) => {
                        format!(
                            "Binding({} {}{}{})",
                            if binding.constant {
                                "Constant"
                            } else {
                                "Variable"
                            },
                            binding.target.format(verbosity),
                            if let Some(annotation) = &binding.annotation {
                                format!(" : {}", annotation.format(verbosity))
                            } else {
                                "".to_string()
                            },
                            if let Some(value) = &binding.value {
                                format!(" = {}", value.format(verbosity))
                            } else {
                                "".to_string()
                            }
                        )
                    }
                    SymbolKind::Structure(structure) => {
                        format!(
                            "Structure({} {})",
                            structure.target.format(verbosity), structure.members.format(verbosity)
                        )
                    }
                    SymbolKind::Enumeration(enumeration) => {
                        format!(
                            "Enumeration({} {})",
                            enumeration.target.format(verbosity), enumeration.members.format(verbosity)
                        )
                    }
                    SymbolKind::Method(method) => {
                        format!(
                            "Method({} {}{} -> {} : {})",
                            method.target.format(verbosity),
                            method.members.format(verbosity),
                            if method.variadic { "- Variadic" } else { "" },
                            method.output.clone().map(|value| *value).format(verbosity),
                            method.body.format(verbosity)
                        )
                    }
                    SymbolKind::Module(module) => {
                        format!("Module({})", module.target.format(verbosity))
                    }
                    SymbolKind::Preference(preference) => {
                        format!(
                            "Preference({}, {})",
                            preference.target.format(verbosity), preference.value.format(verbosity)
                        )
                    }
                }
            }

            2 => {
                match self {
                    SymbolKind::Inclusion(inclusion) => {
                        format!("Inclusion({})", inclusion.target.format(verbosity))
                    }
                    SymbolKind::Extension(extension) => {
                        format!(
                            "Extension({}{}",
                            if let Some(extension) = &extension.extension {
                                format!("{}, ", extension.format(verbosity))
                            } else {
                                "".to_string()
                            },
                            format!("{}, {})", extension.target.format(verbosity), extension.members.format(verbosity))
                        )
                    }
                    SymbolKind::Binding(binding) => {
                        format!(
                            "Binding({} {}{}{})",
                            if binding.constant {
                                "Constant"
                            } else {
                                "Variable"
                            },
                            binding.target.format(verbosity),
                            if let Some(annotation) = &binding.annotation {
                                format!(" : {}", annotation.format(verbosity))
                            } else {
                                "".to_string()
                            },
                            if let Some(value) = &binding.value {
                                format!(" = {}", value.format(verbosity))
                            } else {
                                "".to_string()
                            }
                        )
                    }
                    SymbolKind::Structure(structure) => {
                        format!(
                            "Structure({} {})",
                            structure.target.format(verbosity), structure.members.format(verbosity)
                        )
                    }
                    SymbolKind::Enumeration(enumeration) => {
                        format!(
                            "Enumeration({} {})",
                            enumeration.target.format(verbosity), enumeration.members.format(verbosity)
                        )
                    }
                    SymbolKind::Method(method) => {
                        format!(
                            "Method({} {}{} -> {} : {})",
                            method.target.format(verbosity),
                            method.members.format(verbosity),
                            if method.variadic { "- Variadic" } else { "" },
                            method.output.clone().map(|value| *value).format(verbosity),
                            method.body.format(verbosity)
                        )
                    }
                    SymbolKind::Module(module) => {
                        format!("Module({})", module.target.format(verbosity))
                    }
                    SymbolKind::Preference(preference) => {
                        format!(
                            "Preference({}, {})",
                            preference.target.format(verbosity), preference.value.format(verbosity)
                        )
                    }
                }
            }

            _ => {
                unimplemented!("the verbosity `{}` wasn't implemented for SymbolKind.", verbosity);
            }
        }.into()
    }
}

impl<'element> Hash for Element<'element> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
    }
}

impl<'element> Spanned<'element> for Element<'element> {
    #[track_caller]
    fn borrow_span(&self) -> Span<'element> {
        self.span
    }

    #[track_caller]
    fn span(self) -> Span<'element> {
        self.span
    }
}

impl<'element> Hash for ElementKind<'element> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ElementKind::Literal(kind) => {
                discriminant(self).hash(state);
                kind.hash(state);
            }

            ElementKind::Delimited(delimited) => {
                discriminant(self).hash(state);
                delimited.hash(state);
            }

            ElementKind::Construct(construct) => {
                discriminant(self).hash(state);
                construct.hash(state);
            }

            ElementKind::Binary(binary) => {
                discriminant(self).hash(state);
                binary.hash(state);
            }
            ElementKind::Unary(unary) => {
                discriminant(self).hash(state);
                unary.hash(state);
            }

            ElementKind::Index(index) => {
                discriminant(self).hash(state);
                index.hash(state);
            }
            ElementKind::Invoke(invoke) => {
                discriminant(self).hash(state);
                invoke.hash(state);
            }

            ElementKind::Symbolize(symbol) => {
                discriminant(self).hash(state);
                symbol.hash(state);
            }
        }
    }
}

impl<'element> PartialEq for Element<'element> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl<'element> PartialEq for ElementKind<'element> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ElementKind::Literal(a), ElementKind::Literal(b)) => a == b,

            (ElementKind::Delimited(a), ElementKind::Delimited(b)) => a == b,
            (ElementKind::Construct(a), ElementKind::Construct(b)) => a == b,

            (ElementKind::Binary(a), ElementKind::Binary(b)) => a == b,
            (ElementKind::Unary(a), ElementKind::Unary(b)) => a == b,

            (ElementKind::Index(a), ElementKind::Index(b)) => a == b,
            (ElementKind::Invoke(a), ElementKind::Invoke(b)) => a == b,

            (ElementKind::Symbolize(a), ElementKind::Symbolize(b)) => a == b,

            _ => false,
        }
    }
}

impl<'element> Clone for Element<'element> {
    fn clone(&self) -> Self {
        Element {
            kind: self.kind.clone(),
            span: self.span.clone(),
            id: self.id,
        }
    }
}

impl<'element> Clone for ElementKind<'element> {
    fn clone(&self) -> Self {
        match self {
            ElementKind::Literal(kind) => ElementKind::Literal(kind.clone()),

            ElementKind::Delimited(delimited) => ElementKind::Delimited(delimited.clone()),
            ElementKind::Construct(construct) => ElementKind::Construct(construct.clone()),

            ElementKind::Binary(binary) => ElementKind::Binary(binary.clone()),
            ElementKind::Unary(unary) => ElementKind::Unary(unary.clone()),

            ElementKind::Index(index) => ElementKind::Index(index.clone()),
            ElementKind::Invoke(invoke) => ElementKind::Invoke(invoke.clone()),

            ElementKind::Symbolize(symbol) => ElementKind::Symbolize(symbol.clone()),
        }
    }
}

impl<'element> Eq for Element<'element> {}

impl<'element> Eq for ElementKind<'element> {}

impl<'symbol> Clone for Symbol<'symbol> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            usages: self.usages.clone(),
            kind: self.kind.clone(),
            span: self.span.clone(),
            scope: self.scope.clone(),
            generic: self.generic.clone(),
            specifier: self.specifier.clone(),
        }
    }
}

impl<'symbol> Eq for Symbol<'symbol> {}

impl<'symbol> PartialEq for Symbol<'symbol> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl<'symbol> Hash for Symbol<'symbol> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
    }
}

impl<'symbol> PartialOrd for Symbol<'symbol> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'symbol> Ord for Symbol<'symbol> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

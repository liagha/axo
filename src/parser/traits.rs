use {
    super::{
        Element, ElementKind,
        Symbol, SymbolKind,
    },

    crate::{
        data::memory::discriminant,
        format::{self, Display, Debug, Formatter},
        internal::{
            operation::Ordering,
            hash::{
                Hash, Hasher
            },
        },
        tracker::{Span, Spanned},
    },
};
use crate::format::Show;

impl<'element> Debug for Element<'element> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}", self.kind)
        } else {
            write!(f, "{:?} | {:?}", self.kind, self.span)
        }
    }
}

impl<'element> Debug for ElementKind<'element> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            match self {
                ElementKind::Literal(literal) => {
                    write!(f, "{:#?}", literal)
                },
                ElementKind::Delimited(delimited) => {
                    write!(
                        f,
                        "Delimited({}-{}, {})",
                        delimited.start.to_string(),
                        delimited.end.to_string(),
                        delimited.items
                            .iter()
                            .map(|item| format!("{:#?}", item))
                            .collect::<Vec<_>>()
                            .join(
                                &*delimited.clone().separator.map(
                                    |separator| format!("{} ", separator)
                                ).unwrap_or(" ".to_string())
                            ),
                    )
                }

                ElementKind::Binary(binary) => {
                    write!(f, "Binary({:#?} {:#?} {:#?})", binary.left, binary.operator, binary.right)
                }
                ElementKind::Unary(unary) => {
                    write!(f, "Unary({:#?} {:#?})", unary.operator, unary.operand)
                },
                
                ElementKind::Closure(closure) => {
                    write!(f, "Closure({:#?} {:#?})", closure.members, closure.body)
                }

                ElementKind::Index(index) => {
                    write!(f, "Index({:#?}[{:#?}])", index.target, index.members)
                },
                ElementKind::Invoke(invoke) => {
                    write!(f, "Invoke({:#?}({:#?}))", invoke.target, invoke.members)
                },

                ElementKind::Construct(construct) => {
                    write!(f, "Constructor({:#?} | {:#?})", construct.target, construct.members)
                },

                ElementKind::Symbolize(symbol) => write!(f, "+ {:#?}", symbol),
            }
        } else {
            match self {
                ElementKind::Literal(literal) => {
                    write!(f, "{:?}", literal)
                },
                ElementKind::Delimited(delimited) => {
                    write!(
                        f,
                        "Delimited({}-{}, {})",
                        delimited.start.to_string(),
                        delimited.end.to_string(),
                        delimited.items
                            .iter()
                            .map(|item| format!("{:#?}", item))
                            .collect::<Vec<_>>()
                            .join(
                                &*delimited.clone().separator.map(
                                    |separator| format!("{} ", separator)
                                ).unwrap_or(" ".to_string())
                            ),
                    )
                }

                ElementKind::Binary(binary) => {
                    write!(f, "Binary({:?} {:?} {:?})", binary.left, binary.operator, binary.right)
                }
                ElementKind::Unary(unary) => {
                    write!(f, "Unary({:?} {:?})", unary.operator, unary.operand)
                },

                ElementKind::Closure(closure) => {
                    write!(f, "Closure({:#?} {:#?})", closure.members, closure.body)
                }

                ElementKind::Index(index) => {
                    write!(f, "Index({:?}[{:?}])", index.target, index.members)
                },
                ElementKind::Invoke(invoke) => {
                    write!(f, "Invoke({:?}({:?}))", invoke.target, invoke.members)
                },

                ElementKind::Construct(construct) => {
                    write!(f, "Constructor({:?} | {:?})", construct.target, construct.members)
                },

                ElementKind::Symbolize(symbol) => write!(f, "+ {:?}", symbol),
            }
        }
    }
}

impl<'symbol> Debug for Symbol<'symbol> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(
                f,
                "{:#?} \n{}\n{}",
                self.kind,
                format!("Specification: {:#?}", self.specifier).indent(),
                format!("Generics: {:#?}", self.generic).indent(),
            )
        } else {
            write!(f, "{:?} | {:?} -> {:?}", self.kind, self.span, self.specifier)
        }
    }
}

impl<'symbol> Debug for SymbolKind<'symbol> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            match self {
                SymbolKind::Inclusion(inclusion) => {
                    write!(f, "Inclusion({:#?})", inclusion.target)
                }
                SymbolKind::Extension(extension) => {
                    write!(f, "Extension(")?;

                    if let Some(extension) = &extension.extension {
                        write!(f, "{:#?}, ", extension)?;
                    }

                    write!(f, "{:#?}, {:#?})", extension.target, extension.members)
                }
                SymbolKind::Binding(binding) => {
                    write!(
                        f,
                        "Binding({} {:#?}",
                        if binding.constant { "Constant" } else { "Variable" },
                        binding.target
                    )?;

                    if let Some(annotation) = &binding.annotation {
                        write!(f, " : {:#?}", annotation)?;
                    }

                    if let Some(value) = &binding.value {
                        write!(f, " = {:#?}", value)?;
                    }

                    write!(f, ")")
                }
                SymbolKind::Structure(structure) => {
                    write!(f, "Structure({:#?} {:#?})", structure.target, structure.members)
                }
                SymbolKind::Enumeration(enumeration) => {
                    write!(f, "Enumeration({:#?} {:#?})", enumeration.target, enumeration.members)
                }
                SymbolKind::Method(method) => {
                    write!(
                        f,
                        "Method({:#?} {:#?}{} -> {:#?} : {:#?})",
                        method.target,
                        method.members,
                        if method.variadic {
                            "- Variadic"
                        } else {
                            ""
                        },
                        method.output,
                        method.body)
                }
                SymbolKind::Module(module) => {
                    write!(f, "Module({:#?})", module.target)
                }
                SymbolKind::Preference(preference) => {
                    write!(f, "Preference({:#?}, {:#?})", preference.target, preference.value)
                }
            }
        } else {
            match self {
                SymbolKind::Inclusion(inclusion) => {
                    write!(f, "Inclusion({:?})", inclusion.target)
                }
                SymbolKind::Extension(extension) => {
                    write!(f, "Extension(")?;

                    if let Some(extension) = &extension.extension {
                        write!(f, "{:?}, ", extension)?;
                    }

                    write!(f, "{:?}, {:?})", extension.target, extension.members)
                }
                SymbolKind::Binding(binding) => {
                    write!(
                        f,
                        "Binding({} {:?}",
                        if binding.constant { "Constant" } else { "Variable" },
                        binding.target
                    )?;

                    if let Some(annotation) = &binding.annotation {
                        write!(f, " : {:?}", annotation)?;
                    }

                    if let Some(value) = &binding.value {
                        write!(f, " = {:?}", value)?;
                    }

                    write!(f, ")")
                }
                SymbolKind::Structure(structure) => {
                    write!(f, "Structure({:?} {:?})", structure.target, structure.members)
                }
                SymbolKind::Enumeration(enumeration) => {
                    write!(f, "Enumeration({:?} {:?})", enumeration.target, enumeration.members)
                }
                SymbolKind::Method(method) => {
                    write!(
                        f,
                        "Method({:?} {:?}{} -> {:?} : {:?})",
                        method.target,
                        method.members,
                        if method.variadic {
                            "- Variadic"
                        } else {
                            ""
                        },
                        method.output,
                        method.body)
                }
                SymbolKind::Module(module) => {
                    write!(f, "Module({:?})", module.target)
                }
                SymbolKind::Preference(preference) => {
                    write!(f, "Preference({:?}, {:?})", preference.target, preference.value)
                }
            }
        }
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
            
            ElementKind::Closure(closure) => {
                discriminant(self).hash(state);
                closure.hash(state);
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
            
            ElementKind::Closure(closure) => ElementKind::Closure(closure.clone()),

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
            kind: self.kind.clone(),
            span: self.span.clone(),
            scope: self.scope.clone(),
            generic: self.generic.clone(),
            specifier: self.specifier.clone(),
        }
    }
}

impl<'symbol> Display for Symbol<'symbol> {
    fn fmt(&self, f: &mut Formatter<'_>) -> format::Result {
        write!(f, "{:?}", self)
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
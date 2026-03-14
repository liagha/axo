use broccli::{Color, TextStyle};
use crate::format::Show;
use {
    super::{Element, ElementKind, Symbol, SymbolKind},
    crate::{
        data::memory::discriminant,
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
        self.kind.format(verbosity)
    }
}

impl<'element> Show<'element> for ElementKind<'element> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'element> {
        match verbosity {
            0 => {
                match self {
                    ElementKind::Literal(literal) => {
                        literal.format(verbosity)
                    }
                    
                    ElementKind::Delimited(delimited) => {
                        delimited.format(verbosity)
                    }

                    ElementKind::Binary(binary) => {
                        binary.format(verbosity)
                    }
                    ElementKind::Unary(unary) => {
                        unary.format(verbosity)
                    }

                    ElementKind::Index(index) => {
                        index.format(verbosity)
                    }
                    
                    ElementKind::Invoke(invoke) => {
                        invoke.format(verbosity)
                    }

                    ElementKind::Construct(construct) => {
                        format!("Construct({})", construct.format(verbosity)).into()
                    }

                    ElementKind::Symbolize(symbol) => {
                        symbol.format(verbosity)
                    },
                }
            }

            _ => {
                self.format(verbosity - 1)
            }
        }
    }
}

impl<'symbol> Show<'symbol> for Symbol<'symbol> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'symbol> {
        match verbosity {
            0 => {
                format!(
                    "{}. {}{}",
                    self.identity.colorize(Color::Blue),
                    self.kind.format(verbosity),
                    if self.scope.is_empty() {
                        "".into()
                    } else {
                        format!("\n{}", self.scope.format(verbosity)).indent(verbosity)
                    },
                )
            }

            1 => {
                format!(
                    "{}. {}: {:?}{}",
                    self.identity.colorize(Color::Blue),
                    self.kind.format(verbosity),
                    self.visibility,
                    if self.scope.is_empty() {
                        "".into()
                    } else {
                        format!("\n{}", self.scope.format(verbosity)).indent(verbosity)
                    },
                )
            }

            _ => {
                self.format(verbosity - 1).to_string()
            }
        }.into()
    }
}

impl<'symbol> Show<'symbol> for SymbolKind<'symbol> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'symbol> {
        match verbosity {
            _ => {
                match self {
                    SymbolKind::Binding(binding) => {
                        binding.format(verbosity)
                    }
                    SymbolKind::Structure(structure) => {
                        structure.format(verbosity)
                    }
                    SymbolKind::Union(union) => {
                        union.format(verbosity)
                    }
                    SymbolKind::Function(function) => {
                        function.format(verbosity)
                    }
                    SymbolKind::Module(module) => {
                        module.format(verbosity)
                    }
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

impl<'symbol> Spanned<'symbol> for Symbol<'symbol> {
    #[track_caller]
    fn borrow_span(&self) -> Span<'symbol> {
        self.span
    }

    #[track_caller]
    fn span(self) -> Span<'symbol> {
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
            identity: self.identity,
            kind: self.kind.clone(),
            span: self.span.clone(),
            reference: self.reference,
            typing: self.typing.clone(),
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
            identity: self.identity,
            usages: self.usages.clone(),
            kind: self.kind.clone(),
            span: self.span.clone(),
            scope: self.scope.clone(),
            visibility: self.visibility.clone(),
            typing: self.typing.clone(),
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
        self.identity.cmp(&other.identity)
    }
}

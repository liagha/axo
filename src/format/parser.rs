use {
    broccli::{Color, TextStyle},
    crate::{
        data::Str,
        format::{Show, Verbosity},
        parser::{Element, ElementKind, Symbol, SymbolKind},
    },
};

impl<'element> Show<'element> for Element<'element> {
    fn format(&self, verbosity: Verbosity) -> Str<'element> {
        self.kind.format(verbosity)
    }
}

impl<'element> Show<'element> for ElementKind<'element> {
    fn format(&self, verbosity: Verbosity) -> Str<'element> {
        match verbosity {
            Verbosity::Minimal => {
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
                self.format(verbosity.fallback())
            }
        }
    }
}

impl<'symbol> Show<'symbol> for Symbol<'symbol> {
    fn format(&self, verbosity: Verbosity) -> Str<'symbol> {
        match verbosity {
            Verbosity::Minimal => {
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

            Verbosity::Detailed => {
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
                self.format(verbosity.fallback()).to_string()
            }
        }.into()
    }
}

impl<'symbol> Show<'symbol> for SymbolKind<'symbol> {
    fn format(&self, verbosity: Verbosity) -> Str<'symbol> {
        match verbosity {
            _ => {
                match self {
                    SymbolKind::Binding(binding) => {
                        binding.format(verbosity)
                    }
                    SymbolKind::Structure(structure) => {
                        format!("Structure({})", structure.format(verbosity)).into()
                    }
                    SymbolKind::Union(union) => {
                        format!("Union({})", union.format(verbosity)).into()
                    }
                    SymbolKind::Enumeration(enumeration) => {
                        format!("Enumeration({})", enumeration.format(verbosity)).into()
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

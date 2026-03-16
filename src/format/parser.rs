use {
    broccli::{Color, TextStyle},
    crate::{
        data::Str,
        format::Show,
        parser::{Element, ElementKind, Symbol, SymbolKind},
    },
};

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

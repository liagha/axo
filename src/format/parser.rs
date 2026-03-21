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
        if verbosity == Verbosity::Off {
            return "".into();
        }

        match self {
            ElementKind::Literal(literal) => literal.format(verbosity),
            ElementKind::Delimited(delimited) => delimited.format(verbosity),
            ElementKind::Binary(binary) => binary.format(verbosity),
            ElementKind::Unary(unary) => unary.format(verbosity),
            ElementKind::Index(index) => index.format(verbosity),
            ElementKind::Invoke(invoke) => invoke.format(verbosity),
            ElementKind::Symbolize(symbol) => symbol.format(verbosity),
            ElementKind::Construct(construct) => match verbosity {
                Verbosity::Minimal => construct.format(verbosity),
                Verbosity::Detailed => format!("Construct({})", construct.format(verbosity)).into(),
                Verbosity::Debug => format!(
                    "Construct {{\n{}\n}}",
                    construct.format(verbosity).indent(verbosity)
                ).into(),
                _ => "".into(),
            },
        }
    }
}

impl<'symbol> Show<'symbol> for Symbol<'symbol> {
    fn format(&self, verbosity: Verbosity) -> Str<'symbol> {
        match verbosity {
            Verbosity::Off => "".into(),
            Verbosity::Minimal => format!(
                "{} {}",
                self.identity.colorize(Color::Blue),
                self.kind.format(verbosity)
            ).into(),
            Verbosity::Detailed => format!(
                "Symbol({}. {}: {:?})",
                self.identity.colorize(Color::Blue),
                self.kind.format(verbosity),
                self.visibility,
            ).into(),
            Verbosity::Debug => format!(
                "Symbol {{\n{},\n{},\n{}\n}}",
                format!("identity: {}", self.identity.colorize(Color::Blue)).indent(verbosity),
                format!("kind: {}", self.kind.format(verbosity)).indent(verbosity),
                format!("visibility: {:?}", self.visibility).indent(verbosity),
            ).into(),
        }
    }
}

impl<'symbol> Show<'symbol> for SymbolKind<'symbol> {
    fn format(&self, verbosity: Verbosity) -> Str<'symbol> {
        if verbosity == Verbosity::Off {
            return "".into();
        }

        match self {
            SymbolKind::Binding(binding) => binding.format(verbosity),
            SymbolKind::Function(function) => function.format(verbosity),
            SymbolKind::Module(module) => module.format(verbosity),
            SymbolKind::Structure(structure) => match verbosity {
                Verbosity::Minimal => format!("struct {}", structure.format(verbosity)).into(),
                Verbosity::Detailed => format!("Structure({})", structure.format(verbosity)).into(),
                Verbosity::Debug => format!("Structure {{\n{}\n}}", structure.format(verbosity).indent(verbosity)).into(),
                _ => "".into()
            },
            SymbolKind::Union(union) => match verbosity {
                Verbosity::Minimal => format!("union {}", union.format(verbosity)).into(),
                Verbosity::Detailed => format!("Union({})", union.format(verbosity)).into(),
                Verbosity::Debug => format!("Union {{\n{}\n}}", union.format(verbosity).indent(verbosity)).into(),
                _ => "".into()
            },
        }
    }
}

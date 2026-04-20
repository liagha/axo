use crate::{
    format::{Show, Stencil},
    parser::{Element, ElementKind, Symbol, SymbolKind},
};

impl<'element> Show<'element> for Element<'element> {
    fn format(&self, config: Stencil) -> Stencil {
        config
            .clone()
            .new("Element")
            .field("kind", self.kind.format(config.clone()))
    }
}

impl<'element> Show<'element> for ElementKind<'element> {
    fn format(&self, config: Stencil) -> Stencil {
        let base = config.clone().new("ElementKind");
        match self {
            ElementKind::Literal(literal) => base
                .variant("Literal")
                .field("value", literal.format(config.clone())),
            ElementKind::Delimited(delimited) => base
                .variant("Delimited")
                .field("value", delimited.format(config.clone())),
            ElementKind::Binary(binary) => base
                .variant("Binary")
                .field("value", binary.format(config.clone())),
            ElementKind::Unary(unary) => base
                .variant("Unary")
                .field("value", unary.format(config.clone())),
            ElementKind::Index(index) => base
                .variant("Index")
                .field("value", index.format(config.clone())),
            ElementKind::Invoke(invoke) => base
                .variant("Invoke")
                .field("value", invoke.format(config.clone())),
            ElementKind::Symbolize(symbol) => base
                .variant("Symbolize")
                .field("value", symbol.format(config.clone())),
            ElementKind::Construct(construct) => base
                .variant("Construct")
                .field("value", construct.format(config.clone())),
        }
    }
}

impl<'symbol> Show<'symbol> for Symbol<'symbol> {
    fn format(&self, config: Stencil) -> Stencil {
        config
            .clone()
            .new("Symbol")
            .field("kind", self.kind.format(config.clone()))
    }
}

impl<'symbol> Show<'symbol> for SymbolKind<'symbol> {
    fn format(&self, config: Stencil) -> Stencil {
        let base = config.clone().new("SymbolKind");
        match self {
            SymbolKind::Binding(binding) => base
                .variant("Binding")
                .field("value", binding.format(config.clone())),
            SymbolKind::Function(function) => base
                .variant("Function")
                .field("value", function.format(config.clone())),
            SymbolKind::Module(module) => base
                .variant("Module")
                .field("value", module.format(config.clone())),
            SymbolKind::Structure(structure) => base
                .variant("Structure")
                .field("value", structure.format(config.clone())),
            SymbolKind::Union(union) => base
                .variant("Union")
                .field("value", union.format(config.clone())),
        }
    }
}

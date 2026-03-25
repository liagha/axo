use {
    crate::{
        format::{Show, Stencil},
        parser::{Element, ElementKind, Symbol, SymbolKind},
    },
};

impl<'element> Show<'element> for Element<'element> {
    fn format(&self, config: Stencil) -> Stencil {
        self.kind.format(config)
    }
}

impl<'element> Show<'element> for ElementKind<'element> {
    fn format(&self, config: Stencil) -> Stencil {
        match self {
            ElementKind::Literal(literal) => literal.format(config),
            ElementKind::Delimited(delimited) => delimited.format(config),
            ElementKind::Binary(binary) => binary.format(config),
            ElementKind::Unary(unary) => unary.format(config),
            ElementKind::Index(index) => index.format(config),
            ElementKind::Invoke(invoke) => invoke.format(config),
            ElementKind::Symbolize(symbol) => symbol.format(config),
            ElementKind::Construct(construct) => config.clone().new("ElementKind").variant("Construct").field("value", construct.format(config.clone())),
        }
    }
}

impl<'symbol> Show<'symbol> for Symbol<'symbol> {
    fn format(&self, config: Stencil) -> Stencil {
        config.clone().new("Symbol")
            .field("kind", self.kind.format(config.clone()))
            .field("visibility", format!("{:?}", self.visibility))
    }
}

impl<'symbol> Show<'symbol> for SymbolKind<'symbol> {
    fn format(&self, config: Stencil) -> Stencil {
        match self {
            SymbolKind::Binding(binding) => {
                config.clone().new("SymbolKind").variant("Binding").field("binding", binding.format(config.clone()))
            },
            SymbolKind::Function(function) => {
                config.clone().new("SymbolKind").variant("Function").field("function", function.format(config.clone()))
            },
            SymbolKind::Module(module) => {
                config.clone().new("SymbolKind").variant("Module").field("module", module.format(config.clone()))
            },
            SymbolKind::Structure(structure) => {
                config.clone().new("SymbolKind").variant("Structure").field("structure", structure.format(config.clone()))
            },
            SymbolKind::Union(union) => {
                config.clone().new("SymbolKind").variant("Union").field("union", union.format(config.clone()))
            },
        }
    }
}

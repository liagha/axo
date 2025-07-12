use {
    super::{
        Element, ElementKind,
        Symbol, SymbolKind
    },

    crate::{
        memory::discriminant,

        hash::{
            Hash, Hasher
        },

        axo_data::tree::Tree,
    },
};

impl Hash for Element {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
    }
}

impl Hash for ElementKind {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ElementKind::Literal(kind) => {
                discriminant(self).hash(state);
                kind.hash(state);
            }
            ElementKind::Identifier(name) => {
                discriminant(self).hash(state);
                name.hash(state);
            }
            ElementKind::Procedural(element) => {
                discriminant(self).hash(state);
                element.hash(state);
            }

            ElementKind::Group(group) => {
                discriminant(self).hash(state);
                group.hash(state);
            }
            ElementKind::Sequence(sequence) => {
                discriminant(self).hash(state);
                sequence.hash(state);
            }
            ElementKind::Collection(collection) => {
                discriminant(self).hash(state);
                collection.hash(state);
            }
            ElementKind::Series(series) => {
                discriminant(self).hash(state);
                series.hash(state);
            }
            ElementKind::Bundle(bundle) => {
                discriminant(self).hash(state);
                bundle.hash(state);
            }
            ElementKind::Scope(scope) => {
                discriminant(self).hash(state);
                scope.hash(state);
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

            ElementKind::Label(label) => {
                discriminant(self).hash(state);
                label.hash(state);
            }
            ElementKind::Access(access) => {
                discriminant(self).hash(state);
                access.hash(state);
            }
            ElementKind::Index(index) => {
                discriminant(self).hash(state);
                index.hash(state);
            }
            ElementKind::Invoke(invoke) => {
                discriminant(self).hash(state);
                invoke.hash(state);
            }
            ElementKind::Locate(tree) => {
                discriminant(self).hash(state);
                tree.hash(state);
            }

            ElementKind::Conditioned(conditioned) => {
                discriminant(self).hash(state);
                conditioned.hash(state);
            }
            ElementKind::Repeat(repeat) => {
                discriminant(self).hash(state);
                repeat.hash(state);
            }
            ElementKind::Walk(walk) => {
                discriminant(self).hash(state);
                walk.hash(state);
            }
            ElementKind::Map(map) => {
                discriminant(self).hash(state);
                map.hash(state);
            }

            ElementKind::Symbolize(symbol) => {
                discriminant(self).hash(state);
                symbol.hash(state);
            }
            ElementKind::Assign(assign) => {
                discriminant(self).hash(state);
                assign.hash(state);
            }

            ElementKind::Produce(element) => {
                discriminant(self).hash(state);
                element.hash(state);
            }
            ElementKind::Abort(element) => {
                discriminant(self).hash(state);
                element.hash(state);
            }
            ElementKind::Pass(element) => {
                discriminant(self).hash(state);
                element.hash(state);
            }
        }
    }
}

impl PartialEq for Element {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl PartialEq for ElementKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ElementKind::Literal(a), ElementKind::Literal(b)) => a == b,
            (ElementKind::Identifier(a), ElementKind::Identifier(b)) => a == b,
            (ElementKind::Procedural(a), ElementKind::Procedural(b)) => a == b,

            (ElementKind::Group(a), ElementKind::Group(b)) => a == b,
            (ElementKind::Sequence(a), ElementKind::Sequence(b)) => a == b,
            (ElementKind::Collection(a), ElementKind::Collection(b)) => a == b,
            (ElementKind::Series(a), ElementKind::Series(b)) => a == b,
            (ElementKind::Bundle(a), ElementKind::Bundle(b)) => a == b,
            (ElementKind::Scope(a), ElementKind::Scope(b)) => a == b,
            (ElementKind::Construct(a), ElementKind::Construct(b)) => a == b,

            (ElementKind::Binary(a), ElementKind::Binary(b)) => a == b,
            (ElementKind::Unary(a), ElementKind::Unary(b)) => a == b,

            (ElementKind::Label(a), ElementKind::Label(b)) => a == b,
            (ElementKind::Access(a), ElementKind::Access(b)) => a == b,
            (ElementKind::Index(a), ElementKind::Index(b)) => a == b,
            (ElementKind::Invoke(a), ElementKind::Invoke(b)) => a == b,
            (ElementKind::Locate(a), ElementKind::Locate(b)) => a == b,

            (ElementKind::Conditioned(a), ElementKind::Conditioned(b)) => a == b,
            (ElementKind::Repeat(a), ElementKind::Repeat(b)) => a == b,
            (ElementKind::Walk(a), ElementKind::Walk(b)) => a == b,
            (ElementKind::Map(a), ElementKind::Map(b)) => a == b,

            (ElementKind::Symbolize(a), ElementKind::Symbolize(b)) => a == b,
            (ElementKind::Assign(a), ElementKind::Assign(b)) => a == b,

            (ElementKind::Produce(a), ElementKind::Produce(b)) => a == b,
            (ElementKind::Abort(a), ElementKind::Abort(b)) => a == b,
            (ElementKind::Pass(a), ElementKind::Pass(b)) => a == b,

            _ => false,
        }
    }
}

impl Clone for Element {
    fn clone(&self) -> Self {
        Element {
            kind: self.kind.clone(),
            span: self.span.clone(),
        }
    }
}

impl Clone for ElementKind {
    fn clone(&self) -> Self {
        match self {
            ElementKind::Literal(kind) => ElementKind::Literal(kind.clone()),
            ElementKind::Identifier(name) => ElementKind::Identifier(name.clone()),
            ElementKind::Procedural(element) => ElementKind::Procedural(element.clone()),

            ElementKind::Group(group) => ElementKind::Group(group.clone()),
            ElementKind::Sequence(sequence) => ElementKind::Sequence(sequence.clone()),
            ElementKind::Collection(collection) => ElementKind::Collection(collection.clone()),
            ElementKind::Series(series) => ElementKind::Series(series.clone()),
            ElementKind::Bundle(bundle) => ElementKind::Bundle(bundle.clone()),
            ElementKind::Scope(scope) => ElementKind::Scope(scope.clone()),
            ElementKind::Construct(construct) => ElementKind::Construct(construct.clone()),

            ElementKind::Binary(binary) => ElementKind::Binary(binary.clone()),
            ElementKind::Unary(unary) => ElementKind::Unary(unary.clone()),

            ElementKind::Label(label) => ElementKind::Label(label.clone()),
            ElementKind::Access(access) => ElementKind::Access(access.clone()),
            ElementKind::Index(index) => ElementKind::Index(index.clone()),
            ElementKind::Invoke(invoke) => ElementKind::Invoke(invoke.clone()),
            ElementKind::Locate(tree) => ElementKind::Locate(tree.clone()),

            ElementKind::Conditioned(conditioned) => ElementKind::Conditioned(conditioned.clone()),
            ElementKind::Repeat(repeat) => ElementKind::Repeat(repeat.clone()),
            ElementKind::Walk(walk) => ElementKind::Walk(walk.clone()),
            ElementKind::Map(map) => ElementKind::Map(map.clone()),

            ElementKind::Symbolize(symbol) => ElementKind::Symbolize(symbol.clone()),
            ElementKind::Assign(assign) => ElementKind::Assign(assign.clone()),

            ElementKind::Produce(element) => ElementKind::Produce(element.clone()),
            ElementKind::Abort(element) => ElementKind::Abort(element.clone()),
            ElementKind::Pass(element) => ElementKind::Pass(element.clone()),
        }
    }
}

impl Eq for Element {}

impl Eq for ElementKind {}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
    }
}

impl Hash for SymbolKind {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            SymbolKind::Formation(formation) => {
                discriminant(self).hash(state);
                formation.hash(state);
            }
            SymbolKind::Inclusion(inclusion) => {
                discriminant(self).hash(state);
                inclusion.hash(state);
            }
            SymbolKind::Implementation(implementation) => {
                discriminant(self).hash(state);
                implementation.hash(state);
            }
            SymbolKind::Interface(interface) => {
                discriminant(self).hash(state);
                interface.hash(state);
            }
            SymbolKind::Binding(binding) => {
                discriminant(self).hash(state);
                binding.hash(state);
            }
            SymbolKind::Structure(structure) => {
                discriminant(self).hash(state);
                structure.hash(state);
            }
            SymbolKind::Enumeration(enumeration) => {
                discriminant(self).hash(state);
                enumeration.hash(state);
            }
            SymbolKind::Function(function) => {
                discriminant(self).hash(state);
                function.hash(state);
            }
        }
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl PartialEq for SymbolKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SymbolKind::Formation(f1), SymbolKind::Formation(f2)) => f1 == f2,
            (SymbolKind::Inclusion(a), SymbolKind::Inclusion(b)) => a == b,
            (SymbolKind::Implementation(a), SymbolKind::Implementation(b)) => a == b,
            (SymbolKind::Interface(a), SymbolKind::Interface(b)) => a == b,
            (SymbolKind::Binding(a), SymbolKind::Binding(b)) => a == b,
            (SymbolKind::Structure(a), SymbolKind::Structure(b)) => a == b,
            (SymbolKind::Enumeration(a), SymbolKind::Enumeration(b)) => a == b,
            (SymbolKind::Function(a), SymbolKind::Function(b)) => a == b,
            _ => false,
        }
    }
}

impl Clone for Symbol {
    fn clone(&self) -> Self {
        Symbol {
            kind: self.kind.clone(),
            span: self.span.clone(),
        }
    }
}

impl Clone for SymbolKind {
    fn clone(&self) -> Self {
        match self {
            SymbolKind::Formation(formation) => SymbolKind::Formation(formation.clone()),
            SymbolKind::Inclusion(inclusion) => SymbolKind::Inclusion(inclusion.clone()),
            SymbolKind::Implementation(implementation) => SymbolKind::Implementation(implementation.clone()),
            SymbolKind::Interface(interface) => SymbolKind::Interface(interface.clone()),
            SymbolKind::Binding(binding) => SymbolKind::Binding(binding.clone()),
            SymbolKind::Structure(structure) => SymbolKind::Structure(structure.clone()),
            SymbolKind::Enumeration(enumeration) => SymbolKind::Enumeration(enumeration.clone()),
            SymbolKind::Function(function) => SymbolKind::Function(function.clone()),
        }
    }
}

impl Eq for Symbol {}

impl Eq for SymbolKind {}
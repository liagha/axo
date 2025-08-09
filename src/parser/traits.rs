use {
    super::{
        Element, ElementKind,
    },

    crate::{
        data::memory::discriminant,

        internal::hash::{
            Hash, Hasher
        },
    },
};

impl<'element> Hash for Element<'element> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
    }
}

impl<'element> Hash for ElementKind<'element> {
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
            ElementKind::Block(block) => {
                discriminant(self).hash(state);
                block.hash(state);
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

            ElementKind::Conditional(conditioned) => {
                discriminant(self).hash(state);
                conditioned.hash(state);
            }
            ElementKind::Repeat(repeat) => {
                discriminant(self).hash(state);
                repeat.hash(state);
            }
            ElementKind::Iterate(walk) => {
                discriminant(self).hash(state);
                walk.hash(state);
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

impl<'element> PartialEq for Element<'element> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl<'element> PartialEq for ElementKind<'element> {
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
            (ElementKind::Block(a), ElementKind::Block(b)) => a == b,
            (ElementKind::Construct(a), ElementKind::Construct(b)) => a == b,

            (ElementKind::Binary(a), ElementKind::Binary(b)) => a == b,
            (ElementKind::Unary(a), ElementKind::Unary(b)) => a == b,

            (ElementKind::Label(a), ElementKind::Label(b)) => a == b,
            (ElementKind::Access(a), ElementKind::Access(b)) => a == b,
            (ElementKind::Index(a), ElementKind::Index(b)) => a == b,
            (ElementKind::Invoke(a), ElementKind::Invoke(b)) => a == b,

            (ElementKind::Conditional(a), ElementKind::Conditional(b)) => a == b,
            (ElementKind::Repeat(a), ElementKind::Repeat(b)) => a == b,
            (ElementKind::Iterate(a), ElementKind::Iterate(b)) => a == b,

            (ElementKind::Symbolize(a), ElementKind::Symbolize(b)) => a == b,
            (ElementKind::Assign(a), ElementKind::Assign(b)) => a == b,

            (ElementKind::Produce(a), ElementKind::Produce(b)) => a == b,
            (ElementKind::Abort(a), ElementKind::Abort(b)) => a == b,
            (ElementKind::Pass(a), ElementKind::Pass(b)) => a == b,

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
            ElementKind::Identifier(name) => ElementKind::Identifier(name.clone()),
            ElementKind::Procedural(element) => ElementKind::Procedural(element.clone()),

            ElementKind::Group(group) => ElementKind::Group(group.clone()),
            ElementKind::Sequence(sequence) => ElementKind::Sequence(sequence.clone()),
            ElementKind::Collection(collection) => ElementKind::Collection(collection.clone()),
            ElementKind::Series(series) => ElementKind::Series(series.clone()),
            ElementKind::Bundle(bundle) => ElementKind::Bundle(bundle.clone()),
            ElementKind::Block(block) => ElementKind::Block(block.clone()),
            ElementKind::Construct(construct) => ElementKind::Construct(construct.clone()),

            ElementKind::Binary(binary) => ElementKind::Binary(binary.clone()),
            ElementKind::Unary(unary) => ElementKind::Unary(unary.clone()),

            ElementKind::Label(label) => ElementKind::Label(label.clone()),
            ElementKind::Access(access) => ElementKind::Access(access.clone()),
            ElementKind::Index(index) => ElementKind::Index(index.clone()),
            ElementKind::Invoke(invoke) => ElementKind::Invoke(invoke.clone()),

            ElementKind::Conditional(conditioned) => ElementKind::Conditional(conditioned.clone()),
            ElementKind::Repeat(repeat) => ElementKind::Repeat(repeat.clone()),
            ElementKind::Iterate(walk) => ElementKind::Iterate(walk.clone()),

            ElementKind::Symbolize(symbol) => ElementKind::Symbolize(symbol.clone()),
            ElementKind::Assign(assign) => ElementKind::Assign(assign.clone()),

            ElementKind::Produce(element) => ElementKind::Produce(element.clone()),
            ElementKind::Abort(element) => ElementKind::Abort(element.clone()),
            ElementKind::Pass(element) => ElementKind::Pass(element.clone()),
        }
    }
}

impl<'element> Eq for Element<'element> {}

impl<'element> Eq for ElementKind<'element> {}
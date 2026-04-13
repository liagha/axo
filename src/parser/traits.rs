use crate::{
    data::memory::discriminant,
    internal::{
        hash::{Hash, Hasher},
        operation::Ordering,
    },
    parser::{Element, ElementKind, Symbol},
    tracker::{Span, Spanned},
};

impl<'element> Hash for Element<'element> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
    }
}

impl<'element> Spanned<'element> for Element<'element> {
    #[track_caller]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'symbol> Spanned<'symbol> for Symbol<'symbol> {
    #[track_caller]
    fn span(&self) -> Span {
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
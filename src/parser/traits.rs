use {
    crate::{
        parser::{Element, ElementKind, Symbol},
        data::{
            memory::discriminant,
        },
        internal::{
            cache::{Encode, Decode},
            hash::{Hash, Hasher},
            operation::Ordering,
        },
        tracker::{Span, Spanned},
    },
};
use crate::data::{Aggregate, Binary, Binding, Function, Identity, Index, Invoke, Module, Unary};
use crate::internal::hash::Set;
use crate::parser::{SymbolKind, Visibility};
use crate::resolver::{Scope, Type};
use crate::scanner::Token;


impl<'element> Hash for Element<'element> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
    }
}

impl<'element> Spanned<'element> for Element<'element> {
    #[track_caller]
    fn span(&self) -> Span<'element> {
        self.span
    }
}

impl<'symbol> Spanned<'symbol> for Symbol<'symbol> {
    #[track_caller]
    fn span(&self) -> Span<'symbol> {
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

impl<'element> Encode for Element<'element> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.identity.encode(buffer);
        self.kind.encode(buffer);
        self.span.encode(buffer);
        self.reference.encode(buffer);
        self.typing.encode(buffer);
    }
}

impl<'element> Decode<'element> for Element<'element> {
    fn decode(buffer: &'element [u8], cursor: &mut usize) -> Self {
        Element {
            identity: Identity::decode(buffer, cursor),
            kind: ElementKind::decode(buffer, cursor),
            span: Span::decode(buffer, cursor),
            reference: Option::decode(buffer, cursor),
            typing: Type::decode(buffer, cursor),
        }
    }
}

impl<'element> Encode for ElementKind<'element> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        match self {
            ElementKind::Literal(v) => {
                buffer.push(0);
                v.encode(buffer);
            }
            ElementKind::Delimited(v) => {
                buffer.push(1);
                v.encode(buffer);
            }
            ElementKind::Unary(v) => {
                buffer.push(2);
                v.encode(buffer);
            }
            ElementKind::Binary(v) => {
                buffer.push(3);
                v.encode(buffer);
            }
            ElementKind::Index(v) => {
                buffer.push(4);
                v.encode(buffer);
            }
            ElementKind::Invoke(v) => {
                buffer.push(5);
                v.encode(buffer);
            }
            ElementKind::Construct(v) => {
                buffer.push(6);
                v.encode(buffer);
            }
            ElementKind::Symbolize(v) => {
                buffer.push(7);
                v.encode(buffer);
            }
        }
    }
}

impl<'element> Decode<'element> for ElementKind<'element> {
    fn decode(buffer: &'element [u8], cursor: &mut usize) -> Self {
        let tag = buffer[*cursor];
        *cursor += 1;
        match tag {
            0 => ElementKind::Literal(Token::decode(buffer, cursor)),
            1 => ElementKind::Delimited(Box::decode(buffer, cursor)),
            2 => ElementKind::Unary(Unary::decode(buffer, cursor)),
            3 => ElementKind::Binary(Binary::decode(buffer, cursor)),
            4 => ElementKind::Index(Index::decode(buffer, cursor)),
            5 => ElementKind::Invoke(Invoke::decode(buffer, cursor)),
            6 => ElementKind::Construct(Aggregate::decode(buffer, cursor)),
            7 => ElementKind::Symbolize(Box::decode(buffer, cursor)),
            _ => panic!(),
        }
    }
}

impl<'symbol> Encode for Symbol<'symbol> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.identity.encode(buffer);
        self.usages.encode(buffer);
        self.kind.encode(buffer);
        self.span.encode(buffer);
        self.scope.encode(buffer);
        self.visibility.encode(buffer);
        self.typing.encode(buffer);
    }
}

impl<'symbol> Decode<'symbol> for Symbol<'symbol> {
    fn decode(buffer: &'symbol [u8], cursor: &mut usize) -> Self {
        Symbol {
            identity: Identity::decode(buffer, cursor),
            usages: Set::decode(buffer, cursor),
            kind: SymbolKind::decode(buffer, cursor),
            span: Span::decode(buffer, cursor),
            scope: Scope::decode(buffer, cursor),
            visibility: Visibility::decode(buffer, cursor),
            typing: Type::decode(buffer, cursor),
        }
    }
}

impl<'symbol> Encode for SymbolKind<'symbol> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        match self {
            SymbolKind::Binding(v) => {
                buffer.push(0);
                v.encode(buffer);
            }
            SymbolKind::Structure(v) => {
                buffer.push(1);
                v.encode(buffer);
            }
            SymbolKind::Union(v) => {
                buffer.push(2);
                v.encode(buffer);
            }
            SymbolKind::Function(v) => {
                buffer.push(3);
                v.encode(buffer);
            }
            SymbolKind::Module(v) => {
                buffer.push(4);
                v.encode(buffer);
            }
        }
    }
}

impl<'symbol> Decode<'symbol> for SymbolKind<'symbol> {
    fn decode(buffer: &'symbol [u8], cursor: &mut usize) -> Self {
        let tag = buffer[*cursor];
        *cursor += 1;
        match tag {
            0 => SymbolKind::Binding(Binding::decode(buffer, cursor)),
            1 => SymbolKind::Structure(Aggregate::decode(buffer, cursor)),
            2 => SymbolKind::Union(Aggregate::decode(buffer, cursor)),
            3 => SymbolKind::Function(Function::decode(buffer, cursor)),
            4 => SymbolKind::Module(Module::decode(buffer, cursor)),
            _ => panic!(),
        }
    }
}

impl Encode for Visibility {
    fn encode(&self, buffer: &mut Vec<u8>) {
        match self {
            Visibility::Public => buffer.push(0),
            Visibility::Private => buffer.push(1),
        }
    }
}

impl<'symbol> Decode<'symbol> for Visibility {
    fn decode(buffer: &'symbol [u8], cursor: &mut usize) -> Self {
        let tag = buffer[*cursor];
        *cursor += 1;
        match tag {
            0 => Visibility::Public,
            1 => Visibility::Private,
            _ => panic!(),
        }
    }
}
use orbyte::Orbyte;
use crate::resolver::{ErrorKind, ResolveError};
use crate::{
    data::Identity,
    internal::hash::Set,
    parser::{Element, ElementKind, Symbol, SymbolKind},
    resolver::{Resolvable, Resolver},
    scanner::{Token, TokenKind},
};

#[derive(Clone, Orbyte)]
pub struct Scope {
    pub symbols: Set<Identity>,
    pub parent: Option<Identity>,
}

impl Scope {
    pub fn new(parent: Option<Identity>) -> Self {
        Self {
            symbols: Set::new(),
            parent,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    pub fn insert(&mut self, identity: Identity) {
        self.symbols.remove(&identity);
        self.symbols.insert(identity);
    }

    pub fn extend(&mut self, items: Vec<Identity>) {
        self.symbols.extend(items);
    }

    pub fn replace(&mut self, old: Identity, new: Identity) {
        if self.symbols.remove(&old) {
            self.symbols.insert(new);
        }
    }

    pub fn remove(&mut self, identity: &Identity) -> bool {
        self.symbols.remove(identity)
    }

    pub fn has(&self, target: Identity) -> bool {
        self.symbols.contains(&target)
    }

    pub fn fits<'a>(query: &Element<'a>, symbol: &Symbol<'a>) -> bool {
        match (&query.kind, &symbol.kind) {
            (
                ElementKind::Literal(Token {
                    kind: TokenKind::Identifier(_),
                    ..
                }),
                _,
            ) => true,
            (ElementKind::Invoke(invoke), SymbolKind::Function(function)) => {
                invoke.members.len() == function.members.len()
            }
            (ElementKind::Construct(construct), SymbolKind::Structure(structure)) => {
                let fields = structure
                    .members
                    .iter()
                    .filter(|member| member.is_instance())
                    .filter_map(|member| member.target())
                    .collect::<Vec<_>>();

                let args = construct
                    .members
                    .iter()
                    .filter_map(|member| match &member.kind {
                        ElementKind::Binary(binary) => binary.left.target(),
                        _ => member.target(),
                    })
                    .collect::<Vec<_>>();

                fields == args
            }
            (ElementKind::Construct(construct), SymbolKind::Union(union)) => {
                let fields = union
                    .members
                    .iter()
                    .filter(|member| member.is_instance())
                    .filter_map(|member| member.target())
                    .collect::<Vec<_>>();

                let args = construct
                    .members
                    .iter()
                    .filter_map(|member| match &member.kind {
                        ElementKind::Binary(binary) => binary.left.target(),
                        _ => member.target(),
                    })
                    .collect::<Vec<_>>();

                args.len() == 1 && fields.contains(&args[0])
            }
            _ => false,
        }
    }

    pub fn exact<'a>(&self, target: &Element<'a>, resolver: &Resolver<'a>) -> Option<Symbol<'a>> {
        let query = target.target()?;

        for identity in &self.symbols {
            if let Some(symbol) = resolver.registry.get(identity) {
                if let Some(candidate) = symbol.target() {
                    if query == candidate && Self::fits(target, symbol) {
                        return Some(symbol.clone());
                    }
                }
            }
        }

        None
    }

    pub fn lookup<'a>(
        &self,
        target: &Element<'a>,
        resolver: &Resolver<'a>,
    ) -> Result<Symbol<'a>, Vec<ResolveError<'a>>> {
        if let Some(symbol) = Resolver::builtin(target) {
            return Ok(symbol);
        }

        if let Some(symbol) = self.exact(target, resolver) {
            Ok(symbol)
        } else {
            Err(vec![ResolveError {
                kind: ErrorKind::UndefinedSymbol {
                    query: target.target().unwrap().clone(),
                },
                span: target.span.clone(),
                hints: Vec::new(),
            }])
        }
    }
}

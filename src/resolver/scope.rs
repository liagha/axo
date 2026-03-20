use {
    super::{
        ErrorKind, ResolveError, Resolver,
    },
    crate::{
        data::Identity,
        internal::{hash::Map, hash::Set},
        parser::{Element, ElementKind, Symbol, SymbolKind},
        scanner::{Token, TokenKind},
        resolver::Resolvable,
    },
};

#[derive(Clone)]
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

    pub fn collect<'a>(
        &self,
        scopes: &Map<Identity, Scope>,
        registry: &Map<Identity, Symbol<'a>>,
    ) -> Vec<Symbol<'a>> {
        let mut symbols = Vec::new();
        let mut current = Some(self);

        while let Some(scope) = current {
            for identity in &scope.symbols {
                if let Some(symbol) = registry.get(identity) {
                    symbols.push(symbol.clone());
                }
            }
            current = scope.parent.and_then(|id| scopes.get(&id));
        }

        symbols.sort();
        symbols
    }

    pub fn find(&self, target: Identity, scopes: &Map<Identity, Scope>) -> Option<Identity> {
        if self.symbols.contains(&target) {
            return Some(target);
        }

        let parent = self.parent?;
        scopes.get(&parent)?.find(target, scopes)
    }

    pub fn fits<'a>(query: &Element<'a>, candidate: &Symbol<'a>) -> bool {
        match (&query.kind, &candidate.kind) {
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
                let candidates = structure
                    .members
                    .iter()
                    .filter(|member| member.is_instance())
                    .filter_map(|member| member.target())
                    .collect::<Vec<_>>();

                let members = construct
                    .members
                    .iter()
                    .filter_map(|member| match &member.kind {
                        ElementKind::Binary(binary) => binary.left.target(),
                        _ => member.target(),
                    })
                    .collect::<Vec<_>>();

                candidates == members
            }
            (ElementKind::Construct(construct), SymbolKind::Union(union)) => {
                let candidates = union
                    .members
                    .iter()
                    .filter(|member| member.is_instance())
                    .filter_map(|member| member.target())
                    .collect::<Vec<_>>();

                let members = construct
                    .members
                    .iter()
                    .filter_map(|member| match &member.kind {
                        ElementKind::Binary(binary) => binary.left.target(),
                        _ => member.target(),
                    })
                    .collect::<Vec<_>>();

                members.len() == 1 && candidates.contains(&members[0])
            }
            _ => false,
        }
    }

    pub fn exact<'a>(
        &self,
        target: &Element<'a>,
        scopes: &Map<Identity, Scope>,
        registry: &Map<Identity, Symbol<'a>>,
    ) -> Option<Symbol<'a>> {
        let query = target.target()?;
        let mut current = Some(self);

        while let Some(scope) = current {
            for identity in &scope.symbols {
                if let Some(symbol) = registry.get(identity) {
                    if let Some(candidate) = symbol.target() {
                        if query == candidate && Self::fits(target, symbol) {
                            return Some(symbol.clone());
                        }
                    }
                }
            }
            current = scope.parent.and_then(|id| scopes.get(&id));
        }

        None
    }

    pub fn lookup<'a>(
        &self,
        target: &Element<'a>,
        scopes: &Map<Identity, Scope>,
        registry: &Map<Identity, Symbol<'a>>,
    ) -> Result<Symbol<'a>, Vec<ResolveError<'a>>> {
        if let Some(symbol) = Resolver::builtin(target) {
            return Ok(symbol);
        }

        if let Some(symbol) = self.exact(target, scopes, registry) {
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

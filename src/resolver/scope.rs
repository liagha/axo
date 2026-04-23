use crate::{
    data::Identity,
    internal::hash::Set,
    parser::{Element, ElementKind, Symbol, SymbolKind},
    resolver::{Resolvable, Resolver},
    scanner::{Token, TokenKind},
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

    pub fn has(&self, target: Identity) -> bool {
        self.symbols.contains(&target)
    }

    fn names<'a>(members: &[Element<'a>]) -> Vec<crate::data::Str<'a>> {
        members
            .iter()
            .filter_map(|member| match &member.kind {
                ElementKind::Binary(binary) => binary.left.target(),
                _ => member.target(),
            })
            .collect()
    }

    fn fields<'a>(members: &[Symbol<'a>]) -> Vec<crate::data::Str<'a>> {
        members
            .iter()
            .filter(|member| member.is_instance())
            .filter_map(|member| member.target())
            .collect()
    }

    pub fn fits<'a>(query: &Element<'a>, symbol: &Symbol<'a>) -> bool {
        match (&query.kind, &symbol.kind) {
            (ElementKind::Literal(token), _) => {
                matches!(
                    **token,
                    Token {
                        kind: TokenKind::Identifier(_),
                        ..
                    }
                )
            }
            (ElementKind::Invoke(invoke), SymbolKind::Function(function)) => {
                invoke.members.len() == function.members.len()
            }
            (ElementKind::Construct(construct), SymbolKind::Structure(structure)) => {
                Self::fields(&structure.members) == Self::names(&construct.members)
            }
            (ElementKind::Construct(construct), SymbolKind::Union(union)) => {
                let args = Self::names(&construct.members);
                args.len() == 1 && Self::fields(&union.members).contains(&args[0])
            }
            _ => false,
        }
    }

    pub fn exact<'a>(&self, target: &Element<'a>, resolver: &Resolver<'a>) -> Option<Symbol<'a>> {
        let query = target.target()?;

        self.symbols.iter().find_map(|identity| {
            resolver
                .registry
                .get(identity)
                .filter(|symbol| {
                    symbol.target() == Some(query.clone()) && Self::fits(target, symbol)
                })
                .cloned()
        })
    }

    pub fn lookup<'a>(
        &self,
        target: &Element<'a>,
        resolver: &Resolver<'a>,
    ) -> Result<Symbol<'a>, Vec<crate::resolver::ResolveError<'a>>> {
        Resolver::builtin(target)
            .or_else(|| self.exact(target, resolver))
            .ok_or_else(|| resolver.undefined(target))
    }
}

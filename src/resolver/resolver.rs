use crate::{
    data::Identity,
    internal::hash::{Map, Set},
    parser::{Element, Symbol},
    resolver::{
        next_identity,
        scope::Scope,
        ErrorKind,
        ResolveError,
        Type,
    },
};

pub struct Resolver<'a> {
    pub active: Identity,
    pub scopes: Map<Identity, Scope>,
    pub registry: Map<Identity, Symbol<'a>>,
    pub input: Vec<Element<'a>>,
    pub errors: Vec<ResolveError<'a>>,
    pub variables: Vec<Option<Type<'a>>>,
    pub returns: Vec<Type<'a>>,
    pub dependencies: Set<Identity>,
}

impl Clone for Resolver<'_> {
    fn clone(&self) -> Self {
        Self {
            active: self.active,
            scopes: self.scopes.clone(),
            registry: self.registry.clone(),
            input: self.input.clone(),
            errors: self.errors.clone(),
            variables: self.variables.clone(),
            returns: self.returns.clone(),
            dependencies: self.dependencies.clone(),
        }
    }
}

pub trait Resolvable<'a> {
    fn depending(&self, resolver: &mut Resolver<'a>);
    fn declare(&mut self, resolver: &mut Resolver<'a>);
    fn resolve(&mut self, resolver: &mut Resolver<'a>);
    fn reify(&mut self, resolver: &mut Resolver<'a>);
    fn is_instance(&self) -> bool {
        false
    }
}

impl<'a> Resolver<'a> {
    pub fn new() -> Self {
        let mut scopes = Map::new();
        scopes.insert(0, Scope::new(None));

        Self {
            active: 0,
            scopes,
            registry: Map::new(),
            input: Vec::new(),
            errors: Vec::new(),
            variables: Vec::new(),
            returns: Vec::new(),
            dependencies: Set::new(),
        }
    }

    pub fn active(&self) -> &Scope {
        self.scopes.get(&self.active).unwrap()
    }

    pub fn active_mut(&mut self) -> &mut Scope {
        self.scopes.get_mut(&self.active).unwrap()
    }

    pub fn enter(&mut self) {
        let next = next_identity();

        self.scopes.insert(next, Scope::new(Some(self.active)));
        self.active = next;
    }

    pub fn enter_scope(&mut self, mut scope: Scope) {
        let next = next_identity();

        scope.parent = Some(self.active);
        self.scopes.insert(next, scope);
        self.active = next;
    }

    pub fn exit(&mut self) {
        if let Some(scope) = self.scopes.get(&self.active) {
            if let Some(parent) = scope.parent {
                self.active = parent;
            }
        }
    }

    pub fn insert(&mut self, symbol: Symbol<'a>) {
        let identity = symbol.identity;
        self.registry.insert(identity, symbol);

        if let Some(scope) = self.scopes.get_mut(&self.active) {
            scope.insert(identity);
        }
    }

    pub fn get_symbol(&self, identity: Identity) -> Option<&Symbol<'a>> {
        self.registry.get(&identity)
    }

    pub fn collect(&self) -> Vec<Symbol<'a>> {
        let mut symbols = Vec::new();
        let mut current = Some(self.active());

        while let Some(scope) = current {
            for identity in &scope.symbols {
                if let Some(symbol) = self.registry.get(identity) {
                    symbols.push(symbol.clone());
                }
            }
            current = scope.parent.and_then(|id| self.scopes.get(&id));
        }

        symbols.sort();
        symbols
    }

    pub fn find(&self, target: Identity) -> Option<Identity> {
        let mut current = Some(self.active());

        while let Some(scope) = current {
            if scope.has(target) {
                return Some(target);
            }
            current = scope.parent.and_then(|id| self.scopes.get(&id));
        }

        None
    }

    pub fn exact(&self, target: &Element<'a>) -> Option<Symbol<'a>> {
        let mut current = Some(self.active());

        while let Some(scope) = current {
            if let Some(symbol) = scope.exact(target, self) {
                return Some(symbol);
            }
            current = scope.parent.and_then(|id| self.scopes.get(&id));
        }

        None
    }

    pub fn lookup(&self, target: &Element<'a>) -> Result<Symbol<'a>, Vec<ResolveError<'a>>> {
        if let Some(symbol) = Self::builtin(target) {
            return Ok(symbol);
        }

        if let Some(symbol) = self.exact(target) {
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

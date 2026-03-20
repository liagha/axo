use crate::{
    data::{Identity},
    internal::{
        hash::{
            Map, Set,
        },
    },
    parser::{Element, Symbol},
    resolver::{
        scope::Scope,
        ResolveError,
        Type,
        next_identity,
    },
};

pub struct Resolver<'resolver> {
    pub active: Identity,
    pub scopes: Map<Identity, Scope>,
    pub registry: Map<Identity, Symbol<'resolver>>,
    pub input: Vec<Element<'resolver>>,
    pub errors: Vec<ResolveError<'resolver>>,
    pub variables: Vec<Option<Type<'resolver>>>,
    pub returns: Vec<Type<'resolver>>,
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

pub trait Resolvable<'resolvable> {
    fn depending(&self, resolver: &mut Resolver<'resolvable>);
    fn declare(&mut self, resolver: &mut Resolver<'resolvable>);
    fn resolve(&mut self, resolver: &mut Resolver<'resolvable>);
    fn reify(&mut self, resolver: &mut Resolver<'resolvable>);
    fn is_instance(&self) -> bool {
        false
    }
}

impl<'resolver> Resolver<'resolver> {
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

    pub fn insert(&mut self, symbol: Symbol<'resolver>) {
        let identity = symbol.identity;
        self.registry.insert(identity, symbol);

        if let Some(scope) = self.scopes.get_mut(&self.active) {
            scope.insert(identity);
        }
    }

    pub fn get_symbol(&self, identity: Identity) -> Option<&Symbol<'resolver>> {
        self.registry.get(&identity)
    }
}

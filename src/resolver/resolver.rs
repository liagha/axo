use {
    super::{scope::Scope, ResolveError},
    crate::{
        data::{memory::replace},
        parser::{Element, Symbol},
    },
};

pub struct Resolver<'resolver> {
    pub scope: Scope<Symbol<'resolver>>,
    pub input: Vec<Element<'resolver>>,
    pub errors: Vec<ResolveError<'resolver>>,
}

impl Clone for Resolver<'_> {
    fn clone(&self) -> Self {
        Self {
            scope: self.scope.clone(),
            input: self.input.clone(),
            errors: self.errors.clone(),
        }
    }
}

pub trait Resolvable<'resolvable> {
    fn resolve(
        &mut self,
        resolver: &mut Resolver<'resolvable>,
    );
}

impl<'resolver> Resolver<'resolver> {
    pub fn new() -> Self {
        Self {
            scope: Scope::new(),
            input: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn set_input(&mut self, input: Vec<Element<'resolver>>) {
        self.input = input;
    }

    pub fn enter(&mut self) {
        let parent = replace(&mut self.scope, Scope::new());
        self.scope.attach(parent);
    }

    pub fn enter_scope(&mut self, scope: Scope<Symbol<'resolver>>) {
        let parent = replace(&mut self.scope, scope);
        self.scope.attach(parent);
    }

    pub fn exit(&mut self) {
        if let Some(parent) = self.scope.detach() {
            self.scope = parent;
        }
    }

    pub fn add(&mut self, symbol: Symbol<'resolver>) {
        self.scope.add(symbol);
    }

    pub fn resolve(&mut self) {
        let mut input = self.input.clone();

        for element in input.iter_mut() {
            element.resolve(self);
        }

        self.input = input;
    }
}

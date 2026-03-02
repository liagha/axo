use {
    super::{scope::Scope, ResolveError},
    crate::{
        data::{memory::replace, Boolean, Identity},
        format::Debug,
        parser::{Element, ElementKind, Symbol},
        scanner::Token,
    },
};
use crate::analyzer::Analysis;
use crate::checker::Type;

#[derive(Clone, Debug)]
pub struct Resolution<'resolution> {
    pub reference: Option<Identity>,
    pub typed: Type<'resolution>,
    pub analysis: Analysis<'resolution>,
}

impl<'resolution> Resolution<'resolution> {
    pub fn new(
        reference: Option<Identity>,
        typed: Type<'resolution>,
        analysis: Analysis<'resolution>,
    ) -> Self {
        Self {
            reference,
            typed,
            analysis,
        }
    }
}

#[derive(Clone)]
pub struct Inference<'inference> {
    pub target: Token<'inference>,
    pub declared: Option<Type<'inference>>,
    pub inferred: Option<Type<'inference>>,
}

impl<'inference> Inference<'inference> {
    pub fn new(
        target: Token<'inference>,
        declared: Option<Type<'inference>>,
        inferred: Option<Type<'inference>>,
    ) -> Self {
        Self {
            target,
            declared,
            inferred,
        }
    }
}

pub struct Resolver<'resolver> {
    pub counter: Identity,
    pub scope: Scope<'resolver>,
    pub input: Vec<Element<'resolver>>,
    pub output: Vec<Resolution<'resolver>>,
    pub errors: Vec<ResolveError<'resolver>>,
    pub symbols: Vec<(Symbol<'resolver>, Option<Inference<'resolver>>)>,
    pub cycle: bool,
    pub method: bool,
}

impl Clone for Resolver<'_> {
    fn clone(&self) -> Self {
        Self {
            counter: self.counter,
            scope: self.scope.clone(),
            input: self.input.clone(),
            output: self.output.clone(),
            errors: self.errors.clone(),
            symbols: self.symbols.clone(),
            cycle: false,
            method: false,
        }
    }
}

pub trait Resolvable<'resolvable> {
    fn resolve(
        &self,
        resolver: &mut Resolver<'resolvable>,
    ) -> Result<Resolution<'resolvable>, Vec<ResolveError<'resolvable>>>;
    fn is_instance(&self, resolver: &mut Resolver<'resolvable>) -> Boolean;
}

impl<'resolver> Resolver<'resolver> {
    pub fn new() -> Self {
        Self {
            counter: 0,
            scope: Scope::new(),
            input: Vec::new(),
            output: Vec::new(),
            errors: Vec::new(),
            symbols: Vec::new(),
            cycle: false,
            method: false,
        }
    }

    pub fn with_input(&mut self, input: Vec<Element<'resolver>>) {
        self.input = input;
    }

    pub fn enter(&mut self) {
        let parent = replace(&mut self.scope, Scope::new());
        self.scope.attach(parent);
    }

    pub fn enter_scope(&mut self, scope: Scope<'resolver>) {
        let parent = replace(&mut self.scope, scope);
        self.scope.attach(parent);
    }

    pub fn exit(&mut self) {
        if let Some(parent) = self.scope.detach() {
            self.scope = parent;
        }
    }

    pub fn define(&mut self, symbol: Symbol<'resolver>) {
        self.scope.add(symbol);
    }

    pub fn next_id(&mut self) -> Identity {
        let id = self.counter;
        self.counter += 1;
        id
    }

    pub fn resolve(&mut self) {
        self.symbols.clear();

        for element in self.input.clone() {
            match element.resolve(self) {
                Ok(resolution) => {
                    self.output.push(resolution);
                }
                Err(errors) => {
                    self.errors.extend(errors);
                }
            }
        }
    }
}

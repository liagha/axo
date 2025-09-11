use {
    matchete::{Assessor, Scheme},
    super::{
        error::{
            ErrorKind,
        },
        scope::{
            Scope,
        },
        analyzer::{
            Analysis,
        },
        validator::Sugared,
        checker::Checkable,
        ResolveError,
    },
    crate::{
        tracker::{
            Span,
        },
        scanner::{
            Token, TokenKind,
            OperatorKind,
        },
        parser::{
            Element, ElementKind,
            Symbol, SymbolKind,
        },
        schema::*,
        data::{
            Str,
            Scale,
            Boolean,
            memory::{
                replace,
            }
        },
        resolver::{
            matcher::{
                Affinity, Aligner,
            },
        },
        format::Debug,
    },
};

pub type Id = usize;

#[derive(Debug)]
pub struct Resolver<'resolver> {
    pub counter: Id,
    pub scope: Scope<'resolver>,
    pub input: Vec<Element<'resolver>>,
    pub output: Vec<Analysis<'resolver>>,
    pub errors: Vec<ResolveError<'resolver>>,
}

impl Clone for Resolver<'_> {
    fn clone(&self) -> Self {
        Self {
            counter: self.counter,
            scope: self.scope.clone(),
            input: self.input.clone(),
            output: self.output.clone(),
            errors: self.errors.clone(),
        }
    }
}

pub trait Resolvable<'resolvable> {
    fn resolve(&self, resolver: &mut Resolver<'resolvable>) -> Option<Symbol<'resolvable>>;
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

    pub fn define(&mut self, mut symbol: Symbol<'resolver>) {
        symbol.id = self.next_id();
        self.scope.add(symbol);
    }

    pub fn next_id(&mut self) -> Id {
        let id = self.counter;
        self.counter += 1;
        id
    }

    pub fn get(&mut self, target: &Element<'resolver>) -> Option<Symbol<'resolver>> {
        match self.scope.try_get(target) {
            Ok(symbol) => Some(symbol),
            Err(errors) => {
                self.errors.extend(errors.clone());
                None
            }
        }
    }

    pub fn lookup(&mut self, target: &Element<'resolver>, scope: &Scope<'resolver>) -> Option<Symbol<'resolver>> {
        match Scope::try_lookup(target, scope) {
            Ok(symbol) => Some(symbol),
            Err(errors) => {
                self.errors.extend(errors.clone());
                None
            }
        }
    }

    pub fn process(&mut self) -> Vec<Analysis<'resolver>> {
        self.preresolve();

        for element in self.input.clone() {
            element.resolve(self);
        }

        self.output.clone()
    }

    pub fn preresolve(&mut self) {
        for index in 0..self.input.len() {
            let element = self.input[index].clone();
            self.input[index] = element.desugar();

            if let ElementKind::Symbolize(symbol) = self.input[index].kind.clone() {
                symbol.resolve(self);
            }
        }
    }
}
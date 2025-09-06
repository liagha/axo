use matchete::{Assessor, Scheme};
use {
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
        ResolveError,
    },
    crate::{
        tracker::{
            Span,
        },
        scanner::{
            Token, TokenKind
        },
        parser::{
            Element, ElementKind,
            Symbol, SymbolKind,
        },
        schema::{
            Enumeration, Extension,
            Method, Structure, Module, Binding,
        },
        data::{
            Scale,
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
        let parent_scope = replace(&mut self.scope, Scope::child());
        self.scope.attach(parent_scope);
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

    pub fn try_get(&mut self, target: &Element<'resolver>) -> Result<Symbol<'resolver>, Vec<ResolveError<'resolver>>> {
        let candidates = self.scope.all();

        let mut aligner = Aligner::new();
        let mut affinity = Affinity::new();

        let mut assessor = Assessor::new()
            .floor(0.5)
            .dimension(&mut affinity, 0.6)
            .dimension(&mut aligner, 0.4)
            .scheme(Scheme::Additive);

        let champion = assessor.champion(target, &candidates);

        if let Some(champion) = champion {
            Ok(champion)
        } else {
            let mut errors = assessor.errors.clone();
            if errors.is_empty() {
                errors.push(ResolveError {
                    kind: ErrorKind::UndefinedSymbol { query: target.brand().unwrap().clone() },
                    span: target.span.clone(),
                    hints: Vec::new(),
                });
            }
            Err(errors)
        }
    }

    pub fn get(&mut self, target: &Element<'resolver>) -> Option<Symbol<'resolver>> {
        match self.try_get(target) {
            Ok(symbol) => Some(symbol),
            Err(errors) => {
                self.errors.extend(errors.clone());
                None
            }
        }
    }

    pub fn try_lookup(&mut self, target: &Element<'resolver>, candidates: &Vec<Symbol<'resolver>>) -> Result<Symbol<'resolver>, Vec<ResolveError<'resolver>>> {
        let mut aligner = Aligner::new();
        let mut affinity = Affinity::new();

        let mut assessor = Assessor::new()
            .floor(0.5)
            .dimension(&mut affinity, 0.6)
            .dimension(&mut aligner, 0.4)
            .scheme(Scheme::Additive);

        let champion = assessor.champion(target, candidates);

        if let Some(champion) = champion {
            Ok(champion)
        } else {
            if assessor.errors.is_empty() {
                let error = ResolveError {
                    kind: ErrorKind::UndefinedSymbol { query: target.brand().unwrap().clone() },
                    span: target.span.clone(),
                    hints: Vec::new(),
                };
                Err(vec![error])
            } else {
                Err(assessor.errors.clone())
            }
        }
    }

    pub fn lookup(&mut self, target: &Element<'resolver>, candidates: &Vec<Symbol<'resolver>>) -> Option<Symbol<'resolver>> {
        match self.try_lookup(target, candidates) {
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
            self.resolve(&element);
        }

        self.output.clone()
    }

    pub fn preresolve(&mut self) {
        for index in 0..self.input.len() {
            let element = self.input[index].clone();
            self.input[index] = self.desugar(element);

            if let ElementKind::Symbolize(symbol) = &self.input[index].kind {
                let symbol = symbol.clone();
                self.define(symbol);
            }
        }
    }

    pub fn resolve(&mut self, element: &Element<'resolver>) {
        let Element { kind, .. } = element.clone();

        match kind {
            ElementKind::Assign(assign) => {
                if let Some(symbol) = self.get(&*assign.target) {
                    self.resolve(&*assign.value);

                    let target = self.infer_symbol(symbol.clone());
                    let value = self.infer_element(&*assign.value);

                    self.check(target, value);
                }
            }

            ElementKind::Delimited(delimited) => {
                self.enter();
                delimited.items.iter().for_each(|item| self.resolve(item));
                self.exit();
            }

            ElementKind::Literal(Token { kind: TokenKind::Identifier(_), .. }) => {
                self.get(&element);
            }

            ElementKind::Construct { .. }
            | ElementKind::Invoke { .. }
            | ElementKind::Index { .. } => {
                self.get(&element);
            }

            ElementKind::Binary(binary) => {
                self.resolve(&*binary.left);
                self.resolve(&*binary.right);
            }

            ElementKind::Unary(unary) => self.resolve(&*unary.operand),

            ElementKind::Label(label) => {
                self.resolve(&*label.label);
                self.resolve(&*label.element);
            }

            ElementKind::Conditional(conditioned) => {
                self.resolve(&*conditioned.condition);
                self.enter();
                self.resolve(&*conditioned.then);
                self.exit();

                if let Some(alternate) = conditioned.alternate {
                    self.enter();
                    self.resolve(&*alternate);
                    self.exit();
                }
            }

            ElementKind::While(repeat) => {
                if let Some(condition) = repeat.condition {
                    self.resolve(&*condition);
                }
                self.enter();
                self.resolve(&*repeat.body);
                self.exit();
            }

            ElementKind::Cycle(walk) => {
                self.resolve(&*walk.clause);

                let parent = replace(&mut self.scope, Scope::child());
                self.scope.attach(parent);

                self.enter();
                self.resolve(&*walk.body);
                self.exit();
            }

            ElementKind::Access(access) => {
                let candidates = self.scope.all();
                let target = self.lookup(&*access.target, &candidates);

                if let Some(target) = target {
                    let members = target.scope.all();
                    let member = self.lookup(&*access.member, &members);
                }
            }

            ElementKind::Return(value) | ElementKind::Break(value) | ElementKind::Continue(value) => {
                if let Some(value) = value {
                    self.resolve(&value);
                }
            }

            ElementKind::Symbolize(_)
            | ElementKind::Literal(_)
            | ElementKind::Procedural(_) => {}
        }

        let analysis = self.analyze(element.clone());

        match analysis {
            Ok(analysis) => {
                self.output.push(analysis);
            }
            Err(error) => {
                let error = ResolveError::new(ErrorKind::Analyze { error: error.clone() }, error.span);

                self.errors.push(error);
            }
        }
    }

    pub fn symbolize(&mut self, mut symbol: Symbol<'resolver>) {
        symbol.id = self.next_id();
        match symbol.kind {
            SymbolKind::Inclusion(_) => {}
            SymbolKind::Preference(_) => {}
            SymbolKind::Extension(extension) => {
                let candidates = self.scope.all();

                if let Some(mut target) = self.lookup(&*extension.target, &candidates) {
                    if let Some(extension) = extension.extension {
                        if let Some(found) = self.lookup(&*extension, &candidates) {
                            if let SymbolKind::Structure(structure) = found.kind {
                                self.scope.remove(&target);
                                target.scope.symbols.extend(structure.members.iter().cloned());
                                self.scope.add(target);
                            }
                        }
                    } else {
                        self.scope.remove(&target);
                        target.scope.symbols.extend(extension.members.iter().cloned());
                        self.scope.add(target);
                    }
                }
            }
            _ => {
                self.scope.add(symbol);
            }
        }
    }
}
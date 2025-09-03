use {
    super::{
        error::{
            ErrorKind,
        },
        matcher::{
            symbol_matcher,
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
            Symbol, Symbolic,
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
        format::Debug,
    },
};

#[derive(Debug)]
pub struct Resolver<'resolver> {
    pub scope: Scope<'resolver>,
    pub input: Vec<Element<'resolver>>,
    pub output: Vec<Analysis<'resolver>>,
    pub errors: Vec<ResolveError<'resolver>>,
}

impl Clone for Resolver<'_> {
    fn clone(&self) -> Self {
        Self {
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

    pub fn define(&mut self, symbol: Symbol<'resolver>) {
        self.scope.add(symbol);
    }

    pub fn try_get(&mut self, target: &Element<'resolver>) -> Result<Symbol<'resolver>, Vec<ResolveError<'resolver>>> {
        let candidates = self.scope.all().iter().cloned().collect::<Vec<_>>();
        let mut assessor = symbol_matcher();
        let champion = assessor.champion(target, &candidates);
        // assessor.dimensions.sort_by(|first, other| first.resemblance.to_f64().partial_cmp(&other.resemblance.to_f64()).unwrap());

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
        let mut assessor = symbol_matcher();
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
        self.preresolve(&self.input.clone());

        for element in self.input.clone() {
            self.resolve(&element);
        }

        self.output.clone()
    }

    pub fn preresolve(&mut self, elements: &Vec<Element<'resolver>>) {
        elements.iter().for_each(|element| {
            if let ElementKind::Symbolize(symbol) = &element.kind {
                self.scope.add(symbol.clone());
            }
        })
    }

    pub fn resolve(&mut self, element: &Element<'resolver>) {
        let Element { kind, .. } = self.desugar(element.clone());

        match kind {
            ElementKind::Assign(assign) => {
                if let Some(symbol) = self.get(assign.get_target()) {
                    self.resolve(assign.get_value());

                    let target = self.infer_symbol(symbol.clone());
                    let value = self.infer_element(assign.get_value());

                    if target == value {
                        println!("types match in {}.", symbol.span);
                    } else {
                        println!("types don't match (`{:?}` & `{:?}`) in `{}`.", target, value, symbol.span);
                    }
                }
            }

            ElementKind::Block(body) => {
                self.enter();
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

            ElementKind::Group(group) => {
                for element in group.items {
                    self.resolve(&element);
                }
            }
            ElementKind::Collection(collection) => {
                for element in collection.items {
                    self.resolve(&element);
                }
            }
            ElementKind::Bundle(bundle) => {
                for element in bundle.items {
                    self.resolve(&element);
                }
            }

            ElementKind::Binary(binary) => {
                self.resolve(binary.get_left());
                self.resolve(binary.get_right());
            }

            ElementKind::Unary(unary) => self.resolve(&unary.get_operand()),

            ElementKind::Label(label) => {
                self.resolve(label.get_label());
                self.resolve(label.get_element());
            }

            ElementKind::Conditional(conditioned) => {
                self.resolve(conditioned.get_condition());
                self.enter();
                self.resolve(conditioned.get_then());
                self.exit();

                if let Some(alternate) = conditioned.get_alternate() {
                    self.enter();
                    self.resolve(alternate);
                    self.exit();
                }
            }

            ElementKind::While(repeat) => {
                if let Some(condition) = repeat.get_condition() {
                    self.resolve(condition);
                }
                self.enter();
                self.resolve(repeat.get_body());
                self.exit();
            }

            ElementKind::Cycle(walk) => {
                self.resolve(walk.get_clause());

                let parent = replace(&mut self.scope, Scope::child());
                self.scope.attach(parent);

                self.enter();
                self.resolve(walk.get_body());
                self.exit();
            }

            ElementKind::Access(access) => {
                let candidates = self.scope.all().iter().cloned().collect::<Vec<_>>();
                let target = self.lookup(access.get_target(), &candidates);

                if let Some(target) = target {
                    let members = target.scope.all().iter().cloned().collect::<Vec<_>>();
                    let member = self.lookup(access.get_member(), &members);
                }
            }

            ElementKind::Return(value) | ElementKind::Break(value) | ElementKind::Continue(value) => {
                if let Some(value) = value {
                    self.resolve(&value);
                }
            }

            ElementKind::Symbolize(_)
            | ElementKind::Literal(_)
            | ElementKind::Procedural(_)
            | ElementKind::Sequence(_)
            | ElementKind::Series(_) => {}
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

    pub fn symbolize(&mut self, symbol: Symbol<'resolver>) {
        match symbol.kind {
            Symbolic::Inclusion(_) => {}
            Symbolic::Preference(_) => {}
            Symbolic::Extension(extension) => {
                let candidates = self.scope.all().iter().cloned().collect::<Vec<_>>();
                
                if let Some(mut target) = self.lookup(extension.get_target(), &candidates) {
                    if let Some(extension) = extension.get_extension() {
                        if let Some(found) = self.lookup(extension, &candidates) {
                            if let Symbolic::Structure(structure) = found.kind {
                                self.scope.remove(&target);
                                target.scope.symbols.extend(structure.get_fields().iter().cloned());
                                self.scope.add(target);
                            }
                        }
                    } else {
                        self.scope.remove(&target);
                        target.scope.symbols.extend(extension.get_members().iter().cloned());
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
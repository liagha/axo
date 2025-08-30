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

impl<'resolver> Resolver<'resolver> {
    pub fn new() -> Self {
        Self {
            scope: Scope::new(),
            input: Vec::new(),
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
                    hints: vec![],
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
                    hints: vec![],
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

    pub fn fail(&mut self, error: ErrorKind<'resolver>, span: Span<'resolver>) {
        let error = ResolveError {
            kind: error,
            span: span.clone(),
            hints: vec![],
        };

        self.errors.push(error);
    }

    pub fn process(&mut self) {
        self.symbolize_all(&self.input.clone());
        self.resolve_all(&self.input.clone());
    }

    fn symbolize_all(&mut self, elements: &Vec<Element<'resolver>>) {
        for element in elements {
            self.extract_symbols(element);
        }
    }

    fn resolve_all(&mut self, elements: &Vec<Element<'resolver>>) {
        for element in elements {
            self.resolve(element);
        }
    }

    fn extract_symbols(&mut self, element: &Element<'resolver>) {
        let Element { kind, .. } = element.clone();

        match kind {
            ElementKind::Symbolize(symbol) => {
                self.symbolize(symbol);
            }

            ElementKind::Block(body) => {
                for item in body.items {
                    self.extract_symbols(&item);
                }
            }

            ElementKind::Group(group) => {
                for item in group.items {
                    self.extract_symbols(&item);
                }
            }

            ElementKind::Collection(collection) => {
                for item in collection.items {
                    self.extract_symbols(&item);
                }
            }

            ElementKind::Bundle(bundle) => {
                for item in bundle.items {
                    self.extract_symbols(&item);
                }
            }

            ElementKind::Binary(binary) => {
                self.extract_symbols(binary.get_left());
                self.extract_symbols(binary.get_right());
            }

            ElementKind::Unary(unary) => {
                self.extract_symbols(&unary.get_operand());
            }

            ElementKind::Label(label) => {
                self.extract_symbols(label.get_label());
                self.extract_symbols(label.get_element());
            }

            ElementKind::Conditional(conditioned) => {
                self.extract_symbols(conditioned.get_condition());
                self.extract_symbols(conditioned.get_then());

                if let Some(alternate) = conditioned.get_alternate() {
                    self.extract_symbols(alternate);
                }
            }

            ElementKind::While(repeat) => {
                if let Some(condition) = repeat.get_condition() {
                    self.extract_symbols(condition);
                }
                self.extract_symbols(repeat.get_body());
            }

            ElementKind::Cycle(walk) => {
                self.extract_symbols(walk.get_clause());
                self.extract_symbols(walk.get_body());
            }

            ElementKind::Access(access) => {
                self.extract_symbols(access.get_target());
                self.extract_symbols(access.get_member());
            }

            ElementKind::Return(value) | ElementKind::Break(value) | ElementKind::Continue(value) => {
                if let Some(value) = value {
                    self.extract_symbols(&value);
                }
            }

            ElementKind::Assign(assign) => {
                self.extract_symbols(assign.get_target());
            }

            ElementKind::Construct { .. }
            | ElementKind::Invoke { .. }
            | ElementKind::Index { .. }
            | ElementKind::Literal(_)
            | ElementKind::Procedural(_)
            | ElementKind::Sequence(_)
            | ElementKind::Series(_) => {}
        }
    }

    pub fn resolve(&mut self, element: &Element<'resolver>) {
        let Element { kind, .. } = element.clone();

        match kind {
            ElementKind::Assign(assign) => {
                self.get(assign.get_target());
            }

            ElementKind::Block(body) => {
                self.enter();
                self.resolve_all(&body.items);
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
    }

    pub fn symbolize(&mut self, symbol: Symbol<'resolver>) {
        match symbol.value {
            Symbolic::Inclusion(_) => {}
            Symbolic::Extension(extension) => {
                let candidates = self.scope.all().iter().cloned().collect::<Vec<_>>();
                
                if let Some(mut target) = self.lookup(extension.get_target(), &candidates) {
                    if let Some(extension) = extension.get_extension() {
                        if let Some(found) = self.lookup(extension, &candidates) {
                            if let Symbolic::Structure(structure) = found.value {
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
            Symbolic::Preference(_) => {}
            _ => {
                self.scope.add(symbol);
            }
        }
    }

    pub fn extend(&mut self, symbols: Vec<Symbol<'resolver>>) {
        self.scope.extend(symbols);
    }

    pub fn merge(&mut self, other: Resolver<'resolver>) {
        self.scope.merge(&other.scope);
        self.errors.extend(other.errors);
    }

    pub fn collect(&mut self, elements: Vec<Element<'resolver>>) -> Vec<Option<Symbol<'resolver>>> {
        elements.iter().map(|e| self.get(e)).collect()
    }

    pub fn batch(&mut self, symbols: Vec<Symbol<'resolver>>) {
        for symbol in symbols {
            self.define(symbol);
        }
    }

    pub fn purge(&mut self, symbol: &Symbol<'resolver>) -> bool {
        self.scope.remove(symbol)
    }

    pub fn replace(&mut self, old: &Symbol<'resolver>, new: Symbol<'resolver>) -> bool {
        self.scope.replace(old, new)
    }

    pub fn clear(&mut self) {
        self.scope.clear();
        self.errors.clear();
    }

    pub fn reset(&mut self) {
        self.scope = Scope::new();
        self.errors.clear();
    }

    pub fn restore(&mut self, snapshot: Resolver<'resolver>) {
        *self = snapshot;
    }

    pub fn isolate(&mut self) {
        self.scope.isolate();
    }

    pub fn depth(&self) -> Scale {
        self.scope.depth()
    }

    pub fn symbols(&self) -> Vec<Symbol<'resolver>> {
        self.scope.flatten()
    }

    pub fn visible(&self, symbol: &Symbol<'resolver>) -> bool {
        self.scope.visible(symbol)
    }

    pub fn shadow(&mut self, symbol: Symbol<'resolver>) {
        self.scope.shadow(symbol);
    }

    pub fn nested(&self) -> bool {
        self.scope.nested()
    }

    pub fn toplevel(&self) -> bool {
        self.scope.toplevel()
    }

    pub fn validate(&mut self, element: &Element<'resolver>) -> bool {
        let initial_errors = self.errors.len();
        self.resolve(element);
        self.errors.len() == initial_errors
    }

    pub fn succeed(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn failed(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn count(&self) -> Scale {
        self.scope.count()
    }

    pub fn empty(&self) -> bool {
        self.scope.empty()
    }

    pub fn cascade(&self) -> Vec<Vec<Symbol<'resolver>>> {
        self.scope.cascade().into_iter().map(|set| set.into_iter().collect()).collect()
    }

    pub fn filter<F>(&mut self, predicate: F)
    where
        F: FnMut(&Symbol<'resolver>) -> bool,
    {
        self.scope.retain(predicate);
    }

    pub fn search<F>(&self, predicate: F) -> Vec<Symbol<'resolver>>
    where
        F: Fn(&Symbol<'resolver>) -> bool,
    {
        self.scope.filter(predicate).into_iter().collect()
    }

    pub fn traverse<F>(&mut self, elements: Vec<Element<'resolver>>, visitor: F)
    where
        F: Fn(&Element<'resolver>, &mut Resolver<'resolver>),
    {
        for element in elements {
            visitor(&element, self);
            self.resolve(&element);
        }
    }
}
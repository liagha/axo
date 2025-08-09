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
        parser::{
            Element, ElementKind,
            Symbol,
        },
        schema::{
            Enumeration, Implementation,
            Interface, Method, Structure
        },
        format::Debug,
        data::memory::replace,
    },
};

#[derive(Debug)]
pub struct Resolver<'resolver> {
    pub scope: Scope,
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

    pub fn define(&mut self, symbol: Symbol) {
        self.scope.add(symbol);
    }

    pub fn try_get(&mut self, target: &Element<'resolver>) -> Result<Option<Symbol>, Vec<ResolveError<'resolver>>> {
        let candidates = self.scope.all().iter().cloned().collect::<Vec<_>>();
        let mut assessor = symbol_matcher();
        let champion = assessor.champion(unsafe { std::mem::transmute(target) }, &candidates);

        if let Some(champion) = champion {
            Ok(Some(champion))
        } else {
            if assessor.errors.is_empty() {
                Ok(None)
            } else {
                Err(unsafe { std::mem::transmute(assessor.errors.clone()) })
            }
        }
    }

    pub fn get(&mut self, target: &Element<'resolver>) -> Option<Symbol> {
        match self.try_get(target) {
            Ok(Some(symbol)) => Some(symbol),
            Ok(None) => None,
            Err(errors) => {
                self.errors.extend(errors.clone());

                None
            }
        }
    }

    pub fn try_lookup(&mut self, target: &Element<'resolver>, candidates: Vec<Symbol>) -> Result<Option<Symbol>, Vec<ResolveError<'resolver>>> {
        let mut assessor = symbol_matcher();
        let champion = assessor.champion(unsafe { std::mem::transmute(target) }, &candidates);

        if let Some(champion) = champion {
            Ok(Some(champion))
        } else {
            if assessor.errors.is_empty() {
                Ok(None)
            } else {
                Err(unsafe { std::mem::transmute(assessor.errors.clone()) })
            }
        }
    }

    pub fn lookup(&mut self, target: &Element<'resolver>, candidates: Vec<Symbol>) -> Option<Symbol> {
        match self.try_lookup(target, candidates) {
            Ok(Some(symbol)) => Some(symbol),
            Ok(None) => None,
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
            note: None,
            hints: vec![],
        };

        self.errors.push(error);
    }

    pub fn process(&mut self, elements: Vec<Element<'resolver>>) {
        for element in elements {
            self.resolve(&element.into());
        }
    }

    pub fn resolve(&mut self, element: &Element<'resolver>) {
        let Element { kind, .. } = element.clone();

        match kind {
            ElementKind::Symbolize(symbol) => {
                self.symbolize(symbol);
            }

            ElementKind::Assign(assign) => {
                self.get(assign.get_target());
            }

            ElementKind::Block(body) => {
                self.enter();
                self.process(body.items);
                self.exit();
            }

            ElementKind::Identifier(_) => {
                self.get(&element);
            }

            ElementKind::Construct { .. }
            | ElementKind::Invoke { .. }
            | ElementKind::Index { .. } => {
                self.get(&element);
            }

            ElementKind::Group(group) => {
                for element in group.items {
                    self.resolve(&element.into());
                }
            }
            ElementKind::Collection(collection) => {
                for element in collection.items {
                    self.resolve(&element.into());
                }
            }
            ElementKind::Bundle(bundle) => {
                for element in bundle.items {
                    self.resolve(&element.into());
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

            ElementKind::Repeat(repeat) => {
                if let Some(condition) = repeat.get_condition() {
                    self.resolve(condition);
                }
                self.enter();
                self.resolve(repeat.get_body());
                self.exit();
            }

            ElementKind::Iterate(walk) => {
                self.resolve(walk.get_clause());

                let parent = replace(&mut self.scope, Scope::child());
                self.scope.attach(parent);

                self.resolve(walk.get_body());
                self.exit();
            }

            ElementKind::Access(access) => {
                let candidates = self.scope.all().iter().cloned().collect::<Vec<_>>();
                let _target = self.lookup(access.get_object(), candidates);
            }

            ElementKind::Produce(value) | ElementKind::Abort(value) | ElementKind::Pass(value) => {
                if let Some(value) = value {
                    self.resolve(&value);
                }
            }

            _ => {}
        }
    }

    pub fn symbolize(&mut self, symbol: Symbol) {
        if let Some(implementation) = symbol.cast::<Implementation<Box<Element>, Box<Element>, Symbol>>() {
            let candidates = self.scope.all().iter().cloned().collect::<Vec<_>>();
            if let Some(target) = self.lookup(implementation.get_target(), candidates) {
                if let Some(interface) = implementation.get_interface() {
                    self.scope.remove(&target);

                    let _member = Interface::new(interface.clone(), implementation.get_members().clone());
                    self.scope.add(target);
                } else {
                    self.scope.remove(&target);
                    self.scope.add(target);
                }
            }
        }

        if let Some(_) = symbol.cast::<Structure<Box<Element>, Symbol>>() {
            self.scope.add(symbol.clone());
        } else if let Some(_) = symbol.cast::<Enumeration<Box<Element>, Element>>() {
            self.scope.add(symbol.clone());
        } else if let Some(_) = symbol.cast::<Method<Box<Element>, Symbol, Box<Element>, Option<Box<Element>>>>() {
            self.scope.add(symbol.clone());
        }
    }

    pub fn extend(&mut self, symbols: Vec<Symbol>) {
        self.scope.extend(symbols);
    }

    pub fn merge(&mut self, other: Resolver<'resolver>) {
        self.scope.merge(&other.scope);
        self.errors.extend(other.errors);
    }

    pub fn collect(&mut self, elements: Vec<Element<'resolver>>) -> Vec<Option<Symbol>> {
        elements.iter().map(|e| self.get(e)).collect()
    }

    pub fn batch(&mut self, symbols: Vec<Symbol>) {
        for symbol in symbols {
            self.define(symbol);
        }
    }

    pub fn purge(&mut self, symbol: &Symbol) -> bool {
        self.scope.remove(symbol)
    }

    pub fn replace(&mut self, old: &Symbol, new: Symbol) -> bool {
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

    pub fn depth(&self) -> usize {
        self.scope.depth()
    }

    pub fn symbols(&self) -> Vec<Symbol> {
        self.scope.flatten()
    }

    pub fn visible(&self, symbol: &Symbol) -> bool {
        self.scope.visible(symbol)
    }

    pub fn shadow(&mut self, symbol: Symbol) {
        self.scope.shadow(symbol);
    }

    pub fn nested(&self) -> bool {
        self.scope.nested()
    }

    pub fn toplevel(&self) -> bool {
        self.scope.toplevel()
    }

    pub fn check(&mut self, elements: Vec<Element<'resolver>>) -> bool {
        let initial_errors = self.errors.len();
        self.process(elements);
        self.errors.len() == initial_errors
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

    pub fn count(&self) -> usize {
        self.scope.count()
    }

    pub fn empty(&self) -> bool {
        self.scope.empty()
    }

    pub fn cascade(&self) -> Vec<Vec<Symbol>> {
        self.scope.cascade().into_iter().map(|set| set.into_iter().collect()).collect()
    }

    pub fn filter<F>(&mut self, predicate: F)
    where
        F: FnMut(&Symbol) -> bool,
    {
        self.scope.retain(predicate);
    }

    pub fn search<F>(&self, predicate: F) -> Vec<Symbol>
    where
        F: Fn(&Symbol) -> bool,
    {
        self.scope.filter(predicate).into_iter().collect()
    }

    pub fn traverse<F>(&mut self, elements: Vec<Element<'resolver>>, visitor: F)
    where
        F: Fn(&Element<'resolver>, &mut Resolver<'resolver>),
    {
        for element in elements {
            visitor(&element, self);
            self.resolve(&element.into());
        }
    }
}
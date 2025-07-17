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
        axo_cursor::{
            Span,
        },
        axo_parser::{
            Element, ElementKind,
            Symbol, SymbolKind,
        },
        format::Debug,
        memory::replace,
    },
};
use crate::axo_schema::Interface;

#[derive(Clone, Debug)]
pub struct Resolver {
    pub scope: Scope,
    pub errors: Vec<ResolveError>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            scope: Scope::new(),
            errors: Vec::new(),
        }
    }

    pub fn push_scope(&mut self) {
        let parent_scope = replace(&mut self.scope, Scope::new());
        self.scope.set_parent(parent_scope);
    }

    pub fn pop_scope(&mut self) {
        if let Some(parent) = self.scope.take_parent() {
            self.scope = parent;
        }
    }

    pub fn insert(&mut self, symbol: Symbol) {
        self.scope.insert(symbol);
    }

    pub fn lookup(&mut self, target: &Element, candidates: Vec<Symbol>) -> Option<Symbol> {
        let mut assessor = symbol_matcher();
        let champion = assessor.champion(target, &candidates);
        self.errors.extend(assessor.errors);

        champion.map(|champion| champion)
    }

    pub fn error(&mut self, error: ErrorKind, span: Span) {
        let error = ResolveError {
            kind: error,
            span: span.clone(),
            note: None,
            hints: vec![],
        };
        self.errors.push(error);
    }

    pub fn settle(&mut self, elements: Vec<Element>) {
        for element in elements {
            self.resolve(&element.into());
        }
    }

    pub fn resolve(&mut self, element: &Box<Element>) {
        let Element { kind, .. } = *element.clone();
        let symbols = self.scope.gather().iter().cloned().collect::<Vec<_>>();

        match kind {
            ElementKind::Symbolize(symbol) => {
                self.symbolize(symbol);
            }

            ElementKind::Assign(assign) => {
                self.lookup(assign.get_target(), symbols);
            }

            ElementKind::Block(body) => {
                self.push_scope();
                self.settle(body.items);
                self.pop_scope();
            }

            ElementKind::Identifier(_) => {
                self.lookup(&element, symbols);
            }

            ElementKind::Construct { .. }
            | ElementKind::Invoke { .. }
            | ElementKind::Index { .. } => {
                self.lookup(&element, symbols);
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
                self.push_scope();
                self.resolve(conditioned.get_then());
                self.pop_scope();

                if let Some(alternate) = conditioned.get_alternate() {
                    self.push_scope();
                    self.resolve(alternate);
                    self.pop_scope();
                }
            }

            ElementKind::Repeat(repeat) => {
                if let Some(condition) = repeat.get_condition() {
                    self.resolve(condition);
                }
                self.push_scope();
                self.resolve(repeat.get_body());
                self.pop_scope();
            }

            ElementKind::Iterate(walk) => {
                self.resolve(walk.get_clause());

                let parent = replace(&mut self.scope, Scope::new());
                self.scope.set_parent(parent);

                self.resolve(walk.get_body());
                self.pop_scope();
            }

            ElementKind::Access(access) => {
                let target = self.lookup(access.get_target(), symbols);

                if let Some(target) = target {
                    self.lookup(access.get_object(), target.members);
                }
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
        let symbols = self.scope.gather().iter().cloned().collect::<Vec<_>>();

        match symbol.kind {
            SymbolKind::Formation(_) => {}
            SymbolKind::Inclusion(_) => {}
            SymbolKind::Implementation(implementation) => {
                if let Some(mut target) = self.lookup(implementation.get_target(), symbols) {
                    if let Some(interface) = implementation.get_interface() {
                        self.scope.remove(&target);

                        let member = Symbol::new(SymbolKind::interface(Interface::new(interface.clone(), implementation.get_members().clone())), symbol.span);
                        target.members.push(member);
                        println!("new: {target:?}");
                        self.scope.insert(target);
                    } else {
                        self.scope.remove(&target);

                        target.members.extend(implementation.get_members().clone());
                        println!("new: {target:?}");
                        self.scope.insert(target);
                    }
                }

                println!("f");
            }
            SymbolKind::Interface(_) => {}
            SymbolKind::Binding(_) => {}
            SymbolKind::Structure(_) => {
                self.scope.insert(symbol);
            }
            SymbolKind::Enumeration(_) => {
                self.scope.insert(symbol);
            }
            SymbolKind::Method(_) => {
                self.scope.insert(symbol);
            }
        }
    }
}

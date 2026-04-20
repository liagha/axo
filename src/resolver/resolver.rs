use broccli::Color;
use crate::{
    data::{memory::replace, Identity, Module, Str},
    internal::{hash::Map, Artifact, RecordKind, Session, SessionError},
    format::Show,
    parser::{Element, ElementKind, Symbol, SymbolKind},
    resolver::{next_identity, scope::Scope, ErrorKind, ResolveError, Type},
    scanner::{Token, TokenKind},
    tracker::Span,
};

pub struct Resolver<'a> {
    pub active: Identity,
    pub scopes: Map<Identity, Scope>,
    pub registry: Map<Identity, Symbol<'a>>,
    pub input: Vec<Element<'a>>,
    pub errors: Vec<ResolveError<'a>>,
    pub variables: Vec<Option<Type<'a>>>,
    pub returns: Vec<Type<'a>>,
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
        }
    }
}

pub trait Resolvable<'a> {
    fn declare(&mut self, resolver: &mut Resolver<'a>);
    fn resolve(&mut self, resolver: &mut Resolver<'a>);
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

    pub fn candidates(&self, target: &Element<'a>) -> Vec<Symbol<'a>> {
        let Some(query) = target.target() else {
            return Vec::new();
        };

        let mut symbols = Vec::new();
        let mut current = Some(self.active());

        while let Some(scope) = current {
            for identity in &scope.symbols {
                if let Some(symbol) = self.registry.get(identity) {
                    if symbol.target() == Some(query.clone()) {
                        symbols.push(symbol.clone());
                    }
                }
            }
            current = scope.parent.and_then(|id| self.scopes.get(&id));
        }

        symbols.sort();
        symbols
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
                phantom: Default::default(),
            }])
        }
    }

    pub fn execute(session: &mut Session<'a>, keys: &[Identity]) {
        let mut source: Vec<_> = keys
            .iter()
            .copied()
            .filter(|key| {
                session.records.get(key).map_or(false, |r| r.kind == RecordKind::Source)
            })
            .collect();
        source.sort();

        Self::prepare(session, &source);
        Self::run_declare(session, &source);
        //Self::report(session);
        Self::run_resolve(session, &source);

        session
            .errors
            .extend(session.resolver.errors.drain(..).map(SessionError::Resolve));
    }

    fn prepare(session: &mut Session<'a>, source: &[Identity]) {
        let modules: Vec<_> = source
            .iter()
            .filter_map(|&identity| {
                let record = session.records.get_mut(&identity).unwrap();
                let name = Str::from(record.location.stem().unwrap().to_string());

                let existing = session.resolver.registry.iter().find_map(|(&id, symbol)| {
                    if matches!(symbol.kind, SymbolKind::Module(_)) && symbol.target() == Some(name.clone()) {
                        Some(id)
                    } else {
                        None
                    }
                });

                if let Some(target) = existing {
                    record.store(0, Artifact::Module(target));
                    return None;
                }

                let end = record.content.as_ref().map(|value| value.len() as u32).unwrap_or(0);
                let span = Span::range(identity, 0, end);
                let head = Element::new(
                    ElementKind::literal(Token::new(TokenKind::identifier(name), span)),
                    span,
                ).into();

                let mut symbol = Symbol::new(
                    SymbolKind::module(Module::new(head)),
                    span,
                );

                symbol.identity = identity;
                record.store(0, Artifact::Module(symbol.identity));
                Some(symbol)
            })
            .collect();

        for module in modules {
            session.resolver.insert(module);
        }
    }

    fn run_declare(session: &mut Session<'a>, source: &[Identity]) {
        for &key in source {
            let target = if let Some(Artifact::Module(m)) = session.records.get(&key).unwrap().fetch(0) { *m } else { continue };
            let mut module = session.resolver.registry.remove(&target).unwrap();
            let scope = replace(&mut module.scope, Box::from(Scope::new(None)));

            session.resolver.enter_scope(*scope);

            if let Some(Artifact::Elements(elements)) = session.records.get_mut(&key).unwrap().fetch_mut(2) {
                for element in elements.iter_mut() {
                    element.declare(&mut session.resolver);
                }
            }

            let active = session.resolver.active;
            session.resolver.exit();
            module.set_scope(session.resolver.scopes.remove(&active).unwrap());
            session.resolver.insert(module);
        }
    }

    #[allow(dead_code)]
    fn report(session: &mut Session<'a>) {
        if let Some(stencil) = session.get_stencil() {
            session.report_section(
                "Symbols",
                Color::Blue,
                session
                    .resolver
                    .collect()
                    .iter()
                    .map(|symbol| {
                        let children = symbol
                            .scope
                            .symbols
                            .iter()
                            .filter_map(|identity| session.resolver.get_symbol(*identity))
                            .collect::<Vec<_>>()
                            .format(stencil.clone())
                            .to_string();

                        format!(
                            "{}\n{}\n",
                            symbol.format(stencil.clone()),
                            children.indent(stencil.clone())
                        )
                    })
                    .collect::<Vec<String>>()
                    .join("\n"),
            )
        }
    }

    fn run_resolve(session: &mut Session<'a>, source: &[Identity]) {
        for &key in source {
            let target = if let Some(Artifact::Module(m)) = session.records.get(&key).unwrap().fetch(0) { *m } else { continue };
            let mut module = session.resolver.registry.remove(&target).unwrap();
            let scope = replace(&mut module.scope, Box::from(Scope::new(None)));

            session.resolver.enter_scope(*scope);

            if let Some(Artifact::Elements(elements)) = session.records.get_mut(&key).unwrap().fetch_mut(2) {
                for element in elements.iter_mut() {
                    element.resolve(&mut session.resolver);
                }
            }

            let active = session.resolver.active;
            session.resolver.exit();
            module.scope = Box::from(session.resolver.scopes.remove(&active).unwrap());
            session.resolver.insert(module);
        }
    }
}

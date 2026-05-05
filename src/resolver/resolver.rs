use crate::{
    data::{memory::replace, Identity, Module, Str},
    format::Show,
    internal::{hash::Map, Artifact, RecordKind, Session, SessionError},
    parser::{Element, ElementKind, Symbol, SymbolKind},
    resolver::{next_identity, scope::Scope, ErrorKind, ResolveError, Type},
    scanner::{Token, TokenKind},
    tracker::Span,
};
use broccli::Color;

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
        let identity = next_identity();
        self.scopes.insert(identity, Scope::new(Some(self.active)));
        self.active = identity;
    }

    pub fn enter_scope(&mut self, mut scope: Scope) {
        let identity = next_identity();
        scope.parent = Some(self.active);
        self.scopes.insert(identity, scope);
        self.active = identity;
    }

    pub fn exit(&mut self) {
        if let Some(parent) = self.active().parent {
            self.active = parent;
        }
    }

    pub fn nest<T>(&mut self, combinator: impl FnOnce(&mut Self) -> T) -> (T, Scope) {
        self.enter();
        let value = combinator(self);
        let active = self.active;
        self.exit();
        (value, self.scopes.remove(&active).unwrap())
    }

    pub fn within<T>(
        &mut self,
        scope: Scope,
        combinator: impl FnOnce(&mut Self) -> T,
    ) -> (T, Scope) {
        self.enter_scope(scope);
        let value = combinator(self);
        let active = self.active;
        self.exit();
        (value, self.scopes.remove(&active).unwrap())
    }

    pub fn insert(&mut self, symbol: Symbol<'a>) {
        let identity = symbol.identity;
        self.registry.insert(identity, symbol);
        self.active_mut().insert(identity);
    }

    pub fn get_symbol(&self, identity: Identity) -> Option<&Symbol<'a>> {
        self.registry.get(&identity)
    }

    fn walk<T>(&self, scope: Identity, seed: T, step: impl Fn(T, &Scope) -> T + Copy) -> T {
        let scope = self.scopes.get(&scope).unwrap();
        let seed = step(seed, scope);
        match scope.parent {
            Some(parent) => self.walk(parent, seed, step),
            None => seed,
        }
    }

    pub fn collect(&self) -> Vec<Symbol<'a>> {
        let mut symbols = self.walk(self.active, Vec::new(), |mut items, scope| {
            items.extend(
                scope
                    .symbols
                    .iter()
                    .filter_map(|identity| self.registry.get(identity).cloned()),
            );
            items
        });
        symbols.sort();
        symbols
    }

    pub fn find(&self, target: Identity) -> Option<Identity> {
        self.walk(self.active, None, |found, scope| {
            found.or_else(|| scope.has(target).then_some(target))
        })
    }

    pub fn exact(&self, target: &Element<'a>) -> Option<Symbol<'a>> {
        self.walk(self.active, None, |found, scope| {
            found.or_else(|| scope.exact(target, self))
        })
    }

    pub fn candidates(&self, target: &Element<'a>) -> Vec<Symbol<'a>> {
        let Some(query) = target.target() else {
            return Vec::new();
        };

        let mut symbols = self.walk(self.active, Vec::new(), |mut items, scope| {
            items.extend(scope.symbols.iter().filter_map(|identity| {
                self.registry
                    .get(identity)
                    .filter(|symbol| symbol.target() == Some(query.clone()))
                    .cloned()
            }));
            items
        });

        symbols.sort();
        symbols
    }

    pub fn undefined(&self, target: &Element<'a>) -> Vec<ResolveError<'a>> {
        vec![ResolveError {
            kind: ErrorKind::UndefinedSymbol {
                query: target.target().unwrap().clone(),
            },
            span: target.span.clone(),
            phantom: Default::default(),
        }]
    }

    pub fn lookup(&self, target: &Element<'a>) -> Result<Symbol<'a>, Vec<ResolveError<'a>>> {
        Self::builtin(target)
            .or_else(|| self.exact(target))
            .ok_or_else(|| self.undefined(target))
    }

    pub fn execute(session: &mut Session<'a>, keys: &[Identity]) {
        let mut source = keys
            .iter()
            .copied()
            .filter(|key| {
                session
                    .records
                    .get(key)
                    .is_some_and(|record| record.kind == RecordKind::Source)
            })
            .collect::<Vec<_>>();
        source.sort();

        Self::prepare(session, &source);
        Self::visit(session, &source, |element, resolver| {
            element.declare(resolver)
        });
        Self::visit(session, &source, |element, resolver| {
            element.resolve(resolver)
        });

        session
            .errors
            .extend(session.resolver.errors.drain(..).map(SessionError::Resolve));
    }

    fn module_name(record: &crate::internal::Record<'a>) -> Str<'a> {
        Str::from(record.location.stem().unwrap().to_string())
    }

    fn module_target(session: &Session<'a>, key: Identity) -> Option<Identity> {
        match session.records.get(&key)?.fetch(0)? {
            Artifact::Module(identity) => Some(*identity),
            _ => None,
        }
    }

    fn module_symbol(identity: Identity, name: Str<'a>, span: Span) -> Symbol<'a> {
        let head = Element::new(
            ElementKind::literal(Token::new(TokenKind::identifier(name), span)),
            span,
        )
            .into();

        let mut symbol = Symbol::new(SymbolKind::module(Module::new(head)), span);
        symbol.identity = identity;
        symbol
    }

    fn prepare(session: &mut Session<'a>, source: &[Identity]) {
        let modules = source
            .iter()
            .filter_map(|&identity| {
                let record = session.records.get_mut(&identity).unwrap();
                let name = Self::module_name(record);

                if let Some(target) =
                    session
                        .resolver
                        .registry
                        .iter()
                        .find_map(|(&target, symbol)| {
                            (matches!(symbol.kind, SymbolKind::Module(_))
                                && symbol.target() == Some(name.clone()))
                                .then_some(target)
                        })
                {
                    record.artifacts.insert(0, Artifact::Module(target));
                    return None;
                }

                let span = record.span(identity);
                let symbol = Self::module_symbol(identity, name, span);
                record.artifacts.insert(0, Artifact::Module(identity));
                Some(symbol)
            })
            .collect::<Vec<_>>();

        for module in modules {
            session.resolver.insert(module);
        }
    }

    fn visit(
        session: &mut Session<'a>,
        source: &[Identity],
        combinator: impl Fn(&mut Element<'a>, &mut Resolver<'a>) + Copy,
    ) {
        for &key in source {
            let Some(target) = Self::module_target(session, key) else {
                continue;
            };

            let mut module = session.resolver.registry.remove(&target).unwrap();
            let scope = replace(&mut module.scope, Box::new(Scope::new(None)));

            let (_, scope) = session.resolver.within(*scope, |resolver| {
                if let Some(Artifact::Elements(elements)) =
                    session.records.get_mut(&key).unwrap().fetch_mut(2)
                {
                    for element in elements {
                        combinator(element, resolver);
                    }
                }
            });

            module.set_scope(scope);
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
}
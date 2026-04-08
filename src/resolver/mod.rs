mod element;
mod error;
mod primitives;
mod resolver;
pub mod scope;
mod symbol;
mod traits;
mod typing;

use std::sync::Arc;
use std::time::Duration;
use broccli::Color;
pub use {resolver::*, scope::*};

pub(super) use {error::*, typing::*};

use crate::{
    data::{
        sync::{AtomicUsize, Ordering},
        Identity, Module, Str,
    },
    parser::{Element, ElementKind, Symbol, SymbolKind, Visibility},
    scanner::{Token, TokenKind},
    tracker::Span,
    reporter::Error,
};
use crate::combinator::{Action, Operation, Operator};
use crate::format::Show;
use crate::internal::platform::Lock;
use crate::internal::{CompileError, InputKind, Session};

pub static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn next_identity() -> Identity {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub type ResolveError<'error> = Error<'error, ErrorKind<'error>>;

impl<'source>
Action<
    'static,
    Operator<Arc<Lock<Session<'source>>>>,
    Operation<'source, Arc<Lock<Session<'source>>>>,
> for Resolver<'source>
{
    fn action(
        &self,
        operator: &mut Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) -> () {
        let mut guard = operator.store.write().unwrap();
        let session = &mut *guard;

        use crate::{
            data::memory::replace,
            resolver::{Resolvable, Scope},
        };

        let initial = session.errors.len();
        session.report_start("resolving");

        let mut keys: Vec<_> = session.records.keys().copied().collect();
        keys.sort();

        let modules: Vec<_> = keys
            .iter()
            .filter_map(|&identity| {
                let record = session.records.get_mut(&identity).unwrap();

                if record.kind != InputKind::Source {
                    return None;
                }

                let stem = Str::from(record.location.stem().unwrap().to_string());
                let span = Span::file(Str::from(record.location.to_string())).unwrap();

                let head = Element::new(
                    ElementKind::Literal(Token::new(TokenKind::Identifier(stem), span)),
                    span,
                )
                    .into();

                let mut symbol = Symbol::new(
                    SymbolKind::Module(Module::new(head)),
                    span,
                    Visibility::Public,
                );

                symbol.identity = identity;

                record.module = Some(symbol.identity);
                Some(symbol)
            })
            .collect();

        for module in modules {
            session.resolver.insert(module);
        }

        let mut source: Vec<_> = session
            .records
            .iter()
            .filter_map(|(&key, record)| {
                if record.kind == InputKind::Source && record.module.is_some() {
                    Some(key)
                } else {
                    None
                }
            })
            .collect();
        source.sort();

        for &key in &source {
            let target = session.records.get(&key).unwrap().module.unwrap();
            let mut module = session.resolver.get_symbol(target).unwrap().clone();
            let scope = replace(&mut module.scope, Scope::new(None));

            session.resolver.enter_scope(scope);

            let elements = session
                .records
                .get_mut(&key)
                .unwrap()
                .elements
                .as_mut()
                .unwrap();

            for element in elements.iter_mut() {
                element.declare(&mut session.resolver);
            }

            let active = session.resolver.active;
            session.resolver.exit();

            module.scope = session.resolver.scopes.remove(&active).unwrap();
            session.resolver.insert(module);
        }

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

        for &key in &source {
            let target = session.records.get(&key).unwrap().module.unwrap();
            let mut module = session.resolver.get_symbol(target).unwrap().clone();
            let scope = replace(&mut module.scope, Scope::new(None));

            session.resolver.enter_scope(scope);

            let elements = session
                .records
                .get_mut(&key)
                .unwrap()
                .elements
                .as_mut()
                .unwrap();

            for element in elements.iter_mut() {
                element.resolve(&mut session.resolver);
            }

            let active = session.resolver.active;
            session.resolver.exit();

            module.scope = session.resolver.scopes.remove(&active).unwrap();
            session.resolver.insert(module);
        }

        for &key in &source {
            let target = session.records.get(&key).unwrap().module.unwrap();
            let mut module = session.resolver.get_symbol(target).unwrap().clone();
            let scope = replace(&mut module.scope, Scope::new(None));

            session.resolver.enter_scope(scope);

            let elements = session
                .records
                .get_mut(&key)
                .unwrap()
                .elements
                .as_mut()
                .unwrap();

            for element in elements.iter_mut() {
                element.reify(&mut session.resolver);
            }

            let active = session.resolver.active;
            session.resolver.exit();

            module.scope = session.resolver.scopes.remove(&active).unwrap();
            session.resolver.insert(module);
        }

        session
            .errors
            .extend(session.resolver.errors.drain(..).map(CompileError::Resolve));

        let duration = Duration::from_nanos(session.timer.lap().unwrap());
        session.report_finish("resolving", duration, session.errors.len() - initial);

        if session.errors.is_empty() {
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }
        ()
    }
}

impl<'source> Default for Resolver<'source> {
    fn default() -> Self {
        Resolver::new()
    }
}
mod classifier;
mod element;
pub mod error;
mod parser;
mod symbol;
mod traits;

use std::sync::Arc;
use std::time::Duration;
use broccli::Color;
pub use {
    element::{Element, ElementKind},
    parser::Parser,
    symbol::{Symbol, SymbolKind, Visibility},
};

use {crate::reporter::Error, error::*};
use crate::combinator::{Action, Operation, Operator};
use crate::format::Show;
use crate::internal::platform::Lock;
use crate::internal::{CompileError, InputKind, Session};
use crate::tracker::Peekable;

pub type ParseError<'error> = Error<'error, ErrorKind<'error>>;

pub struct ParseAction;
impl<'source>
Action<
    'static,
    Operator<Arc<Lock<Session<'source>>>>,
    Operation<'source, Arc<Lock<Session<'source>>>>,
> for ParseAction
{
    fn action(
        &self,
        operator: &mut Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) -> () {
        let mut session = operator.store.write().unwrap();
        use crate::parser::Parser;

        let initial = session.errors.len();
        session.report_start("parsing");

        let mut keys: Vec<_> = session.records.keys().copied().collect();
        keys.sort();

        for key in keys {
            let (kind, hash, dirty, location, tokens) = {
                let record = session.records.get(&key).unwrap();
                (
                    record.kind.clone(),
                    record.hash,
                    record.dirty,
                    record.location,
                    record.tokens.clone(),
                )
            };

            if kind != InputKind::Source {
                continue;
            }

            if !dirty {
                if let Some(elements) = session.cache::<Vec<Element>>("elements", hash, None) {
                    session.records.get_mut(&key).unwrap().elements = Some(elements);
                    continue;
                }
            }

            let mut parser = Parser::new(location);
            parser.set_input(tokens.unwrap());
            parser.parse();

            if let Some(stencil) = session.get_stencil() {
                session.report_section(
                    "Elements",
                    Color::Cyan,
                    parser.output.format(stencil).to_string(),
                );
            }

            session.errors.extend(
                parser
                    .errors
                    .iter()
                    .map(|error| CompileError::Parse(error.clone())),
            );

            session.records.get_mut(&key).unwrap().elements =
                session.cache("elements", hash, Some(parser.output));
        }

        let duration = Duration::from_nanos(session.timer.lap().unwrap());
        session.report_finish("parsing", duration, session.errors.len() - initial);

        if session.errors.is_empty() {
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }
        ()
    }
}


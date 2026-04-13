mod classifier;
mod element;
pub mod error;
mod parser;
mod symbol;
mod traits;

pub use {
    element::{Element, ElementKind},
    parser::Parser,
    symbol::{Symbol, SymbolKind, Visibility},
    error::*,
};

use {
    broccli::Color,
    
    crate::{
        reporter::Error,
        combinator::{Action, Operation, Operator},
        data::{
            Identity,
            memory::Arc,
        },
        format::Show,
        internal::{
            platform::Lock,
            time::Duration,
            SessionError, RecordKind, Session,
        },
        tracker::Peekable,
    },
};

pub type ParseError<'error> = Error<'error, ErrorKind<'error>>;

pub fn parse<'source>(session: &mut Session<'source>, keys: &[Identity]) {
    use crate::parser::Parser;

    for &key in keys {
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

        if kind != RecordKind::Source {
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
                .map(|error| SessionError::Parse(error.clone())),
        );

        session.records.get_mut(&key).unwrap().elements =
            session.cache("elements", hash, Some(parser.output));
    }
}

impl<'source>
Action<
    'static,
    Operator<Arc<Lock<Session<'source>>>>,
    Operation<'source, Arc<Lock<Session<'source>>>>,
> for Parser<'source>
{
    fn action(
        &self,
        operator: &mut Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) -> () {
        let mut session = operator.store.write().unwrap();

        let initial = session.errors.len();
        session.report_start("parsing");

        let mut keys: Vec<_> = session.records.keys().copied().collect();
        keys.sort();
        parse(&mut session, &keys);

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

impl<'source> Default for Parser<'source> {
    fn default() -> Self {
        Parser::new(crate::tracker::Location::Entry(crate::data::Str::from(file!())))
    }
}

mod character;
mod classifier;
mod error;
mod operator;
mod punctuation;
mod scanner;
mod token;
mod traits;

use std::sync::Arc;
use std::time::Duration;
use broccli::Color;
pub use {character::Character, operator::*, punctuation::*, scanner::Scanner, token::*};

use {
    crate::reporter::Error,
    error::*
};
use crate::combinator::{Action, Operation};
use crate::format::Show;
use crate::internal::platform::Lock;
use crate::internal::{CompileError, InputKind, Session};

pub type ScanError<'error> = Error<'error, ErrorKind<'error>>;

impl<'source>
Action<
    'static,
    crate::combinator::Operator<Arc<Lock<Session<'source>>>>,
    Operation<'source, Arc<Lock<Session<'source>>>>,
> for Scanner<'source>
{
    fn action(
        &self,
        operator: &mut crate::combinator::Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) -> () {
        let mut session = operator.store.write().unwrap();
        use crate::scanner::Scanner;

        let initial = session.errors.len();
        session.report_start("scanning");

        let mut keys: Vec<_> = session.records.keys().copied().collect();
        keys.sort();

        for key in keys {
            let (kind, hash, dirty, location) = {
                let record = session.records.get(&key).unwrap();
                (
                    record.kind.clone(),
                    record.hash,
                    record.dirty,
                    record.location,
                )
            };

            if kind != InputKind::Source {
                continue;
            }

            if !dirty {
                if let Some(tokens) = session.cache::<Vec<Token>>("tokens", hash, None) {
                    session.records.get_mut(&key).unwrap().tokens = Some(tokens);
                    continue;
                }
            }

            let content = match location.get_value() {
                Ok(content) => content,
                Err(error) => {
                    let kind = ErrorKind::Tracking(error.clone());
                    let error = ScanError::new(kind, error.span);

                    session.errors.push(CompileError::Scan(error));
                    continue;
                }
            };

            let position = crate::tracker::Position::new(location);
            let mut scanner = Scanner::new(position, content);

            scanner.scan();

            if let Some(stencil) = session.get_stencil() {
                session.report_section(
                    "Tokens",
                    Color::Cyan,
                    scanner.output.format(stencil).to_string(),
                );
            }

            session.errors.extend(
                scanner
                    .errors
                    .iter()
                    .map(|error| CompileError::Scan(error.clone())),
            );

            session.records.get_mut(&key).unwrap().tokens =
                session.cache("tokens", hash, Some(scanner.output));
        }

        let duration = Duration::from_nanos(session.timer.lap().unwrap());
        session.report_finish("scanning", duration, session.errors.len() - initial);

        if session.errors.is_empty() {
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }
        ()
    }
}

impl<'source> Default for Scanner<'source> {
    fn default() -> Self {
        let location = crate::tracker::Location::Entry(crate::data::Str::from(file!()));
        let position = crate::tracker::Position::new(location);

        Scanner::new(position, crate::data::Str::from(""))
    }
}
mod analysis;
mod analyzer;
mod element;
mod error;
mod traits;

use std::sync::Arc;
use std::time::Duration;
use broccli::Color;
pub use {analysis::*, analyzer::*};

pub(crate) use error::*;
use crate::combinator::{Action, Operation, Operator};
use crate::format::Show;
use crate::internal::platform::Lock;
use crate::internal::{CompileError, InputKind, Session};
use crate::reporter::Error;

pub type AnalyzeError<'error> = Error<'error, ErrorKind<'error>>;

impl<'source>
Action<
    'static,
    Operator<Arc<Lock<Session<'source>>>>,
    Operation<'source, Arc<Lock<Session<'source>>>>,
> for Analyzer<'source>
{
    fn action(
        &self,
        operator: &mut Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) -> () {
        let mut session = operator.store.write().unwrap();
        use crate::analyzer::{Analysis, Analyzer};

        let initial = session.errors.len();

        session.report_start("analyzing");

        let mut keys: Vec<_> = session
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
        keys.sort();

        for &key in &keys {
            let (hash, dirty, elements) = {
                let record = session.records.get(&key).unwrap();
                (record.hash, record.dirty, record.elements.clone())
            };

            if !dirty {
                if let Some(analyses) = session.cache::<Vec<Analysis>>("analyses", hash, None) {
                    session.records.get_mut(&key).unwrap().analyses = Some(analyses);
                    continue;
                }
            }

            let mut analyzer = Analyzer::new(elements.unwrap());
            analyzer.analyze(&mut session.resolver);

            if let Some(stencil) = session.get_stencil() {
                session.report_section(
                    "Analysis",
                    Color::Blue,
                    analyzer.output.format(stencil).to_string(),
                );
            }

            session.errors.extend(
                analyzer
                    .errors
                    .iter()
                    .map(|error| CompileError::Analyze(error.clone())),
            );

            session.records.get_mut(&key).unwrap().analyses =
                session.cache("analyses", hash, Some(analyzer.output));
        }

        let duration = Duration::from_nanos(session.timer.lap().unwrap());
        session.report_finish("analyzing", duration, session.errors.len() - initial);

        if session.errors.is_empty() {
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }
        ()
    }
}

impl<'source> Default for Analyzer<'source> {
    fn default() -> Self {
        Analyzer::new(Vec::new())
    }
}

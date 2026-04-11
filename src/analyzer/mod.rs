mod analysis;
mod analyzer;
mod element;
mod error;

use {
    broccli::Color,
    crate::{
        format::Show,
        reporter::Error,
        internal::{
            time::Duration,
            platform::Lock,
            SessionError, RecordKind, Session,
        },
        data::{
            Identity,
            memory::Arc,
        },
        combinator::{Action, Operation, Operator},
    },
};

pub use {analysis::*, analyzer::*};

pub(crate) use error::*;

pub type AnalyzeError<'error> = Error<'error, ErrorKind<'error>>;

pub fn analyze<'source>(session: &mut Session<'source>, keys: &[Identity]) {
    use crate::analyzer::{Analysis, Analyzer};

    for &key in keys {
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
                .map(|error| SessionError::Analyze(error.clone())),
        );

        session.records.get_mut(&key).unwrap().analyses =
            session.cache("analyses", hash, Some(analyzer.output));
    }
}

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

        let initial = session.errors.len();

        session.report_start("analyzing");

        let mut keys: Vec<_> = session
            .records
            .iter()
            .filter_map(|(&key, record)| {
                if record.kind == RecordKind::Source && record.module.is_some() {
                    Some(key)
                } else {
                    None
                }
            })
            .collect();
        keys.sort();
        analyze(&mut session, &keys);

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

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
            SessionError, RecordKind, Session, Artifact,
        },
        data::{
            Identity,
            memory::Arc,
        },
        combinator::{Action, Operation, Operator},
    },
};

pub use {analysis::*, analyzer::*, error::*};

pub type AnalyzeError<'error> = Error<'error, ErrorKind<'error>>;

pub fn analyze<'source>(session: &mut Session<'source>, keys: &[Identity]) {
    use crate::analyzer::{Analysis, Analyzer};

    for &key in keys {
        let (hash, dirty, elements) = {
            let record = session.records.get(&key).unwrap();
            let elements = if let Some(Artifact::Elements(elements)) = record.fetch(2) {
                Some(elements.clone())
            } else {
                None
            };
            (record.hash, record.dirty, elements)
        };

        if !dirty {
            if let Some(mut analyses) = session.cache::<Vec<Analysis>>("analyses", hash, None) {
                analyses.shrink_to_fit();
                let record = session.records.get_mut(&key).unwrap();
                record.store(3, Artifact::Analyses(analyses));
                record.artifacts.remove(&2);
                continue;
            }
        }

        let mut analyzer = Analyzer::new(elements.unwrap_or_default());
        analyzer.analyze(&mut session.resolver);

        if let Some(stencil) = session.get_stencil() {
            session.report_section(
                "Analysis",
                Color::Blue,
                analyzer.output.format(stencil).to_string(),
            );
        }

        analyzer.output.shrink_to_fit();

        session.errors.extend(
            analyzer
                .errors
                .iter()
                .map(|error| SessionError::Analyze(error.clone())),
        );

        if let Some(analyses) = session.cache("analyses", hash, Some(analyzer.output)) {
            let record = session.records.get_mut(&key).unwrap();
            record.store(3, Artifact::Analyses(analyses));
            record.artifacts.remove(&2);
        }
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
                if record.kind == RecordKind::Source && record.fetch(0).is_some() {
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
mod analysis;
mod analyzer;
mod element;
mod error;

pub use {analysis::*, analyzer::*, error::*};

use crate::{
    combinator::{Action, Operation, Operator},
    data::memory::Arc,
    internal::{platform::Lock, time::Duration, Session},
    reporter::Error,
};

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
    ) {
        let mut session = operator.store.write().unwrap();
        let initial = session.errors.len();

        session.report_start("analyzing");

        let mut keys: Vec<_> = session.records.keys().copied().collect();
        keys.sort();

        Analyzer::execute(&mut session, &keys);

        let now = session.timer.elapsed();
        let sum: Duration = session.laps.iter().copied().sum();
        let duration = now.saturating_sub(sum);
        
        session.report_finish("analyzing", duration, session.errors.len() - initial);

        if session.errors.is_empty() {
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }
    }
}

impl<'source> Default for Analyzer<'source> {
    fn default() -> Self {
        Analyzer::new(Vec::new())
    }
}

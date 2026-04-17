mod element;
mod error;
mod primitives;
mod resolver;
pub mod scope;
mod symbol;
mod traits;
mod typing;

pub use {resolver::*, scope::*, typing::*, error::*};

use crate::{
    combinator::{Action, Operation, Operator},
    data::{memory::Arc, sync::{AtomicUsize, Ordering}, Identity},
    internal::{platform::Lock, time::Duration, Session},
    reporter::Error,
};

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
    ) {
        let mut guard = operator.store.write().unwrap();
        let session = &mut *guard;
        let initial = session.errors.len();

        session.report_start("resolving");

        let mut keys: Vec<_> = session.records.keys().copied().collect();
        keys.sort();

        Resolver::execute(session, &keys);

        let now = session.timer.elapsed();
        let sum: Duration = session.laps.iter().copied().sum();
        let duration = now.saturating_sub(sum);

        session.report_finish("resolving", duration, session.errors.len() - initial);

        if session.errors.is_empty() {
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }
    }
}

impl<'source> Default for Resolver<'source> {
    fn default() -> Self {
        Resolver::new()
    }
}

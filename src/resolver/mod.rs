mod element;
mod error;
mod primitives;
mod resolver;
pub mod scope;
mod symbol;
mod traits;
mod typing;

pub use {error::*, resolver::*, scope::*, typing::*};

use crate::{
    combinator::{Combinator, Operation, Operator},
    data::{
        memory::Arc,
        sync::{AtomicUsize, Ordering},
        Identity,
    },
    internal::{platform::Lock, Session},
    reporter::Error,
};

pub static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn next_identity() -> Identity {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub type ResolveError<'error> = Error<'error, ErrorKind<'error>>;

impl<'source>
    Combinator<
        'static,
        Operator<Arc<Lock<Session<'source>>>>,
        Operation<'source, Arc<Lock<Session<'source>>>>,
    > for Resolver<'source>
{
    fn combinator(
        &self,
        operator: &mut Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) {
        let mut guard = operator.store.write().unwrap();
        let session = &mut *guard;

        let mut keys: Vec<_> = session.records.keys().copied().collect();
        keys.sort();

        Resolver::execute(session, &keys);

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

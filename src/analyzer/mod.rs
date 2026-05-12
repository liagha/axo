mod analysis;
mod analyzer;
mod element;
mod error;

pub use {analysis::*, analyzer::*, error::*};

pub type AnalyzeError<'error> = Error<'error, ErrorKind<'error>>;

use {
    crate::{internal::session::Store, reporter::Error},
    chaint::{Combinator, Operation, Operator},
};

impl<'op, 'source>
    Combinator<
        'static,
        (
            &'op mut Operator<Store<'source>>,
            &'op mut Operation<'source, Store<'source>>,
        ),
    > for Analyzer<'source>
{
    fn combinator(
        &self,
        joint: &mut (
            &'op mut Operator<Store<'source>>,
            &'op mut Operation<'source, Store<'source>>,
        ),
    ) {
        let (operator, operation) = (&mut joint.0, &mut joint.1);

        let mut session = operator.store.write().unwrap();
        let mut keys: Vec<_> = session.records.keys().copied().collect();
        keys.sort();

        Analyzer::execute(&mut session, &keys);

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

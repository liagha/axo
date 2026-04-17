mod character;
mod classifier;
mod error;
mod operator;
mod punctuation;
mod scanner;
mod token;
mod traits;

pub use {character::Character, operator::*, punctuation::*, scanner::Scanner, token::*, error::*};

pub type ScanError<'error> = Error<'error, ErrorKind<'error>>;

use {
    crate::{
        combinator::{Action, Operation},
        data::memory::Arc,
        internal::{platform::Lock, time::Duration, Session},
    },
};
use crate::reporter::Error;

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
    ) {
        let mut session = operator.store.write().unwrap();
        let initial = session.errors.len();

        session.report_start("scanning");

        let mut keys: Vec<_> = session.records.keys().copied().collect();
        keys.sort();

        Scanner::execute(&mut session, &keys);

        let duration = Duration::from_nanos(session.timer.lap().unwrap());
        session.report_finish("scanning", duration, session.errors.len() - initial);

        if session.errors.is_empty() {
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }
    }
}

impl<'source> Default for Scanner<'source> {
    fn default() -> Self {
        let position = crate::tracker::Position::new(0);
        Scanner::new(position, crate::data::Str::from(""))
    }
}

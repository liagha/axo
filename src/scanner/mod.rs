mod character;
mod error;
mod formation;
mod operator;
mod punctuation;
mod scanner;
mod token;
mod traits;

pub use {
    character::Character, error::*, operator::*, punctuation::*, scanner::Scanner, token::*,
};

pub type ScanError<'error> = Error<'error, ErrorKind<'error>>;

use crate::{
    combinator::{Combinator, Operation},
    data::memory::Arc,
    internal::{platform::Lock, Session},
    reporter::Error,
};

impl<'source>
Combinator<
    'static,
    crate::combinator::Operator<Arc<Lock<Session<'source>>>>,
    Operation<'source, Arc<Lock<Session<'source>>>>,
> for Scanner<'source>
{
    fn combinator(
        &self,
        operator: &mut crate::combinator::Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) {
        let mut session = operator.store.write().unwrap();
        let mut keys: Vec<_> = session.records.keys().copied().collect();
        keys.sort();

        Scanner::execute(&mut session, &keys);

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

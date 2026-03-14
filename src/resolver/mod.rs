mod assessor;
mod element;
mod error;
mod hint;
mod resolver;
pub mod scope;
mod symbol;
mod traits;
mod primitives;
mod types;

pub use resolver::*;

pub(super) use {error::*, hint::*, types::*};

use crate::{
    data::{
        sync::{AtomicUsize, Ordering},
        Identity,
    },
    reporter::{Error, Hint},
};

pub static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn next_identity() -> Identity {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub type ResolveError<'error> = Error<'error, ErrorKind<'error>, HintKind<'error>>;
pub type ResolveHint<'hint> = Hint<HintKind<'hint>>;

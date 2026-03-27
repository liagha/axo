mod element;
mod error;
mod primitives;
mod resolver;
pub mod scope;
mod symbol;
mod traits;
mod typing;

pub use {resolver::*, scope::*};

pub(super) use {error::*, typing::*};

use crate::{
    data::{
        sync::{AtomicUsize, Ordering},
        Identity,
    },
    reporter::Error,
};

pub static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn next_identity() -> Identity {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub type ResolveError<'error> = Error<'error, ErrorKind<'error>>;

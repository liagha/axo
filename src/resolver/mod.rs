mod error;
mod hint;
mod matcher;
mod resolver;
pub mod scope;
#[cfg(feature = "checker")]
pub mod checker;

pub use resolver::Resolver;

pub(super) use {
    error::*,
    hint::*,
};

use crate::reporter::{Error, Hint};

pub type ResolveError<'error> = Error<'error, ErrorKind<'error>, HintKind<'error>>;
pub type ResolveHint<'hint> = Hint<HintKind<'hint>>;
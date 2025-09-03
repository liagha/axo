mod error;
mod hint;
mod matcher;
mod resolver;
pub mod analyzer;
pub mod checker;
pub mod scope;
mod base;

pub use {
    resolver::Resolver,
};


pub(super) use {
    error::*,
    hint::*,
};

use crate::reporter::{Error, Hint};

pub type ResolveError<'error> = Error<'error, ErrorKind<'error>, HintKind<'error>>;
pub type ResolveHint<'hint> = Hint<HintKind<'hint>>;
pub mod analyzer;
mod assessor;
pub mod checker;
mod element;
mod error;
mod hint;
mod primitives;
mod resolver;
pub mod scope;
mod symbol;

pub use resolver::*;

pub(super) use {error::*, hint::*};

use crate::reporter::{Error, Hint};

pub type ResolveError<'error> = Error<'error, ErrorKind<'error>, HintKind<'error>>;
pub type ResolveHint<'hint> = Hint<HintKind<'hint>>;

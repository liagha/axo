mod error;
mod hint;
mod assessor;
mod resolver;
pub mod analyzer;
pub mod checker;
pub mod scope;
mod validator;
mod element;
mod symbol;

pub use {
    resolver::*,
};

pub(super) use {
    error::*,
    hint::*,
};

use {
    crate::{
        reporter::{
            Error, Hint,
        },
    }
};

pub type ResolveError<'error> = Error<'error, ErrorKind<'error>, HintKind<'error>>;
pub type ResolveHint<'hint> = Hint<HintKind<'hint>>;
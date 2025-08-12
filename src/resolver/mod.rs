mod error;
mod hint;
mod matcher;
mod resolver;
pub mod scope;

pub use {
    resolver::Resolver,
};

pub(super) use {
    error::*,
    hint::*,
};

use {
    crate::{
        reporter::Error,
    },
};

pub type ResolveError<'error> = Error<'error, ErrorKind, String, ResolveHint<'error>>;
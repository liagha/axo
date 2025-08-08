pub mod scope;
mod matcher;
mod resolver;
mod error;
mod hint;

use {
    crate::{
        error::Error,
        resolver::{
            error::ErrorKind,
            hint::ResolveHint,
        },
    },
};

pub use {
    resolver::Resolver,
};

pub type ResolveError<'error> = Error<'error, ErrorKind<'error>, String, ResolveHint<'error>>;
pub mod scope;
mod matcher;
mod resolver;
mod error;
mod brand;
mod hint;

use {
    crate::{
        axo_error::Error,
        axo_resolver::{
            error::ErrorKind,
            hint::ResolveHint,
        },
    },
};

pub use {
    resolver::Resolver,
};

pub type ResolveError<'error> = Error<'error, ErrorKind<'error>, String, ResolveHint<'error>>;
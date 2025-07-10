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
        },
    },
};

pub use resolver::Resolver;
use crate::axo_resolver::hint::ResolveHint;

pub type ResolveError = Error<ErrorKind, String, ResolveHint>;
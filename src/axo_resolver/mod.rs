
pub mod error;
pub mod scope;
mod matcher;
mod resolver;

use {
    crate::{
        axo_errors::Error,
        axo_resolver::{
            matcher::Labeled,
            error::ErrorKind,
        },
    },
};

pub use resolver::Resolver;

pub type ResolveError = Error<ErrorKind>;

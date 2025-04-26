pub mod scope;
pub mod validation;
mod matcher;
mod resolver;
mod error;

use {
    crate::{
        axo_errors::Error,
        axo_resolver::{
            error::ErrorKind,
        },
    },
};

pub use resolver::Resolver;

pub type ResolveError = Error<ErrorKind>;
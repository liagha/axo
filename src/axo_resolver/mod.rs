pub mod scope;
mod assessor;
mod resolver;
mod error;
mod brand;

use {
    crate::{
        axo_error::Error,
        axo_resolver::{
            error::ErrorKind,
        },
    },
};

pub use resolver::Resolver;

pub type ResolveError = Error<ErrorKind>;
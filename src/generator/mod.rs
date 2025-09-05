mod error;
mod generator;
mod backend;
mod inkwell;

pub use {
    backend::{
        Backend,
    },
    inkwell::{
        Inkwell
    },
    generator::{
        Generator,
    },
};

pub(super) use {
    error::*,
};

use {
    crate::{
        reporter::Error,
    },
};

pub type GenerateError<'error> = Error<'error, ErrorKind>;
mod error;
mod generator;

pub use {
    generator::{
        Generator,
        Inkwell,
        Backend,
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
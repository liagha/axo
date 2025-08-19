mod checker;
mod primitive;
mod types;
mod error;

pub use {
    checker::Checker,
    types::*,
};

pub(super) use {
    error::*,
};

use {
    crate::{
        reporter::Error,
    },
};

pub type CheckError<'error> = Error<'error, ErrorKind<'error>>;
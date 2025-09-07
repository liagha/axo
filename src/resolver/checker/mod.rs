mod types;
mod checker;
mod error;
mod element;
mod symbol;

pub use {
    types::*,
    checker::*,
};

pub(crate) use {
    error::*,
};

use {
    crate::{
        reporter::Error,
    },
};

pub type CheckError<'error> = Error<'error, ErrorKind<'error>>;
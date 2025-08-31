mod types;
mod checker;
mod error;

pub(super) use {
    error::*,
};

use {
    crate::{
        reporter::Error,
    },
};

pub type CheckError<'error> = Error<'error, ErrorKind<'error>>;
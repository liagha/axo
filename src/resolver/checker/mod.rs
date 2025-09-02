mod types;
mod checker;
mod error;

pub(crate) use {
    error::*,
};

use {
    crate::{
        reporter::Error,
    },
};

pub type CheckError<'error> = Error<'error, ErrorKind<'error>>;
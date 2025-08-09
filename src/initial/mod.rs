mod error;
mod initializer;

pub use {
    initializer::{
        Initializer,
        Preference,
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

pub type InitialError<'error> = Error<'error, ErrorKind<'error>>;
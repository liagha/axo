mod error;
mod initializer;
mod preference;

pub use {
    initializer::{
        Initializer,
    },
    preference::{
        Preference,
    }
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
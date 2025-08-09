use {
    crate::{
        reporter::Error,
        initial::error::ErrorKind,
    }
};

mod error;
pub mod initializer;

pub type InitialError<'error> = Error<'error, ErrorKind<'error>>;
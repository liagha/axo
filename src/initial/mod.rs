use {
    crate::{
        error::Error,
        initial::error::ErrorKind,
    }
};

pub mod initializer;
mod error;

pub type InitialError<'error> = Error<'error, ErrorKind<'error>>;
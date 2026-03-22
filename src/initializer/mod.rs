mod error;
mod initializer;
mod directive;
mod traits;

pub use {initializer::Initializer};


use {
    crate::reporter::Error,
    error::*,
};

pub type InitializeError<'error> = Error<'error, ErrorKind<'error>>;

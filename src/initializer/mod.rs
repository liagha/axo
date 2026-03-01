mod error;
mod initializer;
mod preference;
mod traits;

pub use {initializer::Initializer, preference::Preference};


use {
    crate::reporter::Error,
    error::*,
};

pub type InitializeError<'error> = Error<'error, ErrorKind<'error>>;

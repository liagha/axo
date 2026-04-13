mod directive;
mod error;
mod initializer;
mod traits;

pub use {initializer::Initializer, error::*};

use {crate::reporter::Error};

pub type InitializeError<'error> = Error<'error, ErrorKind<'error>>;

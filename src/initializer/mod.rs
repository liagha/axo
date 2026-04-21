mod directive;
mod error;
mod initializer;
mod traits;

pub use {error::*, initializer::Initializer};

use crate::reporter::Error;

pub type InitializeError<'error> = Error<'error, ErrorKind<'error>>;

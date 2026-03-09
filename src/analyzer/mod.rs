mod analysis;
mod analyzer;
mod element;
mod error;
mod traits;

pub use {analysis::*, analyzer::*};

pub(crate) use error::*;

use crate::reporter::Error;

pub type CheckError<'error> = Error<'error, ErrorKind<'error>>;

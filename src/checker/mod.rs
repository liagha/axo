mod checker;
mod element;
mod error;
mod symbol;
mod types;
mod traits;

pub use {checker::*, types::*};

pub(crate) use {error::*}; 

use crate::reporter::Error;

pub type CheckError<'error> = Error<'error, ErrorKind<'error>>;

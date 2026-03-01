mod checker;
mod element;
mod error;
mod semantics;
mod symbol;
mod types;

pub use {checker::*, types::*};

pub(crate) use {error::*, semantics::*};

use crate::reporter::Error;

pub type CheckError<'error> = Error<'error, ErrorKind<'error>>;

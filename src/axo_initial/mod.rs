use crate::axo_error::Error;
use crate::axo_initial::error::ErrorKind;

pub mod initializer;
mod error;

pub type InitialError = Error<ErrorKind>;
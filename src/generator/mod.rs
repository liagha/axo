mod backend;
mod error;
mod generator;
mod inkwell;

pub use {backend::Backend, error::ErrorKind, generator::Generator, inkwell::Inkwell};

pub(super) use error::*;

use crate::reporter::Error;

pub type GenerateError<'error> = Error<'error, ErrorKind>;

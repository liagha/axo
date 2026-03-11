mod backend;
mod generator;
mod inkwell;

pub use {backend::Backend, generator::Generator, inkwell::Inkwell};

use crate::reporter::Error;
pub use self::inkwell::error::*;

pub type GenerateError<'error> = Error<'error, ErrorKind<'error>>;

mod backend;
mod generator;
mod inkwell;

pub use {backend::Backend, inkwell::Generator};

use crate::reporter::Error;
pub use self::inkwell::error::*;

pub type GenerateError<'error> = Error<'error, ErrorKind<'error>>;

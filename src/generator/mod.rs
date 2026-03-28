mod backend;
mod generator;
mod inkwell;

pub use {backend::Backend, inkwell::Generator};

pub use self::inkwell::error::*;
use crate::reporter::Error;

pub type GenerateError<'error> = Error<'error, ErrorKind<'error>>;


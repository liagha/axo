mod backend;
mod error;
mod generator;
mod inkwell;

pub use {backend::Backend, error::ErrorKind, generator::Generator, inkwell::Inkwell};

use {
    crate::reporter::Error,
};


pub type GenerateError<'error> = Error<'error, ErrorKind>;

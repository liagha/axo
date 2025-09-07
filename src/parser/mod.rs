mod element;
pub mod error;
mod parser;
mod symbolic;
mod traits;
mod classifier;

pub use {
    element::{Element, ElementKind},
    parser::Parser,
    symbolic::{Symbol, SymbolKind},
};

pub(super) use error::*;

use crate::reporter::Error;

pub type ParseError<'error> = Error<'error, ErrorKind<'error>>;

mod classifier;
mod element;
pub mod error;
mod parser;
mod symbol;
mod traits;

pub use {
    element::{Element, ElementKind},
    parser::Parser,
    symbol::{Specifier, Symbol, SymbolKind, Visibility},
};

pub(super) use error::*;

use crate::reporter::Error;

pub type ParseError<'error> = Error<'error, ErrorKind<'error>>;

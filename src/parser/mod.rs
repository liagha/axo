mod classifier;
mod element;
pub mod error;
mod parser;
mod symbol;
mod traits;

pub use {
    element::{Element, ElementKind},
    parser::Parser,
    symbol::{Symbol, SymbolKind, Visibility},
};

use {crate::reporter::Error, error::*};

pub type ParseError<'error> = Error<'error, ErrorKind<'error>>;

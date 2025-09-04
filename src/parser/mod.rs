mod core;
mod delimited;
mod element;
pub mod error;
mod parser;
mod statement;
mod symbol;
mod symbolic;
mod traits;

pub use {
    element::{Element, ElementKind},
    parser::Parser,
    symbol::Symbol,
    symbolic::SymbolKind,
};

pub(super) use {
    error::*,  
};

use {
    crate::{
        reporter::Error,
    },
};

pub type ParseError<'error> = Error<'error, ErrorKind<'error>>;

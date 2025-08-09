mod core;
mod delimited;
mod element;
pub mod error;
mod format;
mod parser;
mod statement;
mod symbol;
mod symbolic;
mod traits;

pub use {
    element::{Element, ElementKind},
    parser::Parser,
    symbol::Symbol,
    symbolic::Symbolic,
};

pub(super) use {
    error::*,  
};

use {
    crate::{
        reporter::Error,
    },
};

pub type ParseError<'error> = Error<'error, ErrorKind>;

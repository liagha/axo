#![allow(unused_imports)]
mod parser;
mod symbol;
mod format;
mod traits;
mod element;
mod core;
mod delimited;
pub mod error;
mod statement;
mod symbolic;

pub use {
    element::{Element, ElementKind},
    symbol::{Symbol},
    symbolic::Symbolic,
    parser::Parser
};

use {
    crate::{
        axo_error::Error,    
    },
    
    error::ErrorKind,
};

pub type ParseError<'error> = Error<'error, ErrorKind>;

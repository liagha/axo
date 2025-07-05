#![allow(unused_imports)]
mod parser;
mod symbol;
mod format;
mod traits;
mod element;
mod core;
mod delimited;
pub mod error;

pub use {
    element::{Element, ElementKind},
    symbol::{Symbol, SymbolKind},
    parser::Parser
};

use {
    crate::{
        axo_error::Error,    
    },
    
    error::ErrorKind,
};

pub type ParseError = Error<ErrorKind>;

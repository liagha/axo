#![allow(unused_imports)]
mod parser;
mod item;
mod format;
mod traits;
mod element;
mod core;
mod delimited;
pub mod error;

pub use {
    element::{Element, ElementKind},
    item::{Item, ItemKind},
    parser::Parser
};

use {
    crate::{
        axo_error::Error,    
    },
    
    error::ErrorKind,
};

pub type ParseError = Error<ErrorKind>;

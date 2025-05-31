#![allow(unused_imports)]
mod parser;
mod item;
mod format;
mod traits;
pub mod error;
mod element;
mod core;
mod delimited;

pub use {
    crate::axo_error::Error,
    element::{Element, ElementKind},
    item::{Item, ItemKind},
    parser::Parser
};

use crate::axo_parser::error::ErrorKind;

pub type ParseError = Error<ErrorKind>;

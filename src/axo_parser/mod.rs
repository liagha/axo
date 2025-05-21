#![allow(unused_imports)]
mod parser;
mod statement;
mod core;
mod item;
mod fmt;
mod delimiter;
mod traits;
pub(crate) mod error;
mod element;

pub use {
    statement::ControlFlow,
    element::{Element, ElementKind},
    item::{Item, ItemKind},
    parser::Parser,
    core::Primary,

    crate::{
        axo_errors::Error,
    }
};

use {
    crate::{
        axo_parser::error::ErrorKind
    }
};

pub type ParseError = Error<ErrorKind>;

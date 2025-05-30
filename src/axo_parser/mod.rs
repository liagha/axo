#![allow(unused_imports)]
mod parser;
mod statement;
mod core;
mod item;
mod format;
mod delimiter;
mod traits;
pub mod error;
mod element;

pub use {
    statement::ControlFlow,
    element::{Element, ElementKind},
    item::{Item, ItemKind},
    parser::Parser,
    core::Primary,

    crate::{
        axo_error::Error,
    }
};

use {
    crate::{
        axo_parser::error::ErrorKind
    }
};

pub type ParseError = Error<ErrorKind>;

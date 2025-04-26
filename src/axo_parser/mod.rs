#![allow(unused_imports)]
mod parser;
mod statement;
mod expression;
mod primary;
mod composite;
mod item;
mod fmt;
mod delimiter;
mod traits;
mod error;
mod element;

pub use {
    statement::ControlFlow,
    element::{Element, ElementKind},
    item::{Item, ItemKind},
    parser::Parser,
    composite::Composite,
    primary::Primary,

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

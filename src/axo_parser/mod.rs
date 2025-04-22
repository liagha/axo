#![allow(unused_imports)]
mod parser;
mod statement;
mod expression;
mod error;
mod primary;
mod composite;
mod item;
mod fmt;
mod delimiter;
mod traits;

pub use {
    statement::ControlFlow,
    expression::{Expr, ExprKind},
    item::{Item, ItemKind},
    parser::Parser,
    composite::Composite,
    primary::Primary,

    crate::{
        axo_errors::Error,
        axo_parser::error::ErrorKind,
    }
};

pub type ParseError = Error<ErrorKind>;

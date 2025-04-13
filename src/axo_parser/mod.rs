#![allow(unused_imports)]
mod parser;
mod statement;
mod expression;
mod error;
mod primary;
mod composite;
mod state;
mod item;
mod fmt;
mod delimiter;

pub use  {
    statement::ControlFlow,
    expression::{Expr, ExprKind},
    item::ItemKind,
    parser::Parser,
    composite::Composite,
    primary::Primary,
    state::*,
};

use crate::{
    axo_errors::Error,
    axo_parser::error::ErrorKind,
};

pub type ParseError = Error<ErrorKind>;

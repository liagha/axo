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

pub type Error = AxoError<ErrorKind>;

pub use crate::axo_errors::Error as AxoError;
pub use statement::ControlFlow;
pub use expression::{Expr, ExprKind};
pub use item::ItemKind;
pub use parser::Parser;
pub use composite::Composite;
pub use primary::Primary;
pub use state::*;
use crate::axo_parser::error::ErrorKind;

#![allow(unused_imports)]

mod syntactic;
mod error;
pub use syntactic::*;
pub use error::*;
use crate::axo_lexer::AxoError;

pub type _Error = AxoError<ErrorKind>;

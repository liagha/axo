mod syntactic;
mod resolver;

pub use syntactic::Validator;

pub use crate::axo_errors::Error as AxoError;

pub type SyntacticError = AxoError<syntactic::ErrorKind>;
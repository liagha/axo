mod validator;
mod error;

use {
    crate::{
        axo_errors::Error,
        axo_validator::error::ErrorKind,
    }
};

pub type ValidateError = Error<ErrorKind>;
mod validator;
mod error;

pub(super) use {
    error::*
};

use {
    crate::{
        reporter::Error,
    }
};

pub type ScanError<'error> = Error<'error, crate::scanner::ErrorKind>;

pub type ValidateError<'error> = Error<'error, ErrorKind>;
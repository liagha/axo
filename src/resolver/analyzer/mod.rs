mod analysis;
mod analyzer;
mod error;
mod element;

pub use {
    analyzer::*,
    analysis::*,
};

pub(crate) use {
    error::*,
};

use {
    crate::{
        reporter::Error,
    },
};

pub type AnalyzeError<'error> = Error<'error, ErrorKind<'error>>;
mod analysis;
mod analyzer;
mod error;

pub use {
    analyzer::*,
    analysis::*,
};

pub(super) use {
    error::*,
};

use {
    crate::{
        reporter::Error,
    },
};

pub type AnalyzeError<'error> = Error<'error, ErrorKind<'error>>;
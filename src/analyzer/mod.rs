mod analysis;
mod analyzer;
mod element;
mod error;

pub use {analysis::*, analyzer::*};

pub(crate) use {analyzer::symbol, element::Analyzer, error::*};

use crate::reporter::Error;

pub type AnalyzeError<'error> = Error<'error, ErrorKind<'error>>;

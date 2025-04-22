pub mod matcher;
pub mod types;
mod common;
mod utils;
mod string;
mod numeric;

pub use matcher::Matcher;
pub use common::SimilarityMetric;
pub use types::*;
pub use string::*;
pub use utils::*;
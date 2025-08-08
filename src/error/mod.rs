mod error;
mod hint;

pub use {
    core::{
        error::Error as Failure,
    },
    error::Error,
    hint::{Hint},
};

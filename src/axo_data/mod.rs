#![allow(unused_imports)]
pub mod float;
pub mod format;
mod string;

pub use {
    string::Str,
    core::{
        slice::from_ref,
    },
};

use super::operations::*;
pub trait Number: Copy + Default + Into<u8> + From<u8> + Add<Output=Self> + Sub<Output=Self> + Mul<Output=Self> + Div<Output=Self> {}

impl<T> Number for T where T: Copy + Default + Into<u8> + From<u8> + Add<Output=Self> + Sub<Output=Self> + Mul<Output=Self> + Div<Output=Self> {}
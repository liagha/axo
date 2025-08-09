pub mod compiler;
pub mod logger;
pub mod timer;

pub mod environment {
    pub use std::env::args;
}

pub mod hash {
    pub use {
        core::hash::{Hash, Hasher},
        std::hash::DefaultHasher,
        hashish::HashSet,
    };
}

pub mod operation {
    pub use core::{
        cmp::Ordering,
        ops::{Add, Deref, DerefMut, Div, Index, Mul, Neg, Range, Rem, Sub},
    };
}

pub use {
    core::arch::asm,
    std::{
        fs::read_to_string,
        io::{stdout, Write},
    },
};

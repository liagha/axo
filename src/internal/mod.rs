pub mod compiler;
pub mod logger;
pub mod timer;

pub use {
    std::{
        env::args,
        io::{
            stdout, Write
        },
        fs::{
            read_to_string,
        },
    },
    core::{
        cmp::{
            Ordering, PartialEq
        },
        ops::{
            Add, Deref, DerefMut, Div, Mul, Neg, Range, Rem, Sub,
        },
        hash::{
            Hash, Hasher,
        },
        arch::asm,
    },
    std::hash::DefaultHasher,
    hashish::HashSet,
};

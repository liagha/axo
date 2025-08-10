pub mod compiler;
pub mod logger;
pub mod timer;

pub mod environment {
    pub use std::env::{args, current_dir, current_exe};
}

pub mod hash {
    pub use {
        core::hash::{Hash, Hasher},
        std::hash::DefaultHasher,
        hashish::HashSet,
    };
}

pub mod operation {
    pub use {
        core::{
            arch::asm,
            cmp::Ordering,
            ops::{Add, Deref, DerefMut, Div, Index, Mul, Neg, Range, Rem, Sub},
        },
    };
}

pub mod platform {
    pub use {
        std::{
            fs::{
                read_dir,
                read_to_string,
            },
            io::{stdout, Write, Error},
            ffi::{OsStr, OsString},
            path::{Path, PathBuf},
        },
    };
}
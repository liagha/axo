pub mod compiler;
pub mod driver;
pub mod logger;
pub mod timer;

pub mod hash {
    pub use {
        core::hash::{Hash, Hasher},
        hashish::HashMap as Map,
        hashish::HashSet as Set,
        std::hash::DefaultHasher,
    };
}

pub mod operation {
    pub use core::{
        arch::asm,
        cmp::Ordering,
        ops::{
            Add, BitAnd, BitOr, BitXor, Deref, DerefMut, Div, Index, Mul, Neg, Range, Rem, Shl,
            Shr, Sub,
        },
    };
}

pub mod platform {
    pub use std::{
        alloc::{alloc, dealloc, realloc, Layout},
        env::{
            args, current_dir, current_exe, var},
        ffi::{OsStr, OsString},
        fs::{canonicalize, create_dir_all, read_dir, read_to_string, File},
        io::{
            stderr, stdin, stdout, BufRead, Error, ErrorKind, Result, Stdin, StdinLock, Stdout,
            StdoutLock, Write,
        },
        path::{Path, PathBuf},
        process::Command,
        ptr::{null, NonNull},
        sync::OnceLock,
    };
}

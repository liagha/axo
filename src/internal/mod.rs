mod session;

pub use {
    session::{
        InputKind,
        PrepareAction,
        Session,
        Record,
    }
};

pub mod cache;
pub mod time;

pub mod hash {
    pub use {
        core::hash::{Hash, Hasher},
        std::collections::HashMap as Map,
        std::collections::HashSet as Set,
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
        env::{args, current_dir, current_exe, var},
        ffi::{OsStr, OsString},
        fs::{canonicalize, create_dir_all, read, read_dir, read_to_string, write, File},
        io::{
            stderr, stdin, stdout, BufRead, Error, ErrorKind, Result, Stdin, StdinLock, Stdout,
            StdoutLock, Write,
        },
        path::{Path, PathBuf},
        process::{
            Command, Stdio,
        },
        ptr::{null, NonNull},
        thread::{sleep, scope},
        sync::{RwLock as Lock},
    };
}

use crate::initializer::InitializeError;
use crate::scanner::ScanError;
use crate::parser::ParseError;
use crate::resolver::ResolveError;
use crate::analyzer::AnalyzeError;
use crate::interpreter::InterpretError;
#[cfg(feature = "generator")]
use crate::generator::GenerateError;
use crate::tracker::TrackError;

pub enum CompileError<'error> {
    Initialize(InitializeError<'error>),
    Scan(ScanError<'error>),
    Parse(ParseError<'error>),
    Resolve(ResolveError<'error>),
    Analyze(AnalyzeError<'error>),
    Interpret(InterpretError<'error>),
    #[cfg(feature = "generator")]
    Generate(GenerateError<'error>),
    Track(TrackError<'error>),
}

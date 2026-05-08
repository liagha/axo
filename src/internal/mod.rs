pub mod session;

pub use session::{Artifact, Record, RecordKind, Session};

pub mod time {
    pub use {
        core::time::Duration,
        std::time::{Instant, SystemTime, UNIX_EPOCH},
    };
}

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
        env::{
            args,
            consts::{ARCH, DLL_EXTENSION, OS},
            current_dir, current_exe, set_current_dir, temp_dir, var,
        },
        ffi::{OsStr, OsString},
        fs::{canonicalize, create_dir_all, metadata, read, read_dir, read_to_string, write, File},
        io::{
            stderr, stdin, stdout, BufRead, Error, ErrorKind, IsTerminal, Read, Result, Stdin,
            StdinLock, Stdout, StdoutLock, Write,
        },
        panic::{catch_unwind, AssertUnwindSafe},
        path::{Path, PathBuf},
        process::{Command, Stdio},
        ptr::{null, NonNull},
        sync::RwLock as Lock,
        thread::{scope, sleep},
    };
}

pub mod foreign {
    pub use std::ffi::{c_char as CChar, c_void as CVoid, CStr};
}

use crate::analyzer::AnalyzeError;
#[cfg(any(feature = "llvm", feature = "interpreter"))]
use crate::emitter::GenerateError;
use crate::initializer::InitializeError;
use crate::parser::ParseError;
use crate::resolver::ResolveError;
use crate::scanner::ScanError;
use crate::tracker::TrackError;

pub enum SessionError<'error> {
    Initialize(InitializeError<'error>),
    Scan(ScanError<'error>),
    Parse(ParseError<'error>),
    Resolve(ResolveError<'error>),
    Analyze(AnalyzeError<'error>),
    #[cfg(any(feature = "llvm", feature = "interpreter"))]
    Generate(GenerateError<'error>),
    Track(TrackError<'error>),
}

pub mod io;
pub mod memory;
pub mod text;

pub use {
    io::{
        eprint, eprint_raw, eprintln, print, print_raw, println, read_line, stdin, stdout,
        write_stderr, write_stdout, IOError, Stdin, Stdout,
    },
    memory::{next_capacity, AllocationError, Result as AllocationResult},
    text::{StringAbi, Utf8Str, Utf8String},
};

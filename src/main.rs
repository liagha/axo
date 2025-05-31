#![allow(dead_code)]

mod axo_data;
mod axo_error;
mod axo_format;
mod axo_form;
mod axo_lexer;
mod axo_parser;
mod axo_resolver;
mod axo_rune;
mod axo_span;
mod timer;
mod compiler;

pub use {
    axo_lexer::{Lexer, PunctuationKind, Token, TokenKind},
    axo_parser::Parser,
    axo_resolver::Resolver,
    axo_rune::*,
    axo_format::*,
    axo_data::{*, peekable::*},
    broccli::{xprintln, Color, TextStyle},
    timer::{Timer, TimeSource},
    compiler::{Compiler, Config, CompilerError},
};

#[cfg(target_arch = "x86_64")]
pub const TIMERSOURCE: timer::CPUCycleSource = timer::CPUCycleSource;

#[cfg(target_arch = "aarch64")]
pub const TIMERSOURCE: timer::ARMGenericTimerSource = timer::ARMGenericTimerSource;

pub type Path = std::path::PathBuf;

pub mod file {
    pub use std::fs::{read_to_string};
    pub use std::io::{Error};
}

/*pub mod process {
    pub use std::process::exit;
}*/

pub mod environment {
    pub use std::env::{args, current_dir, };
}

pub mod thread {
    pub use std::sync::{Arc};
}

pub mod memory {
    pub use core::mem::{replace, swap, discriminant};
}

pub mod compare {
    pub use core::cmp::{PartialEq, Ordering, max, min};
}

pub mod hash {
    pub use core::hash::{Hash, Hasher};
    pub use hashish::*;
}

pub mod char {
    pub use core::char::{from_u32};
}

pub mod any {
    pub use core::any::{Any, TypeId};
}

pub mod operations {
    pub use core::ops::{Add, Sub, Mul, Div, Neg, Rem, Range};
}

pub mod arch {
    pub use core::arch::asm;
}

pub mod marker {
    pub use core::marker::{PhantomData};
}

pub mod string {
    pub use core::str::FromStr;
}

pub mod slice {
    pub use core::slice::*;
}

pub mod format {
    pub use core::fmt::{Display, Debug, Formatter, Result, Write};
}


#[derive(Debug)]
pub enum AppError {
    Compiler(CompilerError),
    ArgumentParsing(String),
    HelpRequested,
}

impl format::Display for AppError {
    fn fmt(&self, f: &mut format::Formatter<'_>) -> format::Result {
        match self {
            AppError::Compiler(e) => write!(f, "{}", e),
            AppError::ArgumentParsing(msg) => write!(f, "{}", msg),
            AppError::HelpRequested => Ok(()), // Help is handled separately
        }
    }
}

impl From<CompilerError> for AppError {
    fn from(error: CompilerError) -> Self {
        AppError::Compiler(error)
    }
}

fn main() {
    println!();

    let main_timer = Timer::new(TIMERSOURCE);

    match run_application(main_timer) {
        Ok(()) => {}
        Err(AppError::HelpRequested) => {}
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}

fn run_application(main_timer: Timer<impl TimeSource>) -> Result<(), AppError> {
    let config = parse_arguments()?;

    if config.time_report {
        println!(
            "Argument Parsing Took {} ns",
            main_timer.to_nanoseconds(main_timer.elapsed().unwrap())
        );
    }

    let file_read_timer = Timer::new(TIMERSOURCE);

    let mut compiler = Compiler::new(config.clone())?;

    if config.time_report {
        println!(
            "File Read Took {} ns",
            file_read_timer.to_nanoseconds(file_read_timer.elapsed().unwrap())
        );
    }

    compiler.compile()?;

    if config.time_report {
        println!(
            "Total Compilation Took {} ns",
            main_timer.to_nanoseconds(main_timer.elapsed().unwrap())
        );
    }

    Ok(())
}

fn parse_arguments() -> Result<Config, AppError> {
    let args: Vec<String> = environment::args().collect();
    let mut config = Config {
        file_path: String::new(),
        verbose: false,
        show_tokens: false,
        show_ast: false,
        time_report: false,
    };

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-v" | "--verbose" => config.verbose = true,
            "-t" | "--tokens" => config.show_tokens = true,
            "-a" | "--ast" => config.show_ast = true,
            "--time" => config.time_report = true,
            "-h" | "--help" => {
                print_usage(&args[0]);
                return Err(AppError::HelpRequested);
            }
            flag => {
                if flag.starts_with('-') {
                    let error_msg = format!("Unknown option: {}", flag);
                    eprintln!("{}", error_msg);
                    print_usage(&args[0]);
                    return Err(AppError::ArgumentParsing(error_msg));
                }
                config.file_path = flag.to_string();
            }
        }

        i += 1;
    }

    if config.file_path.is_empty() {
        return Err(AppError::Compiler(CompilerError::PathRequired));
    }

    Ok(config)
}
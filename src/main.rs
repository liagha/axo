#![allow(dead_code)]

mod axo_data;
mod axo_error;
mod axo_form;
mod axo_format;
mod axo_scanner;
mod axo_parser;
mod axo_resolver;
mod axo_text;
mod axo_cursor;
mod compiler;
mod logger;
mod timer;
mod artifact;

use std::time::Duration;
pub use {
    axo_data::*,
    axo_format::*,
    axo_text::*,
    compiler::{Compiler, CompilerError},
    timer::{TimeSource, Timer},
};

use {
    crate::logger::{LogInfo, LogPlan, Logger},
    log::Level,
};

#[cfg(target_arch = "x86_64")]
pub const TIMERSOURCE: timer::CPUCycleSource = timer::CPUCycleSource;

#[cfg(target_arch = "aarch64")]
pub const TIMERSOURCE: timer::ARMGenericTimerSource = timer::ARMGenericTimerSource;

pub mod file {
    pub use std::fs::read_to_string;
    pub use std::io::Error;
}

pub mod io {
    pub use std::io::{stdout, Write};
}

pub mod environment {
    pub use std::env::args;
}

pub mod thread {
    pub use std::sync::{Arc, Mutex};
}

pub mod memory {
    pub use core::mem::{discriminant, drop, replace, swap};
}

pub mod compare {
    pub use core::cmp::{max, min, Ordering, PartialEq};
}

pub mod hash {
    pub use core::hash::{Hash, Hasher};
    pub use hashish::*;
}

pub mod char {
    pub use core::char::from_u32;
}

pub mod any {
    pub use core::any::{Any, TypeId};
}

pub mod operations {
    pub use core::ops::{Add, Div, Mul, Neg, Range, Rem, Sub};
}

pub mod arch {
    pub use core::arch::asm;
}

pub mod marker {
    pub use core::marker::PhantomData;
}

pub mod string {
    pub use core::str::FromStr;
}

pub mod slice {
    pub use core::slice::*;
}

pub mod format {
    pub use core::fmt::{Debug, Display, Formatter, Result, Write};
}

fn main() {
    let plan = LogPlan::new(vec![LogInfo::Time, LogInfo::Level, LogInfo::Message]) .with_separator(" ".to_string());

    let logger = Logger::new(Level::max(), plan);
    logger.init().expect("fuck");

    println!();

    let main_timer = Timer::new(TIMERSOURCE);

    match run_application(main_timer) {
        Ok(()) => {}
        Err(CompilerError::HelpRequested) => {}
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}

fn run_application(main_timer: Timer<impl TimeSource>) -> Result<(), CompilerError> {
    let (path, verbose) = parse_arguments()?;

    if verbose {
        println!(
            "Argument Parsing Took {} ns",
            main_timer.to_nanoseconds(main_timer.elapsed().unwrap())
        );
    }

    let file_read_timer = Timer::new(TIMERSOURCE);

    let mut compiler = Compiler::new(path, verbose)?;

    if verbose {
        println!(
            "File Read Took {} ns",
            file_read_timer.to_nanoseconds(file_read_timer.elapsed().unwrap())
        );
    }

    compiler.compile()?;

    if verbose {
        let duration = Duration::from_nanos(main_timer.elapsed().unwrap());
        
        println!(
            "Total Compilation Took {} ns",
            duration.as_secs_f64()
        );
    }

    Ok(())
}

fn parse_arguments() -> Result<(&'static str, bool), CompilerError> {
    let args: Vec<String> = environment::args().collect();

    let mut path = String::new();
    let mut verbose = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-v" | "--verbose" => verbose = true,
            "-h" | "--help" => {
                print_usage(&args[0]);
                return Err(CompilerError::HelpRequested);
            }
            flag => {
                if flag.starts_with('-') {
                    let error_msg = format!("Unknown option: {}", flag);
                    eprintln!("{}", error_msg);
                    print_usage(&args[0]);
                    return Err(CompilerError::ArgumentParsing(error_msg));
                }
                path = flag.to_string();
            }
        }

        i += 1;
    }

    if path.is_empty() {
        return Err(CompilerError::PathRequired);
    }
    
    Ok((path.leak(), verbose))
}
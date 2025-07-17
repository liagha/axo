#![allow(dead_code)]
extern crate core;

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
mod axo_checker;
mod axo_schema;

use core::time::Duration;
use broccli::{xprintln, Color};
pub use {
    axo_data::*,
    axo_format::*,
    axo_text::*,
    compiler::{Compiler, CompilerError},
    timer::{TimeSource, Timer},
};

use {
    crate::{
        logger::{LogInfo, LogPlan, Logger},
    },
    log::Level,
};

#[cfg(target_arch = "x86_64")]
pub const TIMERSOURCE: timer::CPUCycleSource = timer::CPUCycleSource;

#[cfg(target_arch = "aarch64")]
pub const TIMERSOURCE: timer::ARMGenericTimerSource = timer::ARMGenericTimerSource;

pub mod data {
    //pub use std::collections::VecDeque;
}

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
    pub use hashish::{HashMap, HashSet};
    pub use std::collections::hash_map::DefaultHasher;
}

pub mod character {
    pub use core::char::{
        from_u32, from_u32_unchecked, MAX
    };

    use crate::axo_data::PrimInt;

    pub fn parse_radix<T: PrimInt>(input: &str, radix: T) -> Option<T> {
        if input.is_empty() {
            return None;
        }

        let radix_u8 = radix.to_u8()?;

        if radix_u8 < 2 || radix_u8 > 36 {
            return None;
        }

        let mut accumulator = T::zero();

        for &byte in input.as_bytes() {
            let value = match byte {
                b'0'..=b'9' => byte - b'0',
                b'a'..=b'z' => byte - b'a' + 10,
                b'A'..=b'Z' => byte - b'A' + 10,
                _ => return None,
            };

            if value >= radix_u8 {
                return None;
            }

            let digit = T::from(value).unwrap();

            accumulator = accumulator.checked_mul(&radix)?
                .checked_add(&digit)?;
        }

        Some(accumulator)
    }
}

pub mod any {
    pub use core::any::{Any, TypeId};
}

pub mod operations {
    pub use core::ops::{Add, Div, Mul, Neg, Range, Rem, Sub, Deref, DerefMut};
}

pub mod architecture {
    pub use core::arch::asm;
}

pub mod marker {
    pub use core::marker::PhantomData;
}

pub mod string {
    pub use core::str::FromStr;
}

pub mod slice {
    pub use core::slice::{
        from_ref,
    };
}

pub mod format {
    pub use core::fmt::{
        Debug, Display,
        Formatter, Result, 
        Write
    };
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
        let duration = Duration::from_nanos(main_timer.elapsed().unwrap());

        xprintln!(
            "Finished {} {} {}s." => Color::Blue,
            "`examining`" => Color::White,
            "in",
            duration.as_secs_f64(),
        );
    }

    let timer = Timer::new(TIMERSOURCE);

    let mut compiler = Compiler::new(path, verbose)?;

    if verbose {
        let duration = Duration::from_nanos(timer.elapsed().unwrap());

        xprintln!(
            "  Finished {} {} {}s." => Color::Blue,
            "`analyzing`" => Color::White,
            "in",
            duration.as_secs_f64(),
        );
    }

    compiler.compile()?;

    if verbose {
        let duration = Duration::from_nanos(main_timer.elapsed().unwrap());

        xprintln!(
            "Finished {} {} {}s." => Color::Blue,
            "`compiling`" => Color::White,
            "in",
            duration.as_secs_f64(),
        );
    }

    Ok(())
}

fn parse_arguments() -> Result<(&'static str, bool), CompilerError> {
    let args: Vec<String> = environment::args().collect();

    let mut path = String::new();
    let mut verbose = false;

    for arg in args.clone() {
        match arg.as_str() {
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
    }

    if path.is_empty() {
        return Err(CompilerError::PathRequired);
    }
    
    Ok((path.leak(), verbose))
}
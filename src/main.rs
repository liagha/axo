#![allow(dead_code)]

mod axo_data;
mod axo_error;
mod axo_fmt;
mod axo_form;
mod axo_lexer;
mod axo_parser;
mod axo_resolver;
mod axo_rune;
mod axo_span;
mod timer;
mod compiler;

pub use {
    axo_lexer::{Lexer, PunctuationKind, OperatorKind, Token, TokenKind},
    axo_parser::Parser,
    axo_resolver::Resolver,
    axo_rune::*,
    axo_fmt::*,
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
}

pub mod process {
    pub use std::process::exit;
}

pub mod environment {
    pub use std::env::{args, current_dir, };
}

pub mod thread {
    pub use std::sync::{Arc, Mutex};
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

pub fn format_vec<Item: format::Display>(vector: &Vec<Item>) -> String {
    vector.iter().map(|form| form.to_string()).collect::<Vec<_>>().join(", ")
}

fn main() {
    println!();

    let main_timer = Timer::new(TIMERSOURCE);

    let config = match parse_arguments() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    if config.time_report {
        println!(
            "Argument Parsing Took {} ns",
            main_timer.to_nanoseconds(main_timer.elapsed().unwrap())
        );
    }

    let file_read_timer = Timer::new(TIMERSOURCE);

    let mut compiler = match Compiler::new(config.clone()) {
        Ok(compiler) => compiler,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    if config.time_report {
        println!(
            "File Read Took {} ns",
            file_read_timer.to_nanoseconds(file_read_timer.elapsed().unwrap())
        );
    }

    if let Err(e) = compiler.compile() {
        eprintln!("{}", e);
        process::exit(1);
    }

    if config.time_report {
        println!(
            "Total Compilation Took {} ns",
            main_timer.to_nanoseconds(main_timer.elapsed().unwrap())
        );
    }
}

fn parse_arguments() -> Result<Config, CompilerError> {
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
                process::exit(0);
            }
            flag => {
                if flag.starts_with('-') {
                    eprintln!("Unknown option: {}", flag);
                    print_usage(&args[0]);
                    process::exit(1);
                }
                config.file_path = flag.to_string();
            }
        }
        i += 1;
    }

    if config.file_path.is_empty() {
        return Err(CompilerError::PathRequired);
    }

    Ok(config)
}

fn print_usage(program: &str) {
    println!("Usage: {} [OPTIONS] <file.axo>", program);
    println!("Options:");
    println!("  -v, --verbose   Enable verbose output");
    println!("  -t, --tokens    Show lexer tokens");
    println!("  -a, --ast       Show parsed AST");
    println!("  --time          Show execution time reports");
    println!("  -h, --help      Show this help message");
}

fn format_tokens(tokens: &[Token]) -> String {
    tokens
        .iter()
        .enumerate()
        .filter(|(_, token)|
            token.kind != TokenKind::Punctuation(PunctuationKind::Space)
        )
        .map(|(i, token)| {
            let token_str = match token.kind {
                TokenKind::Punctuation(PunctuationKind::Newline) => format!(
                    "↓ {:?} | {:#?} ↓\n",
                    token,
                    token.span
                )
                    .term_colorize(Color::Green)
                    .to_string(),
                TokenKind::Punctuation(_) => format!(
                    "{:?} | {:#?}",
                    token,
                    token.span
                )
                    .term_colorize(Color::Green)
                    .to_string(),
                TokenKind::Operator(_) => format!(
                    "{:?} | {:#?}",
                    token,
                    token.span
                )
                    .term_colorize(Color::Orange)
                    .to_string(),
                _ => format!("{:?} | {:#?}", token, token.span),
            };
            if i < tokens.len() - 1
                && !matches!(token.kind, TokenKind::Punctuation(PunctuationKind::Newline))
            {
                format!("{}, ", token_str)
            } else {
                token_str
            }
        })
        .collect()
}
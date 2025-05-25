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
};

pub enum ErrorKind {
    PathRequired,
}

pub type CompileError = axo_error::Error<ErrorKind>;

#[cfg(target_arch = "x86_64")]
pub const TIMERSOURCE: timer::CPUCycleSource = timer::CPUCycleSource;

#[cfg(target_arch = "aarch64")]
pub const TIMERSOURCE: timer::ARMGenericTimerSource = timer::ARMGenericTimerSource;

pub type Path = std::path::PathBuf;

pub mod file {
    pub use std::fs::*;
}

pub mod process {
    pub use std::process::exit;
}

pub mod environment {
    pub use std::env::{args, current_dir, };
}

pub mod thread {
    pub use std::sync::Arc;
}

pub mod memory {
    pub use core::mem::{replace, swap, discriminant};
}

pub mod compare {
    pub use core::cmp::*;
}

pub mod hash {
    pub use core::hash::{Hash, Hasher};
    pub use hashish::*;
}

pub mod char {
    pub use core::char::*;
}

pub mod operations {
    pub use core::ops::*;
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

struct Config {
    file_path: String,
    verbose: bool,
    show_tokens: bool,
    show_ast: bool,
    time_report: bool,
}

fn main() {
    println!();

    let main_timer = Timer::new(TIMERSOURCE);
    let config = parse_args();

    if config.time_report {
        println!(
            "Argument Parsing Took {} ns",
            main_timer.to_nanoseconds(main_timer.elapsed().unwrap())
        );
    }

    let exec_path = environment::current_dir()
        .map(|mut path| {
            path.push(&config.file_path);
            path
        })
        .unwrap_or_else(|e| {
            eprintln!("Failed to get current directory: {}", e);
            process::exit(1);
        });

    process_file(&exec_path, &config);

    if config.time_report {
        println!(
            "Total Compilation Took {} ns",
            main_timer.to_nanoseconds(main_timer.elapsed().unwrap())
        );
    }
}

fn parse_args() -> Config {
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
        eprintln!("No input file specified");
        print_usage(&args[0]);
        process::exit(1);
    }

    config
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

fn process_file(file_path: &Path, config: &Config) {
    xprintln!(
        "{} {}" => Color::Blue,
        "Compiling" => Color::Blue,
        file_path.display()
    );
    xprintln!();

    let file_read_timer = Timer::new(TIMERSOURCE);
    let content = file::read_to_string(file_path).unwrap_or_else(|e| {
        eprintln!("Failed to read file {}: {}", file_path.display(), e);
        process::exit(1);
    });

    if config.verbose {
        xprintln!(
            "File Contents:\n{}" => Color::Magenta,
            indent(&content) => Color::BrightMagenta
        );
        xprintln!();
    }

    if config.time_report {
        println!(
            "File Read Took {} ns",
            file_read_timer.to_nanoseconds(file_read_timer.elapsed().unwrap())
        );
    }

    process_lexing(&content, file_path, config);
}

fn process_lexing(content: &str, file_path: &Path, config: &Config) {
    let lex_timer = Timer::new(TIMERSOURCE);
    
    let mut lexer = Lexer::new(content.to_string(), file_path.clone());
    
    let (tokens, errors) = lexer.lex();

    xprintln!("Tokens:\n{}", indent(&format_tokens(&*tokens)));

    xprintln!();

    for err in errors {
        let (msg, details) = err.format();
        xprintln!(
                "{}\n{}" => Color::Red,
                msg => Color::Orange,
                details
            );
    }

    xprintln!();

    /*
    let tokens = lexer.tokenize().unwrap_or_else(|err| {
        let (msg, details) = err.format();
        xprintln!(
            "{}\n{}" => Color::Red,
            msg => Color::Orange,
            details
        );
        if config.time_report {
            println!(
                "Failed Compilation Took {} ns",
                lex_timer.to_nanoseconds(lex_timer.elapsed().unwrap())
            );
        }
        process::exit(1);
    });

    if config.show_tokens || config.verbose {
        xprintln!("Tokens:\n{}", indent(&format_tokens(&tokens)));
        xprintln!();
    }
    */
    
    if config.time_report {
        println!(
            "Lexing Took {} ns",
            lex_timer.to_nanoseconds(lex_timer.elapsed().unwrap())
        );
    }

    process_parsing(tokens, file_path, config);
}

fn process_parsing(tokens: Vec<Token>, file_path: &Path, config: &Config) {
    let parse_timer = Timer::new(TIMERSOURCE);
    let mut parser = Parser::new(tokens, file_path.clone());
    
    let (test_elements, test_errors) = parser.parse_program();

    let test_ast = test_elements
        .iter()
        .map(|element| format!("{:?}", element))
        .collect::<Vec<String>>()
        .join("\n");
    
    xprintln!("Test Elements:\n{}" => Color::Green, indent(&test_ast));
    xprintln!();

    for err in test_errors {
        let (msg, details) = err.format();
        xprintln!(
                "{}\n{}" => Color::Red,
                msg => Color::Orange,
                details
            );
    }

    parser.restore();

    let elements = parser.parse();

    if !parser.errors.is_empty() {
        for err in parser.errors {
            xprintln!("{}" => Color::Red, err);
        }
        if config.time_report {
            println!(
                "Error Handling Took {} ns",
                parse_timer.to_nanoseconds(parse_timer.elapsed().unwrap())
            );
        }
        process::exit(1);
    }

    if config.show_ast || config.verbose {
        let ast = elements
            .iter()
            .map(|element| format!("{:?}", element))
            .collect::<Vec<String>>()
            .join("\n");
        xprintln!("Elements:\n{}" => Color::Green, indent(&ast));
        xprintln!();
    }

    if config.time_report {
        println!(
            "Parsing Took {} ns",
            parse_timer.to_nanoseconds(parse_timer.elapsed().unwrap())
        );
    }

    process_resolution(elements, config);
}

fn process_resolution(elements: Vec<axo_parser::Element>, config: &Config) {
    let resolver_timer = Timer::new(TIMERSOURCE);
    let mut resolver = Resolver::new();
    resolver.resolve(elements);

    if !resolver.errors.is_empty() {
        for err in resolver.errors {
            let (msg, details) = err.format();
            xprintln!(
                "{}\n{}" => Color::Red,
                msg => Color::Orange,
                details
            );
        }
        process::exit(1);
    }

    if config.verbose && !resolver.scope.all_symbols().is_empty() {
        xprintln!(
            "{}" => Color::Cyan,
            format!("Symbols:\n{:#?}", resolver.scope.all_symbols())
        );
    }

    if config.time_report {
        println!(
            "Resolution Took {} ns",
            resolver_timer.to_nanoseconds(resolver_timer.elapsed().unwrap())
        );
    }
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

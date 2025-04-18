pub mod axo_lexer;
mod axo_parser;
mod axo_semantic;
mod axo_data;
mod axo_rune;
mod axo_errors;
mod axo_codegen;
mod axo_span;

pub use axo_rune::*;

use std::time::Instant;
use std::path::PathBuf;
use axo_lexer::{Lexer, PunctuationKind, Token, TokenKind};
use axo_parser::Parser;
use broccli::{xprintln, Color, TextStyle};
use crate::axo_semantic::Resolver;

struct Config {
    file_path: String,
    verbose: bool,
    show_tokens: bool,
    show_ast: bool,
    time_report: bool,
}

fn main() {
    let config = parse_args();

    let exec_path = match std::env::current_dir() {
        Ok(mut path) => {
            path.push(&config.file_path);
            path
        }
        Err(e) => {
            eprintln!("Failed to get current directory: {}", e);
            std::process::exit(1);
        }
    };

    if config.verbose {
        xprintln!("Path: {}", exec_path.display());
    }

    let start = Instant::now();
    process_file(&exec_path, &config);

    if config.time_report {
        let total_time = start.elapsed().as_millis();
        xprintln!("Total execution time: {}ms" => Color::Green, total_time);
    }
}

fn parse_args() -> Config {
    let args: Vec<String> = std::env::args().collect();
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
            "verbose" | "-v" => config.verbose = true,
            "tokens" | "-t" => config.show_tokens = true,
            "ast" | "-a" => config.show_ast = true,
            "--time" => config.time_report = true,
            "--help" | "-h" => {
                print_usage(&args[0]);
                std::process::exit(0);
            }
            _ => {
                if args[i].starts_with("-") {
                    eprintln!("Unknown option: {}", args[i]);
                    print_usage(&args[0]);
                    std::process::exit(1);
                } else {
                    config.file_path = args[i].clone();
                }
            }
        }
        i += 1;
    }

    if config.file_path.is_empty() {
        eprintln!("No input file specified");
        print_usage(&args[0]);
        std::process::exit(1);
    }

    config
}

fn print_usage(program: &str) {
    println!("Usage: {} [OPTIONS] <file.axo>", program);
    println!("Options:");
    println!("  -v, --verbose     Enable verbose output");
    println!("  -t, --tokens      Show lexer tokens");
    println!("  -a, --ast         Show parsed AST");
    println!("  -b, --build       Build only (parse AST without further processing)");
    println!("  --time            Show execution time reports");
    println!("  -h, --help        Show this help message");
}

fn process_file(file_path: &PathBuf, config: &Config) {
    println!("{} {}\n", "\tCompiling".term_colorize(Color::Blue), file_path.display());

    let start = Instant::now();

    match std::fs::read_to_string(file_path) {
        Ok(content) => {
            if config.verbose {
                let read_time = start.elapsed();
                println!("File read took: {:?}", read_time);

                xprintln!(
                    "File Contents: \n{}" => Color::Blue,
                    content.clone() => Color::BrightBlue
                );
                xprintln!();
            }

            lex_and_parse(content, &config.file_path, config);
        }
        Err(e) => {
            eprintln!("Failed to read file: {}", e);
            std::process::exit(1);
        }
    }
}

fn lex_and_parse(content: String, file_path: &str, config: &Config) {
    let lex_start = Instant::now();
    let mut lexer = Lexer::new(content, PathBuf::from(file_path));

    match lexer.tokenize() {
        Ok(tokens) => {
            let lex_time = lex_start.elapsed().as_secs_f32();

            if config.show_tokens || config.verbose {
                xprintln!("Tokens: \n{}", format_tokens(&tokens));
                xprintln!();
            } else if config.time_report {
                xprintln!("Lexing completed in {} seconds", lex_time);
            }

            println!("Compilation took: {} seconds", lex_start.elapsed().as_secs_f32());

            parse_tokens(tokens, file_path, config);
        }
        Err(err) => {
            let parse_time = lex_start.elapsed().as_secs_f32();
            let (msg, details) = err.format();

            xprintln!("{} \n {} => took {} seconds" => Color::Red,
                            msg => Color::Orange, details, parse_time
                        );

            println!("Compilation took: {} seconds", lex_start.elapsed().as_secs_f32());

            std::process::exit(1);
        }
    }

}

fn parse_tokens(tokens: Vec<Token>, file_path: &str, config: &Config) {
    let parse_start = Instant::now();
    let mut parser = Parser::new(tokens.clone(), PathBuf::from(file_path));

    let expressions = parser.parse_program();

    if parser.errors.is_empty() {
        let parse_time = parse_start.elapsed().as_secs_f32();

        if config.show_ast || config.verbose {
            xprintln!(
                    format!("{:#?}", expressions).term_colorize(Color::Green),
                );

            println!();

            let mut resolver = Resolver::new();

            resolver.resolve(expressions);

            if !resolver.errors.is_empty() {
                for err in resolver.errors {
                    let (msg, details) = err.format();

                    xprintln!("{} \n {}" => Color::Red,
                            msg => Color::Orange, details
                        );
                }
            } else {
                println!("{:#?}", resolver.symbols())
            }

            // let exprs: String = expressions.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("\n");
            // xprintln!("Expressions: {}", format!("{}", exprs).term_colorize(Color::Green));
        } else if config.time_report {
            xprintln!("Parsing completed in {} seconds", parse_time);
        }
    }
    else {
        for err in parser.errors {
            println!("{}", err);
        }
    }
}

fn format_tokens(input: &Vec<Token>) -> String {
    let mut result = String::new();

    for (i, token) in input.iter().enumerate() {
        let token_str = match token.clone().kind {
            TokenKind::Punctuation(PunctuationKind::Newline) => {
                format!("↓{:?} | {}↓", token, token.span).term_colorize(Color::Green).to_string() + "\n"
            }
            TokenKind::Punctuation(_) => {
                format!("{:?} | {}", token, token.span).term_colorize(Color::Green).to_string()
            }
            TokenKind::Operator(_) => {
                format!("{:?} | {}", token, token.span).term_colorize(Color::Orange).to_string()
            }
            TokenKind::Keyword(_) => {
                format!("{:?} | {}", token, token.span).term_colorize(Color::Blue).to_string()
            }
            _ => {
                format!("{:?} | {}", token, token.span).to_string()
            }
        };

        result.push_str(&token_str);

        if i != input.len() - 1 && !matches!(token, Token { kind: TokenKind::Punctuation(PunctuationKind::Newline), .. }) {
            result.push_str(", ");
        }
    }

    result
}
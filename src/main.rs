mod axo_lexer;
mod axo_parser;
mod axo_resolver;
mod axo_data;
mod axo_rune;
mod axo_errors;
mod axo_span;
mod timer;

pub use {
    axo_lexer::{Lexer, PunctuationKind, Token, TokenKind},
    axo_parser::Parser,
    axo_rune::*,
    broccli::{xprintln, Color, TextStyle},
    std::path::PathBuf,
    timer::{Timer, TimeSource, CPUCycleSource},
};
use crate::axo_resolver::Resolver;

struct Config {
    file_path: String,
    verbose: bool,
    show_tokens: bool,
    show_ast: bool,
    time_report: bool,
}

fn main() {
    let main_timer = Timer::new(CPUCycleSource);

    let config = parse_args();
    if config.time_report {
        println!(
            "Argument Parsing Took {} ns",
            main_timer.to_nanoseconds(main_timer.elapsed().unwrap())
        );
    }

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

    process_file(&exec_path, &config);

    if config.time_report {
        println!(
            "Total Compilation Took {} ns",
            main_timer.to_nanoseconds(main_timer.elapsed().unwrap())
        );
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
            "-v" | "--verbose" => config.verbose = true,
            "-t" | "--tokens" => config.show_tokens = true,
            "-a" | "--ast" => config.show_ast = true,
            "--time" => config.time_report = true,
            "-h" | "--help" => {
                print_usage(&args[0]);
                std::process::exit(0);
            }
            _ => {
                if args[i].starts_with('-') {
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
    println!("  -v, --verbose   Enable verbose output");
    println!("  -t, --tokens    Show lexer tokens");
    println!("  -a, --ast       Show parsed AST");
    println!("  --time          Show execution time reports");
    println!("  -h, --help      Show this help message");
}

fn process_file(file_path: &PathBuf, config: &Config) {
    xprintln!(
        "{} {}" => Color::Blue,
        "Compiling" => Color::Blue,
        file_path.display()
    );

    let file_read_timer = Timer::new(CPUCycleSource);
    match std::fs::read_to_string(file_path) {
        Ok(content) => {
            if config.verbose {
                xprintln!(
                    "File Contents:\n{}" => Color::Blue,
                    content.clone() => Color::BrightBlue
                );
            }
            if config.time_report {
                println!(
                    "File Read Took {} ns",
                    file_read_timer.to_nanoseconds(file_read_timer.elapsed().unwrap())
                );
            }
            lex_and_parse(content, file_path.to_str().unwrap_or("unknown"), config);
        }
        Err(e) => {
            eprintln!("Failed to read file {}: {}", file_path.display(), e);
            std::process::exit(1);
        }
    }
}

fn lex_and_parse(content: String, file_path: &str, config: &Config) {
    let lex_timer = Timer::new(CPUCycleSource);
    let mut lexer = Lexer::new(content, PathBuf::from(file_path));

    match lexer.tokenize() {
        Ok(tokens) => {
            if config.show_tokens || config.verbose {
                xprintln!("Tokens:\n{}", format_tokens(&tokens));
            }
            if config.time_report {
                println!(
                    "Lexing Took {} ns",
                    lex_timer.to_nanoseconds(lex_timer.elapsed().unwrap())
                );
            }
            parse_tokens(tokens, file_path, config);
        }
        Err(err) => {
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
            std::process::exit(1);
        }
    }
}

fn parse_tokens(tokens: Vec<Token>, file_path: &str, config: &Config) {
    let parse_timer = Timer::new(CPUCycleSource);
    let mut parser = Parser::new(tokens, PathBuf::from(file_path));
    let expressions = parser.parse_program();

    if parser.errors.is_empty() {
        if config.show_ast || config.verbose {
            xprintln!(
                    "{}" => Color::Green,
                    format!(
                        "\n{:#?}",
                        expressions
                    )
                );

            let resolver_timer = Timer::new(CPUCycleSource);
            let mut resolver = Resolver::new();
            resolver.resolve(expressions);

            if !resolver.errors.is_empty() {
                for err in resolver.errors {
                    let (msg, details) = err.format();
                    xprintln!(
                        "{}\n{}" => Color::Red,
                        msg => Color::Orange,
                        details
                    );
                }
            } else if config.verbose {
                xprintln!(
                    "{}" => Color::Cyan,
                    format!(
                        "Symbols:\n{:#?}",
                        resolver.scope.all_symbols()
                    )
                );
            }

            if config.time_report {
                println!(
                    "Resolution Took {} ns",
                    resolver_timer.to_nanoseconds(resolver_timer.elapsed().unwrap())
                );
            }
        }
        if config.time_report {
            println!(
                "Parsing Took {} ns",
                parse_timer.to_nanoseconds(parse_timer.elapsed().unwrap())
            );
        }
    } else {
        for err in parser.errors {
            xprintln!(
                "{}" => Color::Red,
                err
            );
        }
        if config.time_report {
            println!(
                "Error Handling Took {} ns",
                parse_timer.to_nanoseconds(parse_timer.elapsed().unwrap())
            );
        }
        std::process::exit(1);
    }
}

fn format_tokens(tokens: &[Token]) -> String {
    tokens
        .iter()
        .enumerate()
        .map(|(i, token)| {
            let token_str = match token.kind {
                TokenKind::Punctuation(PunctuationKind::Newline) => format!(
                    "↓ {:?} | {} ↓\n",
                    token,
                    token.span
                )
                    .term_colorize(Color::Green)
                    .to_string(),
                TokenKind::Punctuation(_) => format!(
                    "{:?} | {}",
                    token,
                    token.span
                )
                    .term_colorize(Color::Green)
                    .to_string(),
                TokenKind::Operator(_) => format!(
                    "{:?} | {}",
                    token,
                    token.span
                )
                    .term_colorize(Color::Orange)
                    .to_string(),
                TokenKind::Keyword(_) => format!(
                    "{:?} | {}",
                    token,
                    token.span
                )
                    .term_colorize(Color::Blue)
                    .to_string(),
                _ => format!("{:?} | {}", token, token.span),
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
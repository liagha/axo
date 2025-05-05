mod axo_lexer;
mod axo_parser;
mod axo_resolver;
mod axo_data;
mod axo_rune;
mod axo_errors;
mod axo_span;
mod timer;
mod axo_form;
mod axo_fmt;

pub use {
    axo_lexer::{Lexer, PunctuationKind, Token, TokenKind},
    axo_parser::Parser,
    axo_rune::*,
    axo_fmt::*,
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

    let exec_path = std::env::current_dir()
        .map(|mut path| {
            path.push(&config.file_path);
            path
        })
        .unwrap_or_else(|e| {
            eprintln!("Failed to get current directory: {}", e);
            std::process::exit(1);
        });

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
            flag => {
                if flag.starts_with('-') {
                    eprintln!("Unknown option: {}", flag);
                    print_usage(&args[0]);
                    std::process::exit(1);
                }
                config.file_path = flag.to_string();
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
    xprintln!();

    let file_read_timer = Timer::new(CPUCycleSource);
    let content = std::fs::read_to_string(file_path).unwrap_or_else(|e| {
        eprintln!("Failed to read file {}: {}", file_path.display(), e);
        std::process::exit(1);
    });

    if config.verbose {
        xprintln!(
            "File Contents:\n{}" => Color::Blue,
            indent(&content) => Color::BrightBlue
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

fn process_lexing(content: &str, file_path: &PathBuf, config: &Config) {
    let lex_timer = Timer::new(CPUCycleSource);
    let mut lexer = Lexer::new(content.to_string(), file_path.clone());

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
        std::process::exit(1);
    });

    if config.show_tokens || config.verbose {
        xprintln!("Tokens:\n{}", indent(&format_tokens(&tokens)));
        xprintln!();
    }

    if config.time_report {
        println!(
            "Lexing Took {} ns",
            lex_timer.to_nanoseconds(lex_timer.elapsed().unwrap())
        );
    }

    process_parsing(tokens, file_path, config);
}

fn process_parsing(tokens: Vec<Token>, file_path: &PathBuf, config: &Config) {
    let parse_timer = Timer::new(CPUCycleSource);
    let mut parser = Parser::new(tokens, file_path.clone());
    let elements = parser.parse_program();

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
        std::process::exit(1);
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
    let resolver_timer = Timer::new(CPUCycleSource);
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
        std::process::exit(1);
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
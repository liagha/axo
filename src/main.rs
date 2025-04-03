mod cli;
mod axo_parser;
pub mod axo_lexer;
use std::time::Instant;
use std::path::PathBuf;
use axo_lexer::{Lexer, PunctuationKind, Token, TokenKind};
use axo_parser::Parser;
use broccli::{xprintln, Color, TextStyle};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <file.axo>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];

    let exec_path = match std::env::current_dir() {
        Ok(mut path) => {
            path.push(file_path);
            path
        }
        Err(e) => {
            eprintln!("Failed to get current directory: {}", e);
            std::process::exit(1);
        }
    };

    xprintln!("Path: {}", exec_path.display());

    let start = Instant::now();

    match std::fs::read_to_string(&exec_path) {
        Ok(content) => {
            let read_time = start.elapsed();
            println!("File read took: {:?}", read_time);

            xprintln!(
                "File Contents: \n{}" => Color::Blue,
                content.clone() => Color::BrightBlue
            );

            xprintln!();

            let lex_start = Instant::now();
            let mut lexer = Lexer::new(content, PathBuf::from(file_path));

            match lexer.tokenize() {
                Ok(tokens) => {
                    let lex_time = lex_start.elapsed().as_millis();
                    xprintln!("Tokens: \n{} => took {}ms", format_tokens(&tokens), lex_time);

                    xprintln!();

                    let parse_start = Instant::now();
                    let mut parser = Parser::new(tokens.clone(), PathBuf::from(file_path));
                    match parser.parse_program() {
                        Ok(stmts) => {
                            let parse_time = parse_start.elapsed().as_millis();
                            xprintln!("Parsed AST: {} => took {}ms", format!("{:#?}", stmts).term_colorize(Color::Green), parse_time);
                        },
                        Err(err) => {
                            let end_span = tokens[parser.position].span.clone();
                            let parse_time = parse_start.elapsed().as_millis();
                            let state = parser.state.pop().unwrap().describe_chain();

                            xprintln!("Parse error {}: error while parsing {}: {} => took {}ms" => Color::Red, end_span, state, err => Color::Orange, parse_time);
                        }
                    }
                }
                Err(e) => {
                    let lex_time = lex_start.elapsed().as_millis();
                    xprintln!("Lexing error: {}:{} {} => took {}ms" => Color::Red, lexer.line, lexer.column, e => Color::Orange, lex_time);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to read file: {}", e);
            std::process::exit(1);
        }
    }

    let total_time = start.elapsed().as_millis();
    xprintln!("Total execution time: {}ms" => Color::Green, total_time);
}

fn format_tokens(input: &Vec<Token>) -> String {
    let mut result = String::new();

    for (i, token) in input.iter().enumerate() {
        match token.clone().kind {
            TokenKind::Punctuation(PunctuationKind::Newline) => {
                result.push_str(format!("↓{:?} | {}↓", token, token.span).term_colorize(Color::Green).as_str());
                result.push_str("\n");
            }
            TokenKind::Punctuation(_) => {
                result.push_str(format!("{:?} | {}", token, token.span).term_colorize(Color::Green).as_str());
            }
            TokenKind::Operator(_) => {
                result.push_str(format!("{:?} | {}", token, token.span).term_colorize(Color::Orange).as_str());
            }
            TokenKind::Keyword(_) => {
                result.push_str(format!("{:?} | {}", token, token.span).term_colorize(Color::Blue).as_str());
            }
            _ => {
                result.push_str(format!("{:?} | {}", token, token.span).as_str());
            }
        }

        if i != input.len() - 1 {
            if !matches!(token, Token { kind: TokenKind::Punctuation(PunctuationKind::Newline), .. }) {
                result.push_str(", ");
            }
        }
    }

    result
}
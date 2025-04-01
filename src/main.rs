mod cli;
mod axo_ast;
pub mod axo_lexer;
use std::time::Instant;
use std::path::PathBuf;
use std::str::FromStr;
use axo_lexer::{Lexer, PunctuationKind, Token, TokenKind};
use axo_ast::Parser;
use broccli::{xprintln, Color, TextStyle};

fn main() {
    let mut exec_path = std::env::current_dir().unwrap().to_str().unwrap().to_string();
    let file_path = "/test_project/text.axo";
    exec_path.push_str(file_path);
    xprintln!("Path: {}", exec_path);

    let start = Instant::now();
    if let Ok(content) = std::fs::read_to_string(&exec_path) {
        let read_time = start.elapsed();
        println!("File read took: {:?}", read_time);

        xprintln!(
            "File Contents: \n{}" => Color::Blue,
            content.clone() => Color::BrightBlue
        );

        xprintln!();

        let lex_start = Instant::now();
        let mut lexer = Lexer::new(content, PathBuf::from_str(file_path).unwrap());

        match lexer.tokenize() {
            Ok(tokens) => {
                let lex_time = lex_start.elapsed().as_millis();
                xprintln!("Tokens: \n{} => took {}ms", format_tokens(&tokens), lex_time);

                xprintln!();

                let parse_start = Instant::now();
                let mut parser = Parser::new(tokens.clone(), PathBuf::from_str(file_path).unwrap());
                match parser.parse_program() {
                    Ok(stmts) => {
                        let parse_time = parse_start.elapsed().as_millis();
                        xprintln!("Parsed AST: {} => took {}ms", format!("{:#?}", stmts).term_colorize(Color::Green), parse_time);
                    },
                    Err(err) => {
                        let end_span = tokens[parser.position].span.clone();
                        let parse_time = parse_start.elapsed().as_millis();
                        xprintln!("Parse error ({}): {} => took {}ms" => Color::Red, end_span, err => Color::Orange, parse_time);
                    }
                }
            }
            Err(e) => {
                let lex_time = lex_start.elapsed().as_millis();
                xprintln!("Lexing error: ({}:{}) {} => took {}ms" => Color::Red, lexer.line, lexer.column, e => Color::Orange, lex_time);
            }
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

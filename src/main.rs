mod cli;
mod codegen;
mod parser;
pub mod lexer;
use lexer::{Lexer, PunctuationKind, Token, TokenKind};
use parser::Parser;

use broccli::{xprintln, Color, TextStyle};

fn main() {
    let mut exec_path = std::env::current_dir().unwrap().to_str().unwrap().to_string();
    let file_path = "/test_project/text.axo";
    exec_path.push_str(file_path);
    xprintln!("Path: {}", exec_path);
    if let Ok(content) = std::fs::read_to_string(exec_path) {
        xprintln!(
            "File Contents: \n{}" => Color::Blue,
            content.clone() => Color::BrightBlue
        );

        xprintln!();

        let lexer = Lexer::new(content);

        match lexer.tokenize() {
            Ok(tokens) => {
                xprintln!("Tokens: \n{}", format_tokens(&tokens));

                xprintln!();

                let mut parser = Parser::new(tokens);
                match parser.parse_program() {
                    Ok(stmts) => {
                        println!("Parsed AST: {}", format!("{:#?}", stmts).term_colorize(Color::Green));
                    },
                    Err(err) => xprintln!("Parse error ({}:{}): {}" => Color::Red, parser.line, parser.column, err => Color::Orange),
                }
            }
            Err(e) => xprintln!("Lexing error: {}" => Color::Red, e => Color::Orange),
        }
    }
}

fn format_tokens(input: &Vec<Token>) -> String {
    let mut result = String::new();

    for (i, token) in input.iter().enumerate() {
        match token.clone().kind {
            TokenKind::Punctuation(PunctuationKind::Newline) => {
                result.push_str(format!("↓{:?}↓", token).term_colorize(Color::Green).as_str());
                result.push_str("\n");
            }
            TokenKind::Punctuation(_) => {
                result.push_str(format!("{:?}", token).term_colorize(Color::Green).as_str());
            }
            TokenKind::Operator(_) => {
                result.push_str(format!("{:?}", token).term_colorize(Color::Orange).as_str());
            }
            TokenKind::Keyword(_) => {
                result.push_str(format!("{:?}", token).term_colorize(Color::Blue).as_str());
            }
            TokenKind::EOF => {
                result.push_str(format!("{:?}", token).term_colorize(Color::Red).as_str());
            }
            _ => {
                result.push_str(format!("{:?}", token).as_str());
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

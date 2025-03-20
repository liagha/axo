mod tokens;
mod lexer;
mod errors;
mod cli;
mod codegen;
mod parser;

use broccli::{xprintln, Color, TextStyle};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::tokens::{Punctuation, Token};

fn main() {
    let file_path = "/Users/ali/Projects/axo/test_project/text.axo";
    if let Ok(content) = std::fs::read_to_string(file_path) {
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
        match token.clone() {
            Token::Punctuation(Punctuation::Newline) => {
                result.push_str(format!("↓{:?}↓", token).term_colorize(Color::Green).as_str());
                result.push_str("\n");
            }
            Token::Punctuation(_) => {
                result.push_str(format!("{:?}", token).term_colorize(Color::Green).as_str());
            }
            Token::Operator(_) => {
                result.push_str(format!("{:?}", token).term_colorize(Color::Orange).as_str());
            }
            Token::Keyword(_) => {
                result.push_str(format!("{:?}", token).term_colorize(Color::Blue).as_str());
            }
            Token::EOF => {
                result.push_str(format!("{:?}", token).term_colorize(Color::Red).as_str());
            }
            _ => {
                result.push_str(format!("{:?}", token).as_str());
            }
        }

        if i != input.len() - 1 && token != &Token::Punctuation(Punctuation::Newline) {
            result.push_str(", ");
        }
    }

    result
}
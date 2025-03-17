mod parser;
mod tokens;
mod lexer;
mod errors;
mod cli;

use crate::lexer::Lexer;
use crate::parser::Parser;

fn main() {
    let file_path = "/Users/ali/Projects/axo/test_project/text.axo";
    if let Ok(content) = std::fs::read_to_string(file_path) {
        println!("File Contents: \n{}\n", &*content);

        let mut lexer = Lexer::new(content);

        match lexer.tokenize() {
            Ok(tokens) => {
                println!("Tokens: {:?}", tokens);

                let mut parser = Parser::new(tokens);
                match parser.parse_program() {
                    Ok(ast) => println!("Parsed AST: {:#?}", ast),
                    Err(err) => eprintln!("Parse error: {}", err),
                }
            }
            Err(e) => println!("Lexing error: {:?}", e),
        }
    }
}
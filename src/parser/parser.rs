#![allow(dead_code)]
use crate::lexer::{OperatorKind, PunctuationKind, Token, TokenKind};
use crate::parser::{Expr};
use crate::parser::error::{ParseError, SyntaxPosition, SyntaxType};
use crate::parser::expression::Expression;
use crate::parser::statement::Statement;

pub struct Parser {
    tokens: Vec<Token>,
    pub current: usize,
    pub line: usize,
    pub column: usize,
    pub debug: u8,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0, line: 1, column: 1, debug: 0 }
    }

    pub fn advance(&mut self) -> Option<Token> {
        while self.current < self.tokens.len() {
            let token = self.tokens[self.current].clone();
            self.current += 1;

            match &token.kind {
                TokenKind::Punctuation(PunctuationKind::Newline) => {
                    self.line += 1;
                    self.column = 0;
                    if self.debug >= 2 {
                        println!("DEBUG (L2): Skipping newline token. Line now: {}", self.line);
                    }
                    continue;
                }
                TokenKind::Comment(_) => {
                    self.column += 1;
                    if self.debug >= 2 {
                        println!("DEBUG (L2): Skipping comment token.");
                    }
                    continue;
                }
                _ => {
                    self.column += 1;
                    if self.debug >= 3 {
                        println!("DEBUG (L3): Advancing token: {:?} at line {}, column {}", token, self.line, self.column);
                    }
                    return Some(token);
                }
            }
        }
        if self.debug >= 2 {
            println!("DEBUG (L2): No more tokens to advance");
        }
        None
    }

    pub fn peek(&self) -> Option<&Token> {
        let mut current = self.current;
        let mut line = self.line;
        let mut column = self.column;

        while let Some(token) = self.tokens.get(current) {
            match token.kind {
                TokenKind::Punctuation(PunctuationKind::Newline) => {
                    current += 1;
                    line += 1;
                    column = 0;
                    if self.debug >= 2 {
                        println!("DEBUG (L2): Skipping newline token. Line now: {}", line);
                    }
                }
                TokenKind::Comment(_) => {
                    current += 1;
                    column += 1;

                    if self.debug >= 2 {
                        println!("DEBUG (L2): Skipping comment token.");
                    }
                }
                _ => {
                    if self.debug >= 3 {
                        println!("DEBUG (L3): Peeking token: {:?} at line {}, column {}", token, line, column);
                    }
                    return Some(token);
                }
            }
        }
        if self.debug >= 2 {
            println!("DEBUG (L2): No more tokens to peek");
        }
        None
    }

    pub fn match_token(&mut self, expected: &TokenKind) -> bool {
        // First, clone the debug level to avoid borrowing issues
        let debug_level = self.debug;

        // Use a local variable for peeking to avoid mutable borrow
        if let Some(token) = self.tokens.get(self.current) {
            if token.kind == TokenKind::Punctuation(PunctuationKind::Newline) {
                self.current += 1;
                self.line += 1;
                self.column = 0;
                if debug_level >= 2 {
                    println!("DEBUG (L2): Skipping newline token. Line now: {}", self.line);
                }
                return false;
            }

            if &token.kind == expected {
                if debug_level >= 1 {
                    println!("DEBUG (L1): Matched token: {:?}", token);
                }
                self.advance();
                return true;
            }

            if debug_level >= 1 {
                println!("DEBUG (L1): Token mismatch. Expected: {:?}, Found: {:?}", expected, token);
            }
        }

        false
    }

    pub fn parse_program(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut statements = Vec::new();
        while let Some(token) = self.peek() {
            if token.kind == TokenKind::EOF {
                break;
            }

            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }

    pub fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        if let Some(token) = self.peek().cloned() {
            match token.kind.clone() {
                TokenKind::Punctuation(PunctuationKind::LeftBracket) => self.parse_array(),
                TokenKind::Punctuation(PunctuationKind::LeftParen) => self.parse_tuple(),
                TokenKind::Operator(OperatorKind::Pipe) => self.parse_closure(),
                TokenKind::Identifier(name) => {
                    self.advance();
                    let mut expr = Expr::Identifier(name.clone());

                    while let Some(token) = self.peek() {
                        match &token.kind {
                            TokenKind::Punctuation(PunctuationKind::LeftBracket) => expr = self.parse_index(expr)?,
                            TokenKind::Punctuation(PunctuationKind::LeftParen) => expr = self.parse_call(name.clone())?,
                            TokenKind::Operator(OperatorKind::Dot) => {
                                self.advance();
                                let field = self.parse_expression()?;
                                expr = Expr::FieldAccess(Box::new(expr), Box::new(field));
                            }
                            TokenKind::Operator(OperatorKind::DoubleColon) => {
                                self.advance();
                                let field = self.parse_expression()?;
                                expr = Expr::FieldAccess(Box::new(expr), Box::new(field));
                            }
                            TokenKind::Punctuation(PunctuationKind::LeftBrace) => expr = self.parse_struct_init(expr)?,
                            _ => break,
                        }
                    }

                    Ok(expr)
                }
                TokenKind::Str(_) 
                | TokenKind::Char(_) 
                | TokenKind::Boolean(_) 
                | TokenKind::Float(_) 
                | TokenKind::Integer(_) => { self.advance(); Ok(Expr::Literal(token.clone())) }

                TokenKind::EOF => Err(ParseError::UnexpectedEOF),
                token => Err(ParseError::InvalidSyntax(format!("Unexpected token: {:?}", token))),
            }
        } else {
            Err(ParseError::UnexpectedEOF)
        }
    }
}

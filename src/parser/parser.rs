#![allow(dead_code)]
use crate::lexer::{OperatorKind, PunctuationKind, Token, TokenKind, Span};
use crate::parser::{Expr, ExprKind};
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
        let debug_level = self.debug;

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

    pub fn expect_token(
        &mut self, 
        expected: TokenKind, 
        position: SyntaxPosition, 
        syntax_type: SyntaxType
    ) -> Result<Token, ParseError> {
        if let Some(token) = self.advance() {
            if token.kind == expected {
                Ok(token)
            } else {
                Err(ParseError::ExpectedToken(expected, position, syntax_type))
            }
        } else {
            Err(ParseError::UnexpectedEOF)
        }
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
            let Token { kind, span } = token.clone();

            match kind {
                TokenKind::Punctuation(PunctuationKind::LeftBracket) => self.parse_array(),
                TokenKind::Punctuation(PunctuationKind::LeftParen) => self.parse_tuple(),
                TokenKind::Operator(OperatorKind::Pipe) => self.parse_closure(),
                TokenKind::Identifier(name) => {
                    self.advance();
                    let kind = ExprKind::Identifier(name.clone());
                    let mut expr = Expr { kind, span };

                    while let Some(token) = self.peek() {
                        match &token.kind {
                            TokenKind::Punctuation(PunctuationKind::Semicolon) => return Ok(expr),
                            TokenKind::Punctuation(PunctuationKind::LeftBrace) => expr = self.parse_struct(expr)?,
                            TokenKind::Punctuation(PunctuationKind::LeftBracket) => expr = self.parse_index(expr)?,
                            TokenKind::Punctuation(PunctuationKind::LeftParen) => { 
                                expr = self.parse_call(expr)?; 
                                return Ok(expr); 
                            },
                            TokenKind::Operator(OperatorKind::Dot) => {
                                let field = self.parse_expression()?;
                                
                                self.advance();

                                let span = Span { start: expr.span.start, end: field.span.end };

                                let kind = ExprKind::FieldAccess(expr.into(), field.into());
                                expr = Expr { kind, span };
                            }
                            _ => break,
                        }
                    }

                    Ok(expr)
                }
                TokenKind::Str(_) 
                | TokenKind::Char(_) 
                | TokenKind::Boolean(_) 
                | TokenKind::Float(_) 
                | TokenKind::Integer(_) => { 
                    self.advance();

                    let kind = ExprKind::Literal(token.clone());
                    let span = token.span;

                    let expr = Expr { kind, span };

                    Ok(expr) 
                }

                TokenKind::EOF => Err(ParseError::UnexpectedEOF),
                token => Err(ParseError::InvalidSyntax(format!("Unexpected token: {:?}", token))),
            }
        } else {
            Err(ParseError::UnexpectedEOF)
        }
    }

    pub fn parse_single(&mut self) -> Result<Expr, ParseError> {
        if let Some(token) = self.peek().cloned() {
            let Token { kind, span } = token.clone();

            match kind {
                TokenKind::Punctuation(PunctuationKind::LeftBracket) => self.parse_array(),
                TokenKind::Punctuation(PunctuationKind::LeftParen) => self.parse_tuple(),
                TokenKind::Operator(OperatorKind::Pipe) => self.parse_closure(),
                TokenKind::Identifier(name) => {
                    self.advance();
                    let kind = ExprKind::Identifier(name.clone());
                    let mut expr = Expr { kind, span };

                    Ok(expr)
                }
                TokenKind::Str(_) 
                | TokenKind::Char(_) 
                | TokenKind::Boolean(_) 
                | TokenKind::Float(_) 
                | TokenKind::Integer(_) => { 
                    self.advance();

                    let kind = ExprKind::Literal(token.clone());
                    let span = token.span;

                    let expr = Expr { kind, span };

                    Ok(expr) 
                }

                TokenKind::EOF => Err(ParseError::UnexpectedEOF),
                token => Err(ParseError::InvalidSyntax(format!("Unexpected token: {:?}", token))),
            }
        } else {
            Err(ParseError::UnexpectedEOF)
        }
    }
}

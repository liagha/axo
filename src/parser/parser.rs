#![allow(dead_code)]
use crate::errors::ParseError;
use crate::parser::expression::Expression;
use crate::parser::statement::{EnumVariant, Statement};
use crate::tokens::{Operator, Punctuation, Token};

#[derive(Clone)]
pub enum Expr {
    Number(f64),
    Boolean(bool),
    Char(char),
    String(String),
    Identifier(String),
    Binary(Box<Expr>, Operator, Box<Expr>),
    Unary(Operator, Box<Expr>),
    Array(Vec<Expr>),
    Index(Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Lambda(Vec<Expr>, Box<Expr>),
    Assign(Box<Expr>, Box<Expr>),
    StructInit(Box<Expr>, Vec<(Expr, Expr)>),
    FieldAccess(Box<Expr>, Box<Expr>),
    Tuple(Vec<Expr>),
}

#[derive(Clone)]
pub enum Stmt {
    Expression(Expr),
    Assignment(Expr, Expr),
    Definition(Expr, Option<Expr>),
    CompoundAssignment(Expr, Operator, Expr),
    StructDef(Expr, Vec<(Expr, Expr)>),
    EnumDef(Expr, Vec<(Expr, Option<EnumVariant>)>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    Block(Vec<Stmt>),
    For(Option<Box<Stmt>>, Expr, Option<Box<Stmt>>, Box<Stmt>),
    Function(Expr, Vec<Expr>, Box<Stmt>),
    Return(Option<Expr>),
    Break(Option<Expr>),
    Continue,
}

pub struct Parser {
    tokens: Vec<Token>,
    pub current: usize,
    pub line: usize,
    pub column: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0, line: 1, column: 1 }
    }

    pub fn advance(&mut self) -> Option<Token> {
        while self.current < self.tokens.len() {
            let token = self.tokens[self.current].clone();
            self.current += 1;

            match &token {
                Token::Punctuation(Punctuation::Newline) => {
                    self.line += 1;
                    self.column = 0;
                    continue;
                }
                Token::Operator(Operator::DoubleSlash) => {
                    while let Some(next_token) = self.tokens.get(self.current) {
                        if matches!(next_token, Token::Punctuation(Punctuation::Newline)) {
                            break;
                        }
                        self.current += 1;
                    }
                    continue;
                }
                Token::Operator(Operator::SlashStar) => {
                    while let Some(next_token) = self.tokens.get(self.current) {
                        if matches!(next_token, Token::Operator(Operator::StarSlash)) {
                            self.current += 1;
                            break;
                        }
                        self.current += 1;
                    }
                    continue;
                }
                _ => {
                    self.column += 1;
                    return Some(token);
                }
            }
        }
        None
    }

    pub fn peek(&mut self) -> Option<&Token> {
        while let Some(token) = self.tokens.get(self.current) {
            match token {
                Token::Punctuation(Punctuation::Newline) => {
                    self.current += 1;
                    self.line += 1;
                    self.column = 0;
                }
                Token::Operator(Operator::DoubleSlash) => {
                    while let Some(next_token) = self.tokens.get(self.current) {
                        if matches!(next_token, Token::Punctuation(Punctuation::Newline)) {
                            break;
                        }
                        self.current += 1;
                    }
                    self.current += 1;
                }
                Token::Operator(Operator::SlashStar) => {
                    while let Some(next_token) = self.tokens.get(self.current) {
                        if matches!(next_token, Token::Operator(Operator::StarSlash)) {
                            self.current += 1;
                            break;
                        }
                        self.current += 1;
                    }
                }
                _ => return Some(token),
            }
        }
        None
    }

    pub fn match_token(&mut self, expected: &Token) -> bool {
        while let Some(token) = self.peek() {
            if token == &Token::Punctuation(Punctuation::Newline) {
                self.current += 1;
                self.line += 1;
                self.column = 0;
                continue;
            }
            if token == expected {
                self.advance();
                return true;
            }
            return false;
        }
        false
    }

    pub fn parse_program(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();
        while let Some(token) = self.peek() {
            if token == &Token::Punctuation(Punctuation::Newline) {
                self.advance();
                continue;
            }

            if token == &Token::EOF {
                break;
            }

            if token == &Token::Punctuation(Punctuation::LeftBrace) {
                self.advance();
                statements.push(self.parse_block()?);
            } else {
                statements.push(self.parse_statement()?);
            }
        }
        Ok(statements)
    }

    pub fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            Some(Token::Punctuation(Punctuation::LeftBracket)) => self.parse_array(),
            Some(Token::Punctuation(Punctuation::LeftParen)) => self.parse_tuple(),
            Some(Token::Operator(Operator::Pipe)) => self.parse_lambda(),
            Some(Token::Identifier(name)) => {
                let name_clone = name.clone();
                self.advance();

                let mut expr = Expr::Identifier(name_clone);

                loop {
                    match self.peek() {
                        Some(Token::Punctuation(Punctuation::LeftBracket)) => {
                            expr = self.parse_index(expr)?;
                        }
                        Some(Token::Punctuation(Punctuation::LeftParen)) => {
                            expr = self.parse_call(match expr {
                                Expr::Identifier(ref s) => s.clone(),
                                _ => return Err(ParseError::InvalidSyntax("Invalid function call".into())),
                            })?;
                        }
                        Some(Token::Operator(Operator::Dot)) => {
                            self.advance();
                            let field = self.parse_expression()?;

                            expr = Expr::FieldAccess(expr.into(), field.into());
                        }
                        Some(Token::Punctuation(Punctuation::LeftBrace)) => {
                            self.advance();
                            let mut fields = Vec::new();

                            while let Some(Token::Identifier(field_name)) = self.advance() {
                                if !self.match_token(&Token::Operator(Operator::Colon)) {
                                    return Err(ParseError::ExpectedOperator(Operator::Colon, "after field name".into()));
                                }

                                let value = self.parse_expression()?;
                                fields.push((Expr::Identifier(field_name), value));

                                if !self.match_token(&Token::Operator(Operator::Comma)) {
                                    break;
                                }
                            }

                            if !self.match_token(&Token::Punctuation(Punctuation::RightBrace)) {
                                return Err(ParseError::ExpectedPunctuation(Punctuation::RightBrace, "after struct fields".into()));
                            }

                            expr = Expr::StructInit(expr.into(), fields);
                        }
                        _ => break,
                    }
                }
                Ok(expr)
            }
            _ => match self.advance() {
                Some(Token::Integer(n)) => Ok(Expr::Number(n as f64)),
                Some(Token::Float(n)) => Ok(Expr::Number(n)),
                Some(Token::Boolean(b)) => Ok(Expr::Boolean(b)),
                Some(Token::Char(c)) => Ok(Expr::Char(c)),
                Some(Token::Str(s)) => Ok(Expr::String(s)),
                Some(Token::Punctuation(Punctuation::LeftParen)) => {
                    let expr = self.parse_expression()?;
                    if self.match_token(&Token::Punctuation(Punctuation::RightParen)) {
                        Ok(expr)
                    } else {
                        Err(ParseError::InvalidSyntax("Expected ')'".into()))
                    }
                }
                Some(Token::EOF) => Err(ParseError::UnexpectedEOF),
                Some(token) => Err(ParseError::UnexpectedToken(token, "".to_string())),
                None => Err(ParseError::UnexpectedEOF),
            },
        }
    }
}

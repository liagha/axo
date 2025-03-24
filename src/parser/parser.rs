#![allow(dead_code)]

use crate::errors::ParseError;
use crate::lexer::{OperatorKind, PunctuationKind, Token, TokenKind};
use crate::parser::expression::Expression;
use crate::parser::statement::{EnumVariant, Statement};

#[derive(Clone)]
pub enum Expr {
    Number(f64),
    Boolean(bool),
    Char(char),
    String(String),
    Identifier(String),
    Binary(Box<Expr>, OperatorKind, Box<Expr>),
    Unary(OperatorKind, Box<Expr>),
    Typed(Box<Expr>, Box<Expr>),
    Array(Vec<Expr>),
    Index(Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Lambda(Vec<Expr>, Box<Expr>),
    StructInit(Box<Expr>, Vec<Expr>),
    FieldAccess(Box<Expr>, Box<Expr>),
    Tuple(Vec<Expr>),
}

#[derive(Clone)]
pub enum Stmt {
    Expression(Expr),
    Assignment(Expr, Box<Stmt>),
    Definition(Expr, Option<Box<Stmt>>),
    CompoundAssignment(Expr, OperatorKind, Box<Stmt>),
    StructDef(Expr, Vec<Expr>),
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

            match &token.kind {
                TokenKind::Punctuation(PunctuationKind::Newline) => {
                    self.line += 1;
                    self.column = 0;
                    continue;
                }
                TokenKind::Operator(OperatorKind::DoubleSlash) => {
                    while let Some(next_token) = self.tokens.get(self.current) {
                        if matches!(next_token.kind, TokenKind::Punctuation(PunctuationKind::Newline)) {
                            break;
                        }
                        self.current += 1;
                    }
                    continue;
                }
                TokenKind::Operator(OperatorKind::SlashStar) => {
                    while let Some(next_token) = self.tokens.get(self.current) {
                        if matches!(next_token.kind, TokenKind::Operator(OperatorKind::StarSlash)) {
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
            match token.kind {
                TokenKind::Punctuation(PunctuationKind::Newline) => {
                    self.current += 1;
                    self.line += 1;
                    self.column = 0;
                }
                TokenKind::Operator(OperatorKind::DoubleSlash) => {
                    while let Some(next_token) = self.tokens.get(self.current) {
                        if matches!(next_token.kind, TokenKind::Punctuation(PunctuationKind::Newline)) {
                            break;
                        }
                        self.current += 1;
                    }
                    self.current += 1;
                }
                TokenKind::Operator(OperatorKind::SlashStar) => {
                    while let Some(next_token) = self.tokens.get(self.current) {
                        if matches!(next_token.kind, TokenKind::Operator(OperatorKind::StarSlash)) {
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

    pub fn match_token(&mut self, expected: &TokenKind) -> bool {
        while let Some(token) = self.peek() {
            if token.kind == TokenKind::Punctuation(PunctuationKind::Newline) {
                self.current += 1;
                self.line += 1;
                self.column = 0;
                continue;
            }
            if &token.kind == expected {
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
            if token.kind == TokenKind::Punctuation(PunctuationKind::Newline) {
                self.advance();
                continue;
            }

            if token.kind == TokenKind::EOF {
                break;
            }

            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }

    pub fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let peek = self.peek();
        let peek_kind = match peek {
            None => None,
            Some(token) => Some(token.kind.clone()),
        };

        match peek_kind {
            Some(TokenKind::Punctuation(PunctuationKind::LeftBracket)) => self.parse_array(),
            Some(TokenKind::Punctuation(PunctuationKind::LeftParen)) => self.parse_tuple(),
            Some(TokenKind::Operator(OperatorKind::Pipe)) => self.parse_lambda(),
            Some(TokenKind::Identifier(name)) => {
                let name_clone = name.clone();
                self.advance();

                let mut expr = Expr::Identifier(name_clone);

                loop {
                    let peek = self.peek();
                    let peek_kind = match peek {
                        None => None,
                        Some(token) => Some(token.kind.clone()),
                    };

                    match peek_kind {
                        Some(TokenKind::Punctuation(PunctuationKind::LeftBracket)) => {
                            expr = self.parse_index(expr)?;
                        }
                        Some(TokenKind::Punctuation(PunctuationKind::LeftParen)) => {
                            expr = self.parse_call(match expr {
                                Expr::Identifier(ref s) => s.clone(),
                                _ => return Err(ParseError::InvalidSyntax("Invalid function call".into())),
                            })?;
                        }
                        Some(TokenKind::Operator(OperatorKind::Dot)) => {
                            self.advance();
                            let field = self.parse_expression()?;

                            expr = Expr::FieldAccess(expr.into(), field.into());
                        }
                        Some(TokenKind::Punctuation(PunctuationKind::LeftBrace)) => {
                            // Call the dedicated method for struct initialization
                            expr = self.parse_struct_init(expr)?;
                        }
                        _ => break,
                    }
                }
                Ok(expr)
            }
            _ => {
                let advance = self.advance();
                let advance_kind = match advance {
                    None => None,
                    Some(token) => Some(token.kind.clone()),
                };

                match advance_kind {
                    Some(TokenKind::Integer(n)) => Ok(Expr::Number(n as f64)),
                    Some(TokenKind::Float(n)) => Ok(Expr::Number(n)),
                    Some(TokenKind::Boolean(b)) => Ok(Expr::Boolean(b)),
                    Some(TokenKind::Char(c)) => Ok(Expr::Char(c)),
                    Some(TokenKind::Str(s)) => Ok(Expr::String(s)),
                    Some(TokenKind::Punctuation(PunctuationKind::LeftParen)) => {
                        let expr = self.parse_expression()?;
                        if self.match_token(&TokenKind::Punctuation(PunctuationKind::RightParen)) {
                            Ok(expr)
                        } else {
                            Err(ParseError::InvalidSyntax("Expected ')'".into()))
                        }
                    }
                    Some(TokenKind::EOF) => Err(ParseError::UnexpectedEOF),
                    Some(token) => Err(ParseError::UnexpectedToken(token, "".to_string())),
                    None => Err(ParseError::UnexpectedEOF),
                }
            },
        }
    }
}

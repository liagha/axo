#![allow(dead_code)]
use crate::tokens::{Token, Operator, Keyword, Punctuation};
use crate::errors::ParseError;

#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    Boolean(bool),
    Char(char),
    String(String),
    Identifier(String),
    Binary(Box<Expr>, Operator, Box<Expr>),
    Unary(Operator, Box<Expr>),
}

#[derive(Clone)]
pub enum Stmt {
    Expression(Expr),
    Assignment(String, Expr),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    Block(Vec<Stmt>),
}

impl core::fmt::Debug for Stmt {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Stmt::Expression(expr) => write!(f, "{:?}", expr),
            Stmt::Assignment(name, expr) => write!(f, "Assignment({:?}, {:?})", name, expr),
            Stmt::If(cond, then, else_) => { write!(f, "If( Condition: {:?} | Then: {:?} | Else: {:?} )", cond, then, else_) }
            Stmt::While(cond, then) => write!(f, "While( Condition: {:?} | Then: {:?} )", cond, then),
            Stmt::Block(stmts) => {
                write!(f, "Block({:?})", stmts)
            }
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    fn advance(&mut self) -> Option<Token> {
        if self.current < self.tokens.len() {
            let token = self.tokens[self.current].clone();
            self.current += 1;
            Some(token)
        } else {
            None
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn peek_next(&self) -> Option<&Token> {
        self.tokens.get(self.current + 1)
    }

    fn match_token(&mut self, expected: &Token) -> bool {
        if let Some(token) = self.peek() {
            if token == expected {
                self.advance();
                return true;
            }
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

    fn parse_block(&mut self) -> Result<Stmt, ParseError> {
        let mut statements = Vec::new();
        while let Some(token) = self.peek() {
            if token == &Token::Punctuation(Punctuation::Newline) {
                self.advance();
                continue;
            }

            if token == &Token::Punctuation(Punctuation::RightBrace) {
                self.advance();
                return Ok(Stmt::Block(statements));
            }

            let stmt = self.parse_statement()?;
            statements.push(stmt);

            if self.peek() == Some(&Token::Punctuation(Punctuation::Semicolon)) {
                self.advance();
            }
        }
        Err(ParseError::UnexpectedEOF)
    }

    pub fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_term()?;

        while let Some(Token::Operator(op)) = self.peek() {
            if matches!(op, Operator::EqualEqual | Operator::NotEqual | Operator::GreaterThan
            | Operator::LessThan | Operator::GreaterThanOrEqual | Operator::LessThanOrEqual)
            {
                let op = op.clone();
                self.advance();
                let right = self.parse_term()?;
                expr = Expr::Binary(Box::new(expr), op, Box::new(right));
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_factor()?;
        while let Some(Token::Operator(op)) = self.peek() {
            if matches!(op, Operator::Plus | Operator::Minus | Operator::LogicalOr) {
                let op = op.clone();
                self.advance();
                let right = self.parse_factor()?;
                expr = Expr::Binary(Box::new(expr), op, Box::new(right));
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_unary()?;
        while let Some(Token::Operator(op)) = self.peek() {
            if matches!(op, Operator::Star | Operator::Slash | Operator::Percent | Operator::DotDot | Operator::LogicalAnd) {
                let op = op.clone();
                self.advance();
                let right = self.parse_unary()?; // Changed to parse_unary
                expr = Expr::Binary(Box::new(expr), op, Box::new(right));
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if let Some(Token::Operator(op)) = self.peek() {
            if matches!(op, Operator::Exclamation | Operator::Minus) {
                let op = op.clone();
                self.advance();
                let expr = self.parse_unary()?;
                return Ok(Expr::Unary(op, Box::new(expr)));
            }
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.advance() {
            Some(Token::Integer(n)) => Ok(Expr::Number(n as f64)),
            Some(Token::Float(n)) => Ok(Expr::Number(n)),
            Some(Token::Boolean(b)) => Ok(Expr::Boolean(b)),
            Some(Token::Identifier(name)) => Ok(Expr::Identifier(name)),
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
            Some(token) => Err(ParseError::UnexpectedToken(token.to_string())),
            None => Err(ParseError::UnexpectedEOF),
        }
    }

    pub fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        if let Some(Token::Keyword(kw)) = self.peek() {
            match kw {
                Keyword::If => self.parse_if_statement(),
                Keyword::While => self.parse_while_statement(),
                _ => Err(ParseError::InvalidSyntax("Unknown statement".to_string())),
            }
        } else if let Some(Token::Identifier(name)) = self.peek().cloned() {
            let current_pos = self.current;
            self.advance();

            if self.peek() == Some(&Token::Operator(Operator::Equal)) {
                self.advance();
                let value = self.parse_expression()?;
                if !self.match_token(&Token::Punctuation(Punctuation::Semicolon)) {
                    return Err(ParseError::InvalidSyntax("Expected ';' after assignment".to_string()));
                }
                return Ok(Stmt::Assignment(name, value));
            } else {
                self.current = current_pos;
                let expr = self.parse_expression()?;
                if !self.match_token(&Token::Punctuation(Punctuation::Semicolon)) {
                    return Err(ParseError::InvalidSyntax("Expected ';' after expression".to_string()));
                }
                return Ok(Stmt::Expression(expr));
            }
        } else {
            let expr = self.parse_expression()?;
            if !self.match_token(&Token::Punctuation(Punctuation::Semicolon)) {
                return Err(ParseError::InvalidSyntax("Expected ';' after expression".to_string()));
            }
            Ok(Stmt::Expression(expr))
        }
    }

    fn parse_if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.advance();
        if !self.match_token(&Token::Punctuation(Punctuation::LeftParen)) {
            return Err(ParseError::InvalidSyntax("Expected '(' after 'if'".to_string()));
        }
        let condition = self.parse_expression()?;
        if !self.match_token(&Token::Punctuation(Punctuation::RightParen)) {
            return Err(ParseError::InvalidSyntax("Expected ')' after condition".to_string()));
        }
        let then_branch = if self.match_token(&Token::Punctuation(Punctuation::LeftBrace)) {
            Box::new(self.parse_block()?)
        } else {
            Box::new(self.parse_statement()?)
        };

        let else_branch = if self.match_token(&Token::Keyword(Keyword::Else)) {
            if self.peek() == Some(&Token::Keyword(Keyword::If)) {
                Some(Box::new(self.parse_if_statement()?))
            } else if self.match_token(&Token::Punctuation(Punctuation::LeftBrace)) {
                Some(Box::new(self.parse_block()?))
            } else {
                Some(Box::new(self.parse_statement()?))
            }
        } else {
            None
        };

        Ok(Stmt::If(condition, then_branch, else_branch))
    }

    fn parse_while_statement(&mut self) -> Result<Stmt, ParseError> {
        self.advance();
        if !self.match_token(&Token::Punctuation(Punctuation::LeftParen)) {
            return Err(ParseError::InvalidSyntax("Expected '(' after 'if'".to_string()));
        }
        let condition = self.parse_expression()?;
        if !self.match_token(&Token::Punctuation(Punctuation::RightParen)) {
            return Err(ParseError::InvalidSyntax("Expected ')' after condition".to_string()));
        }
        let then_branch = if self.match_token(&Token::Punctuation(Punctuation::LeftBrace)) {
            Box::new(self.parse_block()?)
        } else {
            Box::new(self.parse_statement()?)
        };

        Ok(Stmt::While(condition, then_branch))
    }
}
use crate::parser::{Parser, Statement};
use crate::parser::statement::EnumVariant;
use crate::lexer::{OperatorKind, PunctuationKind, TokenKind, Token};
use crate::parser::error::{ParseError, SyntaxPosition, SyntaxType};

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
    StructInit(Box<Expr>, Vec<Box<Expr>>),
    FieldAccess(Box<Expr>, Box<Expr>),
    Tuple(Vec<Expr>),
    Assignment(Box<Expr>, Box<Expr>),
    Definition(Box<Expr>, Option<Box<Expr>>),
    CompoundAssignment(Box<Expr>, OperatorKind, Box<Expr>),
    StructDef(Box<Expr>, Vec<Expr>),
    EnumDef(Box<Expr>, Vec<(Expr, Option<EnumVariant>)>),
    If(Box<Expr>, Box<Expr>, Option<Box<Expr>>),
    While(Box<Expr>, Box<Expr>),
    Block(Vec<Expr>),
    For(Box<Expr>, Box<Expr>),
    Function(Box<Expr>, Vec<Expr>, Box<Expr>),
    Return(Option<Box<Expr>>),
    Break(Option<Box<Expr>>),
    Continue,
}

pub trait Expression {
    fn parse_unary(&mut self) -> Result<Expr, ParseError>;
    fn parse_factor(&mut self) -> Result<Expr, ParseError>;
    fn parse_term(&mut self) -> Result<Expr, ParseError>;
    fn parse_expression(&mut self) -> Result<Expr, ParseError>;
    fn parse_array(&mut self) -> Result<Expr, ParseError>;
    fn parse_index(&mut self, left: Expr) -> Result<Expr, ParseError>;
    fn parse_call(&mut self, name: String) -> Result<Expr, ParseError>;
    fn parse_lambda(&mut self) -> Result<Expr, ParseError>;
    fn parse_tuple(&mut self) -> Result<Expr, ParseError>;
    fn parse_struct_init(&mut self, struct_name: Expr) -> Result<Expr, ParseError>;
}

impl Expression for Parser {
    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if let Some(Token { kind: TokenKind::Operator(op), ..}) = self.peek() {
            if op.is_unary() {
                let op = op.clone();
                self.advance();
                let expr = self.parse_unary()?;
                return Ok(Expr::Unary(op, Box::new(expr)));
            }
        }

        self.parse_primary()
    }
    fn parse_factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_unary()?;
        while let Some(Token { kind: TokenKind::Operator(op), ..}) = self.peek() {
            if op.is_factor() {
                let op = op.clone();
                self.advance();
                let right = self.parse_unary()?;
                expr = Expr::Binary(Box::new(expr), op, Box::new(right));
            } else if op == &OperatorKind::Colon {
                self.advance();
                let right = self.parse_unary()?;
                expr = Expr::Typed(Box::new(expr), Box::new(right));
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_factor()?;
        while let Some(Token { kind: TokenKind::Operator(op), ..}) = self.peek() {
            if op.is_term() {
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
    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_term()?;

        while let Some(Token { kind: TokenKind::Operator(op), ..}) = self.peek() {
            if op.is_expression() {
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
    fn parse_array(&mut self) -> Result<Expr, ParseError> {
        self.advance();
        let mut elements = Vec::new();

        if self.match_token(&TokenKind::Punctuation(PunctuationKind::RightBracket)) {
            return Ok(Expr::Array(elements));
        }

        elements.push(self.parse_expression()?);

        while self.match_token(&TokenKind::Operator(OperatorKind::Comma)) {
            elements.push(self.parse_expression()?);
        }

        if !self.match_token(&TokenKind::Punctuation(PunctuationKind::RightBracket)) {
            let err = ParseError::ExpectedToken(TokenKind::Punctuation(PunctuationKind::RightBracket), SyntaxPosition::After, SyntaxType::ArrayElements);

            return Err(err);
        }

        Ok(Expr::Array(elements))
    }

    fn parse_index(&mut self, left: Expr) -> Result<Expr, ParseError> {
        self.advance();
        let index = self.parse_expression()?;

        if !self.match_token(&TokenKind::Punctuation(PunctuationKind::RightBracket)) {
            let err = ParseError::ExpectedToken(TokenKind::Punctuation(PunctuationKind::RightBracket), SyntaxPosition::After, SyntaxType::ArrayElements);

            return Err(err);
        }

        Ok(Expr::Index(Box::new(left), Box::new(index)))
    }


    fn parse_call(&mut self, name: String) -> Result<Expr, ParseError> {
        self.advance();
        let mut arguments = Vec::new();

        if self.match_token(&TokenKind::Punctuation(PunctuationKind::RightParen)) {
            return Ok(Expr::Call(Expr::Identifier(name).into(), arguments));
        }

        arguments.push(self.parse_expression()?);

        while self.match_token(&TokenKind::Operator(OperatorKind::Comma)) {
            arguments.push(self.parse_expression()?);
        }

        if !self.match_token(&TokenKind::Punctuation(PunctuationKind::RightParen)) {
            let err = ParseError::ExpectedToken(TokenKind::Punctuation(PunctuationKind::RightParen), SyntaxPosition::After, SyntaxType::FunctionCall);

            return Err(err);
        }

        Ok(Expr::Call(Expr::Identifier(name).into(), arguments))
    }

    fn parse_lambda(&mut self) -> Result<Expr, ParseError> {
        self.advance();
        let mut parameters = Vec::new();

        if !self.match_token(&TokenKind::Operator(OperatorKind::Pipe)) {
            if let Some(Token { kind: TokenKind::Identifier(name), ..}) = self.advance() {
                parameters.push(Expr::Identifier(name));

                while self.match_token(&TokenKind::Operator(OperatorKind::Comma)) {
                    if let Some(Token { kind: TokenKind::Identifier(name), .. }) = self.advance() {
                        parameters.push(Expr::Identifier(name));
                    } else {
                        let err = ParseError::ExpectedSyntax(SyntaxType::ParameterName);

                        return Err(err);
                    }
                }

                if !self.match_token(&TokenKind::Operator(OperatorKind::Pipe)) {
                    let err = ParseError::ExpectedToken(TokenKind::Operator(OperatorKind::Pipe), SyntaxPosition::After, SyntaxType::LambdaParameters);

                    return Err(err);
                }
            } else {
                let err = ParseError::ExpectedToken(TokenKind::Operator(OperatorKind::Pipe), SyntaxPosition::After, SyntaxType::LambdaParameters);

                return Err(err);
            }
        }

        let body = self.parse_expression()?;

        Ok(Expr::Lambda(parameters, Box::new(body)))
    }

    fn parse_tuple(&mut self) -> Result<Expr, ParseError> {
        self.advance();
        let mut elements = Vec::new();

        if self.match_token(&TokenKind::Punctuation(PunctuationKind::RightParen)) {
            return Ok(Expr::Tuple(elements));
        }

        elements.push(self.parse_expression()?);

        if self.match_token(&TokenKind::Operator(OperatorKind::Comma)) {
            if !self.match_token(&TokenKind::Punctuation(PunctuationKind::RightParen)) {
                elements.push(self.parse_expression()?);

                while self.match_token(&TokenKind::Operator(OperatorKind::Comma)) {
                    if let Some(Token { kind: TokenKind::Punctuation(PunctuationKind::RightParen), .. }) = self.peek() {
                        break;
                    }
                    elements.push(self.parse_expression()?);
                }

                if !self.match_token(&TokenKind::Punctuation(PunctuationKind::RightParen)) {
                    let err = ParseError::ExpectedToken(TokenKind::Punctuation(PunctuationKind::RightParen), SyntaxPosition::After, SyntaxType::TupleElements);

                    return Err(err);
                }
            }

            Ok(Expr::Tuple(elements))
        } else {
            if !self.match_token(&TokenKind::Punctuation(PunctuationKind::RightParen)) {
                let err = ParseError::ExpectedToken(TokenKind::Punctuation(PunctuationKind::RightParen), SyntaxPosition::After, SyntaxType::Expression);

                return Err(err);
            }

            Ok(elements.pop().unwrap())
        }
    }

    fn parse_struct_init(&mut self, struct_name: Expr) -> Result<Expr, ParseError> {
        self.advance();

        let mut statements = Vec::new();

        while let Some(token) = self.peek() {
            println!("{:?}", token);

            match token.kind {
                TokenKind::Punctuation(PunctuationKind::RightBrace) |
                TokenKind::Punctuation(PunctuationKind::Semicolon) => {
                    self.advance();
                    return Ok(Expr::StructInit(struct_name.into(), statements))
                }
                TokenKind::Operator(OperatorKind::Comma) => {
                    self.advance();
                }
                _ => {
                    let stmt = self.parse_statement()?;
                    statements.push(stmt.into());
                }
            }
        }

        Err(ParseError::UnexpectedEOF)
    }
}

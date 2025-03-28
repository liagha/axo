use crate::parser::{Parser, Statement};
use crate::lexer::{OperatorKind, PunctuationKind, TokenKind, Token, Span};
use crate::parser::error::{ParseError, SyntaxPosition, SyntaxType};

#[derive(Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Clone)]
pub enum ExprKind {
    Literal(Token),
    Identifier(String),
    Binary(Box<Expr>, Token, Box<Expr>),
    Unary(Token, Box<Expr>),
    Typed(Box<Expr>, Box<Expr>),
    Array(Vec<Expr>),
    Index(Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Closure(Vec<Expr>, Box<Expr>),
    StructInit(Box<Expr>, Vec<Box<Expr>>),
    FieldAccess(Box<Expr>, Box<Expr>),
    Tuple(Vec<Expr>),
    Assignment(Box<Expr>, Box<Expr>),
    Definition(Box<Expr>, Option<Box<Expr>>),
    StructDef(Box<Expr>, Vec<Expr>),
    Enum(Box<Expr>, Vec<Expr>),
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
    fn parse_call(&mut self, name: Expr) -> Result<Expr, ParseError>;
    fn parse_closure(&mut self) -> Result<Expr, ParseError>;
    fn parse_tuple(&mut self) -> Result<Expr, ParseError>;
    fn parse_struct(&mut self, struct_name: Expr) -> Result<Expr, ParseError>;
}

impl Expression for Parser {
    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if let Some(Token { kind: TokenKind::Operator(op), span: Span { start, .. } }) = self.peek().cloned() {
            if op.is_unary() {
                let op = self.advance().unwrap();

                let unary = self.parse_unary()?;
                let end = unary.span.end;

                let span = Span { start, end };

                let kind = ExprKind::Unary(op, unary.into());

                let expr = Expr { kind, span };

                return Ok(expr);
            }
        }

        self.parse_primary()
    }
    fn parse_factor(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;

        while let Some(Token { kind: TokenKind::Operator(op), ..}) = self.peek().cloned() {
            if op.is_factor() {
                let op = self.advance().unwrap();

                let right = self.parse_unary()?;

                let start = left.span.start;
                let end = right.span.end;
                let span = Span { start, end };

                let kind = ExprKind::Binary(left.into(), op, right.into());  

                left = Expr { kind, span };
            } else if op == OperatorKind::Colon {
                self.advance();

                let right = self.parse_unary()?;

                let start = left.span.start;
                let end = right.span.end;
                let span = Span { start, end };

                let kind = ExprKind::Typed(left.into(), right.into());  

                left = Expr { kind, span };
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_factor()?;

        while let Some(Token { kind: TokenKind::Operator(op), ..}) = self.peek().cloned() {
            if op.is_term() {
                let op = self.advance().unwrap();

                let right = self.parse_factor()?;

                let start = left.span.start;
                let end = right.span.end;
                let span = Span { start, end };

                let kind = ExprKind::Binary(left.into(), op, right.into());  

                left = Expr { kind, span };
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_term()?;

        while let Some(Token { kind: TokenKind::Operator(op), ..}) = self.peek().cloned() {
            if op.is_expression() {
                let op = self.advance().unwrap();

                let right = self.parse_term()?;

                let start = left.span.start;
                let end = right.span.end;
                let span = Span { start, end };

                let kind = ExprKind::Binary(left.into(), op, right.into());  

                left = Expr { kind, span };
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_index(&mut self, left: Expr) -> Result<Expr, ParseError> {
        self.advance();

        let Expr { span: Span { start, .. }, .. } = left;

        let index = self.parse_expression()?;

        if let Some(Token { kind: TokenKind::Punctuation(PunctuationKind::RightBracket), span: Span { end, ..} }) = self.advance() {
            let kind = ExprKind::Index(left.into(), index.into());
            let span = Span { start, end };
            let expr = Expr { kind, span };

            Ok(expr)
        } else {
            let err = ParseError::ExpectedToken(TokenKind::Punctuation(PunctuationKind::RightBracket), SyntaxPosition::After, SyntaxType::ArrayElements);

            Err(err)
        }
    }

    fn parse_array(&mut self) -> Result<Expr, ParseError> {
        let Token { span: Span { start, .. }, .. } = self.advance().unwrap();

        let mut elements = Vec::new();
        
        while let Some(token) = self.peek().cloned() {
            match token {
                Token { kind: TokenKind::Punctuation(PunctuationKind::RightBracket), span: Span { end, ..} } => {
                    self.advance();

                    return Ok(Expr {
                        kind: ExprKind::Array(elements),
                        span: Span { 
                            start, 
                            end 
                        }
                    });
                }
                Token { kind: TokenKind::Operator(OperatorKind::Comma), .. } => {
                    self.advance();
                }
                _ => {
                    let expr = self.parse_expression()?;
                    elements.push(expr.into());
                }
            }
        }


        let err = ParseError::ExpectedToken(TokenKind::Punctuation(PunctuationKind::RightBrace), SyntaxPosition::After, SyntaxType::ArrayElements);

        Err(err)

    }

    fn parse_call(&mut self, name: Expr) -> Result<Expr, ParseError> {
        self.advance();

        let Expr { span: Span { start, .. }, .. } = name;

        let mut parameters = Vec::new();
        
        while let Some(token) = self.peek().cloned() {
            match token {
                Token { kind: TokenKind::Punctuation(PunctuationKind::RightParen), span: Span { end, ..} } => {
                    self.advance();

                    return Ok(Expr {
                        kind: ExprKind::Call(name.into(), parameters),
                        span: Span { 
                            start, 
                            end 
                        }
                    });
                }
                Token { kind: TokenKind::Operator(OperatorKind::Comma), .. } => {
                    self.advance();
                }
                _ => {
                    let expr = self.parse_expression()?;
                    parameters.push(expr.into());
                }
            }
        }

        let err = ParseError::ExpectedToken(TokenKind::Punctuation(PunctuationKind::RightParen), SyntaxPosition::After, SyntaxType::FunctionParameters);

        Err(err)
    }

    fn parse_closure(&mut self) -> Result<Expr, ParseError> {
        let Token { span: Span { start, .. }, .. } = self.advance().unwrap();

        let mut parameters = Vec::new();
        
        while let Some(token) = self.peek().cloned() {
            match token {
                Token { kind: TokenKind::Operator(OperatorKind::Pipe), span: Span { end, ..} } => {
                    let body = self.parse_statement()?;

                    return Ok(Expr {
                        kind: ExprKind::Closure(parameters, body.into()),
                        span: Span { 
                            start, 
                            end 
                        }
                    });
                }
                Token { kind: TokenKind::Operator(OperatorKind::Comma), .. } => {
                    self.advance();
                }
                _ => {
                    let expr = self.parse_expression()?;
                    parameters.push(expr.into());
                }
            }
        }

        let err = ParseError::ExpectedToken(TokenKind::Operator(OperatorKind::Pipe), SyntaxPosition::After, SyntaxType::ClosureParameters);

        Err(err)
    }

    fn parse_tuple(&mut self) -> Result<Expr, ParseError> {
        let Token { span: Span { start, .. }, .. } = self.advance().unwrap();

        let mut parameters = Vec::new();
        
        while let Some(token) = self.peek().cloned() {
            match token {
                Token { kind: TokenKind::Punctuation(PunctuationKind::RightParen), span: Span { end, ..} } => {
                    self.advance();

                    if parameters.len() != 1 { 
                        return Ok(Expr {
                            kind: ExprKind::Tuple(parameters),
                            span: Span { 
                                start, 
                                end 
                            }
                        });
                    } else {
                        return Ok(parameters.pop().unwrap())
                    }
                }
                Token { kind: TokenKind::Operator(OperatorKind::Comma), .. } => {
                    self.advance();
                }
                _ => {
                    let expr = self.parse_expression()?;
                    parameters.push(expr.into());
                }
            }
        }

        let err = ParseError::ExpectedToken(TokenKind::Punctuation(PunctuationKind::RightParen), SyntaxPosition::After, SyntaxType::FunctionParameters);

        Err(err)
    }

    fn parse_struct(&mut self, struct_name: Expr) -> Result<Expr, ParseError> {
        self.advance();

        let Expr { span: Span { start, .. }, .. } = struct_name;

        let mut statements = Vec::new();

        while let Some(token) = self.peek() {
            match token.kind {
                TokenKind::Punctuation(PunctuationKind::RightBrace) |
                    TokenKind::Punctuation(PunctuationKind::Semicolon) => {
                        let Token { span: Span { end, .. }, .. } = self.advance().unwrap();

                        let kind = ExprKind::StructInit(struct_name.into(), statements);
                        let expr = Expr { kind, span: Span { start, end} };

                        return Ok(expr)
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

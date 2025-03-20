use crate::errors::ParseError;
use crate::parser::Parser;
use crate::parser::parser::Expr;
use crate::tokens::{Operator, Punctuation, Token};

pub trait Expression {
    fn parse_factor(&mut self) -> Result<Expr, ParseError>;
    fn parse_term(&mut self) -> Result<Expr, ParseError>;
    fn parse_expression(&mut self) -> Result<Expr, ParseError>;
    fn parse_unary(&mut self) -> Result<Expr, ParseError>;
    fn parse_array(&mut self) -> Result<Expr, ParseError>;
    fn parse_index(&mut self, left: Expr) -> Result<Expr, ParseError>;
    fn parse_call(&mut self, name: String) -> Result<Expr, ParseError>;
    fn parse_lambda(&mut self) -> Result<Expr, ParseError>;
    fn parse_tuple(&mut self) -> Result<Expr, ParseError>;
}

impl Expression for Parser {
    fn parse_factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_unary()?;
        while let Some(Token::Operator(op)) = self.peek() {
            if op.is_factor() {
                let op = op.clone();
                self.advance();
                let right = self.parse_unary()?;
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

        while let Some(Token::Operator(op)) = self.peek() {
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

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if let Some(Token::Operator(op)) = self.peek() {
            if op.is_unary() {
                let op = op.clone();
                self.advance();
                let expr = self.parse_unary()?;
                return Ok(Expr::Unary(op, Box::new(expr)));
            }
        }
        self.parse_primary()
    }

    fn parse_array(&mut self) -> Result<Expr, ParseError> {
        self.advance();
        let mut elements = Vec::new();

        if self.match_token(&Token::Punctuation(Punctuation::RightBracket)) {
            return Ok(Expr::Array(elements));
        }

        elements.push(self.parse_expression()?);

        while self.match_token(&Token::Operator(Operator::Comma)) {
            elements.push(self.parse_expression()?);
        }

        if !self.match_token(&Token::Punctuation(Punctuation::RightBracket)) {
            return Err(ParseError::ExpectedPunctuation(Punctuation::RightBracket, "after array elements".to_string()));
        }

        Ok(Expr::Array(elements))
    }

    fn parse_index(&mut self, left: Expr) -> Result<Expr, ParseError> {
        self.advance();
        let index = self.parse_expression()?;

        if !self.match_token(&Token::Punctuation(Punctuation::RightBracket)) {
            return Err(ParseError::ExpectedPunctuation(Punctuation::RightBracket, "after array elements".to_string()));
        }

        Ok(Expr::Index(Box::new(left), Box::new(index)))
    }


    fn parse_call(&mut self, name: String) -> Result<Expr, ParseError> {
        self.advance();
        let mut arguments = Vec::new();

        if self.match_token(&Token::Punctuation(Punctuation::RightParen)) {
            return Ok(Expr::Call(Expr::Identifier(name).into(), arguments));
        }

        arguments.push(self.parse_expression()?);

        while self.match_token(&Token::Operator(Operator::Comma)) {
            arguments.push(self.parse_expression()?);
        }

        if !self.match_token(&Token::Punctuation(Punctuation::RightParen)) {
            return Err(ParseError::ExpectedPunctuation(Punctuation::RightParen, "after function arguments".to_string()));
        }

        Ok(Expr::Call(Expr::Identifier(name).into(), arguments))
    }

    fn parse_lambda(&mut self) -> Result<Expr, ParseError> {
        self.advance();
        let mut parameters = Vec::new();

        if !self.match_token(&Token::Operator(Operator::Pipe)) {
            if let Some(Token::Identifier(name)) = self.advance() {
                parameters.push(Expr::Identifier(name));

                while self.match_token(&Token::Operator(Operator::Comma)) {
                    if let Some(Token::Identifier(name)) = self.advance() {
                        parameters.push(Expr::Identifier(name));
                    } else {
                        return Err(ParseError::ExpectedSyntax("parameter name".to_string()));
                    }
                }

                if !self.match_token(&Token::Operator(Operator::Pipe)) {
                    return Err(ParseError::ExpectedOperator(Operator::Pipe, "after lambda parameters".to_string()));
                }
            } else {
                return Err(ParseError::ExpectedSyntax("parameter name or '|'".to_string()));
            }
        }

        let body = self.parse_expression()?;

        Ok(Expr::Lambda(parameters, Box::new(body)))
    }

    fn parse_tuple(&mut self) -> Result<Expr, ParseError> {
        self.advance();
        let mut elements = Vec::new();

        if self.match_token(&Token::Punctuation(Punctuation::RightParen)) {
            return Ok(Expr::Tuple(elements));
        }

        elements.push(self.parse_expression()?);

        if self.match_token(&Token::Operator(Operator::Comma)) {
            if !self.match_token(&Token::Punctuation(Punctuation::RightParen)) {
                elements.push(self.parse_expression()?);

                while self.match_token(&Token::Operator(Operator::Comma)) {
                    if self.peek() == Some(&Token::Punctuation(Punctuation::RightParen)) {
                        break;
                    }
                    elements.push(self.parse_expression()?);
                }

                if !self.match_token(&Token::Punctuation(Punctuation::RightParen)) {
                    return Err(ParseError::ExpectedPunctuation(
                        Punctuation::RightParen,
                        "after tuple elements".to_string()
                    ));
                }
            }

            Ok(Expr::Tuple(elements))
        } else {
            if !self.match_token(&Token::Punctuation(Punctuation::RightParen)) {
                return Err(ParseError::ExpectedPunctuation(
                    Punctuation::RightParen,
                    "after expression".to_string()
                ));
            }

            Ok(elements.pop().unwrap())
        }
    }
}
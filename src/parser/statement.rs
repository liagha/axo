#![allow(dead_code)]

use crate::{
    errors::ParseError,
    parser::{Parser, parser::{Expr,Stmt}, expression::Expression},
    tokens::{Keyword, Operator, Punctuation, Token},
};

#[derive(Debug, Clone)]
pub enum EnumVariant {
    Tuple(Vec<Expr>),
    Struct(Vec<(Expr, Expr)>),
    Discriminant(Expr),
}

pub trait Statement {
    fn parse_statement(&mut self) -> Result<Stmt, ParseError>;
    fn parse_let_statement(&mut self) -> Result<Stmt, ParseError>;
    fn parse_block(&mut self) -> Result<Stmt, ParseError>;
    fn parse_function_declaration(&mut self) -> Result<Stmt, ParseError>;
    fn parse_if_statement(&mut self) -> Result<Stmt, ParseError>;
    fn parse_while_statement(&mut self) -> Result<Stmt, ParseError>;
    fn parse_for_statement(&mut self) -> Result<Stmt, ParseError>;
    fn parse_return_statement(&mut self) -> Result<Stmt, ParseError>;
    fn parse_break_statement(&mut self) -> Result<Stmt, ParseError>;
    fn parse_continue_statement(&mut self) -> Result<Stmt, ParseError>;
    fn parse_condition(&mut self) -> Result<Expr, ParseError>;
    fn parse_struct_definition(&mut self) -> Result<Stmt, ParseError>;
    fn parse_enum_definition(&mut self) -> Result<Stmt, ParseError>;
}

impl Statement for Parser {
    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        if let Some(Token::Keyword(kw)) = self.peek() {
            match kw {
                Keyword::If => self.parse_if_statement(),
                Keyword::While => self.parse_while_statement(),
                Keyword::For => self.parse_for_statement(),
                Keyword::Fn => self.parse_function_declaration(),
                Keyword::Return => self.parse_return_statement(),
                Keyword::Break => self.parse_break_statement(),
                Keyword::Continue => self.parse_continue_statement(),
                Keyword::Let => self.parse_let_statement(),
                Keyword::Struct => self.parse_struct_definition(),
                Keyword::Enum => self.parse_enum_definition(),
                _ => Err(ParseError::UnknownStatement),
            }
        } else {
            let left = match self.parse_expression() {
                Ok(expr) => expr,
                Err(e) => return Err(e),
            };

            if self.peek() == Some(&Token::Operator(Operator::Equal)) {
                self.advance();
                let value = self.parse_expression()?;

                if !self.match_token(&Token::Punctuation(Punctuation::Semicolon)) {
                    return Err(ParseError::ExpectedPunctuation(Punctuation::Semicolon, "after assignment".to_string()));
                }

                Ok(Stmt::Assignment(left, value))
            } else if Operator::is_compound_token(self.peek()) {
                let operator = Token::get_operator(self.peek()).unwrap();

                self.advance();
                let value = self.parse_expression()?;

                if !self.match_token(&Token::Punctuation(Punctuation::Semicolon)) {
                    return Err(ParseError::ExpectedPunctuation(Punctuation::Semicolon, "after compound assignment".to_string()));
                }

                Ok(Stmt::CompoundAssignment(left, operator, value))
            } else {
                if !self.match_token(&Token::Punctuation(Punctuation::Semicolon)) {
                    return Err(ParseError::ExpectedPunctuation(Punctuation::Semicolon, "after expression".to_string()));
                }

                Ok(Stmt::Expression(left))
            }
        }
    }
    fn parse_let_statement(&mut self) -> Result<Stmt, ParseError> {
        self.advance();

        let name = self.parse_expression()?;

        let initializer = if self.match_token(&Token::Operator(Operator::Equal)) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        if !self.match_token(&Token::Punctuation(Punctuation::Semicolon)) {
            return Err(ParseError::ExpectedPunctuation(Punctuation::Semicolon, "after variable declaration".to_string()));
        }

        Ok(Stmt::Definition(name, initializer))
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
        }

        Err(ParseError::UnexpectedEOF)
    }

    fn parse_function_declaration(&mut self) -> Result<Stmt, ParseError> {
        self.advance();

        let name = if let Some(Token::Identifier(name)) = self.advance() {
            name
        } else {
            return Err(ParseError::ExpectedSyntax("function name".to_string()));
        };

        if !self.match_token(&Token::Punctuation(Punctuation::LeftParen)) {
            return Err(ParseError::ExpectedPunctuation(Punctuation::LeftParen, "after function name".to_string()));
        }

        let mut parameters = Vec::new();

        if !self.match_token(&Token::Punctuation(Punctuation::RightParen)) {
            if let Some(Token::Identifier(param)) = self.advance() {
                parameters.push(Expr::Identifier(param));

                while self.match_token(&Token::Operator(Operator::Comma)) {
                    if let Some(Token::Identifier(param)) = self.advance() {
                        parameters.push(Expr::Identifier(param));
                    } else {
                        return Err(ParseError::ExpectedSyntax("parameter name".to_string()));
                    }
                }

                if !self.match_token(&Token::Punctuation(Punctuation::RightParen)) {
                    return Err(ParseError::ExpectedPunctuation(Punctuation::RightParen, "after parameters".to_string()));
                }
            } else {
                return Err(ParseError::ExpectedSyntax("parameter name or ')'".to_string()));
            }
        }

        let body = if !self.match_token(&Token::Punctuation(Punctuation::LeftBrace)) {
            self.parse_statement()?
        } else {
            self.parse_block()?
        };

        Ok(Stmt::Function(Expr::Identifier(name), parameters, Box::new(body)))
    }

    fn parse_if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.advance();

        let condition = self.parse_condition()?;

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

        let condition = self.parse_condition()?;

        let body = if self.match_token(&Token::Punctuation(Punctuation::LeftBrace)) {
            Box::new(self.parse_block()?)
        } else {
            Box::new(self.parse_statement()?)
        };

        Ok(Stmt::While(condition, body))
    }

    fn parse_for_statement(&mut self) -> Result<Stmt, ParseError> {
        self.advance();

        let has_paren = self.match_token(&Token::Punctuation(Punctuation::LeftParen));

        let initializer = if self.match_token(&Token::Punctuation(Punctuation::Semicolon)) {
            None
        } else {
            Some(self.parse_statement()?)
        };

        let condition = if self.peek() == Some(&Token::Punctuation(Punctuation::Semicolon)) {
            Expr::Boolean(true)
        } else {
            self.parse_expression()?
        };

        if !self.match_token(&Token::Punctuation(Punctuation::Semicolon)) {
            return Err(ParseError::ExpectedPunctuation(Punctuation::Semicolon, "after for condition".to_string()));
        }

        let increment = if (has_paren && self.peek() == Some(&Token::Punctuation(Punctuation::RightParen))) ||
            (!has_paren && (self.peek() == Some(&Token::Punctuation(Punctuation::LeftBrace)) ||
                self.peek() == Some(&Token::Punctuation(Punctuation::Semicolon)))) {
            None
        } else {
            if let Some(Token::Identifier(name)) = self.peek().cloned() {
                self.advance();
                if self.peek() == Some(&Token::Operator(Operator::PlusEqual)) {
                    self.advance();
                    let value = self.parse_expression()?;
                    Some(Stmt::CompoundAssignment(Expr::Identifier(name), Operator::PlusEqual, value))
                } else {
                    self.current -= 1;
                    Some(Stmt::Expression(self.parse_expression()?))
                }
            } else {
                Some(Stmt::Expression(self.parse_expression()?))
            }
        };

        if has_paren {
            if !self.match_token(&Token::Punctuation(Punctuation::RightParen)) {
                return Err(ParseError::ExpectedPunctuation(Punctuation::RightParen, "after for clauses".to_string()));
            }
        }

        let body = if self.match_token(&Token::Punctuation(Punctuation::LeftBrace)) {
            self.parse_block()?
        } else {
            self.parse_statement()?
        };

        Ok(Stmt::For(initializer.map(Box::new), condition, increment.map(Box::new), Box::new(body)))
    }

    fn parse_return_statement(&mut self) -> Result<Stmt, ParseError> {
        self.advance();

        if self.match_token(&Token::Punctuation(Punctuation::Semicolon)) {
            return Ok(Stmt::Return(None));
        }

        let value = self.parse_expression()?;

        if !self.match_token(&Token::Punctuation(Punctuation::Semicolon)) {
            return Err(ParseError::ExpectedPunctuation(Punctuation::Semicolon, "after return value".to_string()));
        }

        Ok(Stmt::Return(Some(value)))
    }

    fn parse_break_statement(&mut self) -> Result<Stmt, ParseError> {
        self.advance();

        if self.match_token(&Token::Punctuation(Punctuation::Semicolon)) {
            return Ok(Stmt::Break(None));
        }

        let value = self.parse_expression()?;

        if !self.match_token(&Token::Punctuation(Punctuation::Semicolon)) {
            return Err(ParseError::ExpectedPunctuation(Punctuation::Semicolon, "after break value".to_string()));
        }

        Ok(Stmt::Break(Some(value)))
    }

    fn parse_continue_statement(&mut self) -> Result<Stmt, ParseError> {
        self.advance();

        if !self.match_token(&Token::Punctuation(Punctuation::Semicolon)) {
            return Err(ParseError::ExpectedPunctuation(Punctuation::Semicolon, "after 'continue'".to_string()));
        }

        Ok(Stmt::Continue)
    }

    fn parse_condition(&mut self) -> Result<Expr, ParseError> {
        let has_paren = self.match_token(&Token::Punctuation(Punctuation::LeftParen));

        let condition = self.parse_expression()?;

        if has_paren {
            if !self.match_token(&Token::Punctuation(Punctuation::RightParen)) {
                return Err(ParseError::ExpectedPunctuation(Punctuation::RightParen, "after condition".to_string()));
            }
        }

        Ok(condition)
    }

    fn parse_struct_definition(&mut self) -> Result<Stmt, ParseError> {
        self.advance();

        let name = if let Some(Token::Identifier(name)) = self.advance() {
            name
        } else {
            return Err(ParseError::ExpectedSyntax("Expected struct name".to_string()));
        };

        if !self.match_token(&Token::Punctuation(Punctuation::LeftBrace)) {
            return Err(ParseError::ExpectedPunctuation(Punctuation::LeftBrace, "Expected '{' after struct name".to_string()));
        }

        let mut fields = Vec::new();

        while let Some(Token::Identifier(field_name)) = self.advance() {
            if !self.match_token(&Token::Operator(Operator::Colon)) {
                return Err(ParseError::ExpectedOperator(Operator::Colon, "Expected ':' after field name".to_string()));
            }

            let field_type = if let Some(Token::Identifier(type_name)) = self.advance() {
                type_name
            } else {
                return Err(ParseError::ExpectedSyntax("Expected field type".to_string()));
            };

            fields.push((Expr::Identifier(field_name), Expr::Identifier(field_type)));

            if !self.match_token(&Token::Operator(Operator::Comma)) {
                break;
            }
        }

        if !self.match_token(&Token::Punctuation(Punctuation::RightBrace)) {
            return Err(ParseError::ExpectedPunctuation(Punctuation::RightBrace, "Expected '}' after struct fields".to_string()));
        }

        Ok(Stmt::StructDef(Expr::Identifier(name), fields))
    }

    fn parse_enum_definition(&mut self) -> Result<Stmt, ParseError> {
        self.advance();

        let name = if let Some(Token::Identifier(name)) = self.advance() {
            name
        } else {
            return Err(ParseError::ExpectedSyntax("Expected enum name".to_string()));
        };

        if !self.match_token(&Token::Punctuation(Punctuation::LeftBrace)) {
            return Err(ParseError::ExpectedPunctuation(Punctuation::LeftBrace, "Expected '{' after enum name".to_string()));
        }

        let mut variants = Vec::new();

        while self.peek() != Some(&Token::Punctuation(Punctuation::RightBrace)) {
            if self.peek() == Some(&Token::Punctuation(Punctuation::Newline)) {
                self.advance();
                continue;
            }

            let variant_name = if let Some(Token::Identifier(name)) = self.advance() {
                name
            } else {
                return Err(ParseError::ExpectedSyntax("Expected variant name".to_string()));
            };

            let variant_data = if self.match_token(&Token::Punctuation(Punctuation::LeftParen)) {
                let mut fields = Vec::new();

                if !self.match_token(&Token::Punctuation(Punctuation::RightParen)) {
                    if let Some(Token::Identifier(type_name)) = self.advance() {
                        fields.push(Expr::Identifier(type_name));

                        while self.match_token(&Token::Operator(Operator::Comma)) {
                            if self.peek() == Some(&Token::Punctuation(Punctuation::RightParen)) {
                                break;
                            }

                            if let Some(Token::Identifier(type_name)) = self.advance() {
                                fields.push(Expr::Identifier(type_name));
                            } else {
                                return Err(ParseError::ExpectedSyntax("Expected field type".to_string()));
                            }
                        }
                    }

                    if !self.match_token(&Token::Punctuation(Punctuation::RightParen)) {
                        return Err(ParseError::ExpectedPunctuation(Punctuation::RightParen, "Expected ')' after variant fields".to_string()));
                    }
                }

                Some(EnumVariant::Tuple(fields))
            } else if self.match_token(&Token::Punctuation(Punctuation::LeftBrace)) {
                let mut fields = Vec::new();

                while let Some(Token::Identifier(field_name)) = self.advance() {
                    if !self.match_token(&Token::Operator(Operator::Colon)) {
                        return Err(ParseError::ExpectedOperator(Operator::Colon, "Expected ':' after field name".to_string()));
                    }

                    let field_type = if let Some(Token::Identifier(type_name)) = self.advance() {
                        type_name
                    } else {
                        return Err(ParseError::ExpectedSyntax("Expected field type".to_string()));
                    };

                    fields.push((Expr::Identifier(field_name), Expr::Identifier(field_type)));

                    if !self.match_token(&Token::Operator(Operator::Comma)) {
                        break;
                    }
                }

                if !self.match_token(&Token::Punctuation(Punctuation::RightBrace)) {
                    return Err(ParseError::ExpectedPunctuation(Punctuation::RightBrace, "Expected '}' after struct variant fields".to_string()));
                }

                Some(EnumVariant::Struct(fields))
            } else if self.match_token(&Token::Operator(Operator::Equal)) {
                let value = self.parse_expression()?;
                Some(EnumVariant::Discriminant(value))
            } else {
                None
            };

            variants.push((Expr::Identifier(variant_name), variant_data));

            if !self.match_token(&Token::Operator(Operator::Comma)) {
                if self.peek() != Some(&Token::Punctuation(Punctuation::RightBrace)) {
                    return Err(ParseError::ExpectedOperator(Operator::Comma, "Expected ',' after enum variant".to_string()));
                }
            }
        }

        // Expect closing brace
        if !self.match_token(&Token::Punctuation(Punctuation::RightBrace)) {
            return Err(ParseError::ExpectedPunctuation(Punctuation::RightBrace, "Expected '}' after enum variants".to_string()));
        }

        Ok(Stmt::EnumDef(Expr::Identifier(name), variants))
    }
}
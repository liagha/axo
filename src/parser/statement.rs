#![allow(dead_code)]

use crate::{
    errors::ParseError,
    parser::{Parser, parser::{Expr,Stmt}, expression::Expression},
};
use crate::lexer::{KeywordKind, OperatorKind, PunctuationKind, TokenKind, Token};

#[derive(Debug, Clone)]
pub enum EnumVariant {
    Tuple(Vec<Expr>),
    Struct(Vec<Expr>),
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
        if let Some(token) = self.peek() {
            match &token.kind {
                TokenKind::Keyword(kw) => match kw {
                    KeywordKind::If => self.parse_if_statement(),
                    KeywordKind::While => self.parse_while_statement(),
                    KeywordKind::For => self.parse_for_statement(),
                    KeywordKind::Fn => self.parse_function_declaration(),
                    KeywordKind::Return => self.parse_return_statement(),
                    KeywordKind::Break => self.parse_break_statement(),
                    KeywordKind::Continue => self.parse_continue_statement(),
                    KeywordKind::Let => self.parse_let_statement(),
                    KeywordKind::Struct => self.parse_struct_definition(),
                    KeywordKind::Enum => self.parse_enum_definition(),
                    _ => Err(ParseError::UnknownStatement),
                },
                TokenKind::Punctuation(PunctuationKind::LeftBrace) => {
                    self.advance();

                    self.parse_block()
                },
                _ => {
                    let left = self.parse_expression()?;

                    if let Some(token) = self.peek() {
                        if token.kind == TokenKind::Operator(OperatorKind::Equal) {
                            self.advance();
                            let right_stmt = self.parse_statement()?;

                            Ok(Stmt::Assignment(left, Box::new(right_stmt)))
                        } else if OperatorKind::is_compound_token(&token.kind) {
                            let operator = TokenKind::get_operator(&token.kind).unwrap();

                            self.advance();

                            let right_stmt = self.parse_statement()?;

                            Ok(Stmt::CompoundAssignment(left, operator.decompound(), Box::new(right_stmt)))
                        } else {
                            self.advance();

                            Ok(Stmt::Expression(left))
                        }
                    } else {
                        Err(ParseError::UnexpectedEOF)
                    }
                }
            }
        } else {
            Err(ParseError::UnexpectedEOF)
        }
    }

    fn parse_let_statement(&mut self) -> Result<Stmt, ParseError> {
        self.advance();

        let identifier = self.parse_expression()?;

        if let Some(token) = self.peek() {
            if token.kind == TokenKind::Operator(OperatorKind::Equal) {
                self.advance();

                let value_stmt = self.parse_statement()?;

                Ok(Stmt::Definition(identifier, Some(Box::new(value_stmt))))
            } else {
                if !self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon)) {
                    return Err(ParseError::ExpectedPunctuation(PunctuationKind::Semicolon, "after variable declaration".to_string()));
                }

                Ok(Stmt::Definition(identifier, None))
            }
        } else {
            Err(ParseError::UnexpectedEOF)
        }
    }

    fn parse_block(&mut self) -> Result<Stmt, ParseError> {
        let mut statements = Vec::new();

        while let Some(token) = self.peek() {
            match token.kind {
                // Skip newlines
                TokenKind::Punctuation(PunctuationKind::Newline) => {
                    self.advance();
                    continue;
                }

                // End of block
                TokenKind::Punctuation(PunctuationKind::RightBrace) => {
                    self.advance();
                    return Ok(Stmt::Block(statements));
                }

                // Parse a regular statement
                _ => {
                    let stmt = self.parse_statement()?;
                    statements.push(stmt);
                }
            }
        }

        // If we reach here, it means we encountered an unexpected EOF
        Err(ParseError::UnexpectedEOF)
    }
    fn parse_function_declaration(&mut self) -> Result<Stmt, ParseError> {
        self.advance();

        let name = if let Some(token) = self.advance() {
            if let TokenKind::Identifier(name) = token.kind {
                name
            } else {
                return Err(ParseError::ExpectedSyntax("function name".to_string()));
            }
        } else {
            return Err(ParseError::UnexpectedEOF);
        };

        if !self.match_token(&TokenKind::Punctuation(PunctuationKind::LeftParen)) {
            return Err(ParseError::ExpectedPunctuation(PunctuationKind::LeftParen, "after function name".to_string()));
        }

        let mut parameters = Vec::new();

        if !self.match_token(&TokenKind::Punctuation(PunctuationKind::RightParen)) {
            if let Some(token) = self.advance() {
                if let TokenKind::Identifier(param) = token.kind {
                    parameters.push(Expr::Identifier(param));

                    while self.match_token(&TokenKind::Operator(OperatorKind::Comma)) {
                        if let Some(token) = self.advance() {
                            if let TokenKind::Identifier(param) = token.kind {
                                parameters.push(Expr::Identifier(param));
                            } else {
                                return Err(ParseError::ExpectedSyntax("parameter name".to_string()));
                            }
                        } else {
                            return Err(ParseError::UnexpectedEOF);
                        }
                    }

                    if !self.match_token(&TokenKind::Punctuation(PunctuationKind::RightParen)) {
                        return Err(ParseError::ExpectedPunctuation(PunctuationKind::RightParen, "after parameters".to_string()));
                    }
                } else {
                    return Err(ParseError::ExpectedSyntax("parameter name".to_string()));
                }
            } else {
                return Err(ParseError::UnexpectedEOF);
            }
        }

        let body = if !self.match_token(&TokenKind::Punctuation(PunctuationKind::LeftBrace)) {
            self.parse_statement()?
        } else {
            self.parse_block()?
        };

        Ok(Stmt::Function(Expr::Identifier(name), parameters, Box::new(body)))
    }

    fn parse_if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.advance();

        let condition = self.parse_condition()?;

        let then_branch = if self.match_token(&TokenKind::Punctuation(PunctuationKind::LeftBrace)) {
            Box::new(self.parse_block()?)
        } else {
            Box::new(self.parse_statement()?)
        };

        let else_branch = if self.match_token(&TokenKind::Keyword(KeywordKind::Else)) {
            if let Some(token) = self.peek() {
                if token.kind == TokenKind::Keyword(KeywordKind::If) {
                    Some(Box::new(self.parse_if_statement()?))
                } else if self.match_token(&TokenKind::Punctuation(PunctuationKind::LeftBrace)) {
                    Some(Box::new(self.parse_block()?))
                } else {
                    Some(Box::new(self.parse_statement()?))
                }
            } else {
                return Err(ParseError::UnexpectedEOF);
            }
        } else {
            None
        };

        Ok(Stmt::If(condition, then_branch, else_branch))
    }

    fn parse_while_statement(&mut self) -> Result<Stmt, ParseError> {
        self.advance();

        let condition = self.parse_condition()?;

        let body = if self.match_token(&TokenKind::Punctuation(PunctuationKind::LeftBrace)) {
            Box::new(self.parse_block()?)
        } else {
            Box::new(self.parse_statement()?)
        };

        Ok(Stmt::While(condition, body))
    }

    fn parse_for_statement(&mut self) -> Result<Stmt, ParseError> {
        self.advance();

        let has_paren = self.match_token(&TokenKind::Punctuation(PunctuationKind::LeftParen));

        let initializer = if self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon)) {
            None
        } else {
            Some(self.parse_statement()?)
        };

        let condition = if let Some(token) = self.peek() {
            if token.kind == TokenKind::Punctuation(PunctuationKind::Semicolon) {
                Expr::Boolean(true)
            } else {
                self.parse_expression()?
            }
        } else {
            return Err(ParseError::UnexpectedEOF);
        };

        if !self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon)) {
            return Err(ParseError::ExpectedPunctuation(PunctuationKind::Semicolon, "after for condition".to_string()));
        }

        let increment = if let Some(token) = self.peek() {
            if (has_paren && token.kind == TokenKind::Punctuation(PunctuationKind::RightParen)) ||
                (!has_paren && (token.kind == TokenKind::Punctuation(PunctuationKind::LeftBrace) ||
                    token.kind == TokenKind::Punctuation(PunctuationKind::Semicolon))) {
                None
            } else {
                if let Some(peek_token) = self.peek() {
                    if let TokenKind::Identifier(name) = &peek_token.kind {
                        let name = name.clone();
                        self.advance();
                        if let Some(next_token) = self.peek() {
                            if next_token.kind == TokenKind::Operator(OperatorKind::PlusEqual) {
                                self.advance();
                                let value = self.parse_expression()?;
                                Some(Stmt::CompoundAssignment(Expr::Identifier(name), OperatorKind::PlusEqual, Stmt::Expression(value).into()))
                            } else {
                                self.current -= 1;
                                Some(Stmt::Expression(self.parse_expression()?))
                            }
                        } else {
                            return Err(ParseError::UnexpectedEOF);
                        }
                    } else {
                        Some(Stmt::Expression(self.parse_expression()?))
                    }
                } else {
                    return Err(ParseError::UnexpectedEOF);
                }
            }
        } else {
            return Err(ParseError::UnexpectedEOF);
        };

        if has_paren {
            if !self.match_token(&TokenKind::Punctuation(PunctuationKind::RightParen)) {
                return Err(ParseError::ExpectedPunctuation(PunctuationKind::RightParen, "after for clauses".to_string()));
            }
        }

        let body = if self.match_token(&TokenKind::Punctuation(PunctuationKind::LeftBrace)) {
            self.parse_block()?
        } else {
            self.parse_statement()?
        };

        Ok(Stmt::For(initializer.map(Box::new), condition, increment.map(Box::new), Box::new(body)))
    }

    fn parse_return_statement(&mut self) -> Result<Stmt, ParseError> {
        self.advance();

        if self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon)) {
            return Ok(Stmt::Return(None));
        }

        let value = self.parse_expression()?;

        if !self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon)) {
            return Err(ParseError::ExpectedPunctuation(PunctuationKind::Semicolon, "after return value".to_string()));
        }

        Ok(Stmt::Return(Some(value)))
    }

    fn parse_break_statement(&mut self) -> Result<Stmt, ParseError> {
        self.advance();

        if self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon)) {
            return Ok(Stmt::Break(None));
        }

        let value = self.parse_expression()?;

        if !self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon)) {
            return Err(ParseError::ExpectedPunctuation(PunctuationKind::Semicolon, "after break value".to_string()));
        }

        Ok(Stmt::Break(Some(value)))
    }

    fn parse_continue_statement(&mut self) -> Result<Stmt, ParseError> {
        self.advance();

        if !self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon)) {
            return Err(ParseError::ExpectedPunctuation(PunctuationKind::Semicolon, "after 'continue'".to_string()));
        }

        Ok(Stmt::Continue)
    }

    fn parse_condition(&mut self) -> Result<Expr, ParseError> {
        let has_paren = self.match_token(&TokenKind::Punctuation(PunctuationKind::LeftParen));

        let condition = self.parse_expression()?;

        if has_paren {
            if !self.match_token(&TokenKind::Punctuation(PunctuationKind::RightParen)) {
                return Err(ParseError::ExpectedPunctuation(PunctuationKind::RightParen, "after condition".to_string()));
            }
        }

        Ok(condition)
    }

    fn parse_struct_definition(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // Consume the 'struct' keyword

        let name = if let Some(token) = self.advance() {
            if let TokenKind::Identifier(name) = token.kind {
                name
            } else {
                return Err(ParseError::ExpectedSyntax("Expected struct name".to_string()));
            }
        } else {
            return Err(ParseError::UnexpectedEOF);
        };

        if !self.match_token(&TokenKind::Punctuation(PunctuationKind::LeftBrace)) {
            return Err(ParseError::ExpectedPunctuation(PunctuationKind::LeftBrace, "Expected '{' after struct name".to_string()));
        }

        let mut fields = Vec::new();

        loop {
            match self.peek() {
                Some(_) => {
                    let field = self.parse_expression()?;

                    fields.push(field);

                    if !self.match_token(&TokenKind::Operator(OperatorKind::Comma)) {
                        break;
                    }
                },
                None => {
                    return Err(ParseError::UnexpectedEOF);
                }
            }
        }

        if !self.match_token(&TokenKind::Punctuation(PunctuationKind::RightBrace)) {
            return Err(ParseError::ExpectedPunctuation(PunctuationKind::RightBrace, "Expected '}' after struct fields".to_string()));
        }

        Ok(Stmt::StructDef(Expr::Identifier(name), fields))
    }
    fn parse_enum_definition(&mut self) -> Result<Stmt, ParseError> {
        self.advance();

        let name = if let Some(token) = self.advance() {
            if let TokenKind::Identifier(name) = token.kind {
                name
            } else {
                return Err(ParseError::ExpectedSyntax("Expected enum name".to_string()));
            }
        } else {
            return Err(ParseError::UnexpectedEOF);
        };

        if !self.match_token(&TokenKind::Punctuation(PunctuationKind::LeftBrace)) {
            return Err(ParseError::ExpectedPunctuation(PunctuationKind::LeftBrace, "Expected '{' after enum name".to_string()));
        }

        let mut variants = Vec::new();

        while let Some(token) = self.peek() {
            if token.kind == TokenKind::Punctuation(PunctuationKind::RightBrace) {
                break;
            }

            if token.kind == TokenKind::Punctuation(PunctuationKind::Newline) {
                self.advance();
                continue;
            }

            let variant_name = if let Some(token) = self.advance() {
                if let TokenKind::Identifier(name) = token.kind {
                    name
                } else {
                    return Err(ParseError::ExpectedSyntax("Expected variant name".to_string()));
                }
            } else {
                return Err(ParseError::UnexpectedEOF);
            };

            let variant_data = if self.match_token(&TokenKind::Punctuation(PunctuationKind::LeftParen)) {
                let mut fields = Vec::new();

                if !self.match_token(&TokenKind::Punctuation(PunctuationKind::RightParen)) {
                    if let Some(token) = self.advance() {
                        if let TokenKind::Identifier(type_name) = token.kind {
                            fields.push(Expr::Identifier(type_name));

                            while self.match_token(&TokenKind::Operator(OperatorKind::Comma)) {
                                if let Some(token) = self.peek() {
                                    if token.kind == TokenKind::Punctuation(PunctuationKind::RightParen) {
                                        break;
                                    }
                                } else {
                                    return Err(ParseError::UnexpectedEOF);
                                }

                                if let Some(token) = self.advance() {
                                    if let TokenKind::Identifier(type_name) = token.kind {
                                        fields.push(Expr::Identifier(type_name));
                                    } else {
                                        return Err(ParseError::ExpectedSyntax("Expected field type".to_string()));
                                    }
                                } else {
                                    return Err(ParseError::UnexpectedEOF);
                                }
                            }
                        } else {
                            return Err(ParseError::ExpectedSyntax("Expected field type".to_string()));
                        }
                    } else {
                        return Err(ParseError::UnexpectedEOF);
                    }

                    if !self.match_token(&TokenKind::Punctuation(PunctuationKind::RightParen)) {
                        return Err(ParseError::ExpectedPunctuation(PunctuationKind::RightParen, "Expected ')' after variant fields".to_string()));
                    }
                }

                Some(EnumVariant::Tuple(fields))
            } else if self.match_token(&TokenKind::Punctuation(PunctuationKind::LeftBrace)) {
                let mut fields = Vec::new();

                loop {
                    match self.peek() {
                        Some(Token { kind: TokenKind::Identifier(_field_name), .. }) => {
                            let field = self.parse_expression()?;

                            fields.push(field);

                            if !self.match_token(&TokenKind::Operator(OperatorKind::Comma)) {
                                break;
                            }
                        },
                        Some(Token { kind: TokenKind::Punctuation(PunctuationKind::RightBrace), .. }) => {
                            break;
                        },
                        Some(token) => {
                            return Err(ParseError::UnexpectedToken(token.kind.clone(), "field name or '}'".into()));
                        },
                        None => {
                            return Err(ParseError::UnexpectedEOF);
                        }
                    }
                }

                if !self.match_token(&TokenKind::Punctuation(PunctuationKind::RightBrace)) {
                    return Err(ParseError::ExpectedPunctuation(PunctuationKind::RightBrace, "Expected '}' after struct fields".to_string()));
                }

                Some(EnumVariant::Struct(fields))
            } else if self.match_token(&TokenKind::Operator(OperatorKind::Equal)) {
                let value = self.parse_expression()?;
                Some(EnumVariant::Discriminant(value))
            } else {
                None
            };

            variants.push((Expr::Identifier(variant_name), variant_data));

            if !self.match_token(&TokenKind::Operator(OperatorKind::Comma)) {
                if let Some(token) = self.peek() {
                    if token.kind != TokenKind::Punctuation(PunctuationKind::RightBrace) {
                        return Err(ParseError::ExpectedOperator(OperatorKind::Comma, "Expected ',' after enum variant".to_string()));
                    }
                } else {
                    return Err(ParseError::UnexpectedEOF);
                }
            }
        }

        // Expect closing brace
        if !self.match_token(&TokenKind::Punctuation(PunctuationKind::RightBrace)) {
            return Err(ParseError::ExpectedPunctuation(PunctuationKind::RightBrace, "Expected '}' after enum variants".to_string()));
        }

        Ok(Stmt::EnumDef(Expr::Identifier(name), variants))
    }
}
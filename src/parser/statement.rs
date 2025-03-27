#![allow(dead_code)]

use crate::parser::{expression::Expression, Parser};
use crate::lexer::{KeywordKind, OperatorKind, PunctuationKind, Token, TokenKind};
use crate::parser::error::{ParseError, SyntaxPosition, SyntaxType};
use crate::parser::expression::Expr;

#[derive(Debug, Clone)]
pub enum EnumVariant {
    Tuple(Vec<Expr>),
    Struct(Vec<Expr>),
    Discriminant(Expr),
}

pub trait Statement {
    fn parse_statement(&mut self) -> Result<Expr, ParseError>;
    fn parse_let_statement(&mut self) -> Result<Expr, ParseError>;
    fn parse_block(&mut self) -> Result<Expr, ParseError>;
    fn parse_function_declaration(&mut self) -> Result<Expr, ParseError>;
    fn parse_if_statement(&mut self) -> Result<Expr, ParseError>;
    fn parse_while_statement(&mut self) -> Result<Expr, ParseError>;
    fn parse_for_statement(&mut self) -> Result<Expr, ParseError>;
    fn parse_return_statement(&mut self) -> Result<Expr, ParseError>;
    fn parse_break_statement(&mut self) -> Result<Expr, ParseError>;
    fn parse_continue_statement(&mut self) -> Result<Expr, ParseError>;
    fn parse_struct_definition(&mut self) -> Result<Expr, ParseError>;
    fn parse_enum_definition(&mut self) -> Result<Expr, ParseError>;
}

impl Statement for Parser {
    fn parse_statement(&mut self) -> Result<Expr, ParseError> {
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
                    KeywordKind::Impl => unimplemented!(),
                    KeywordKind::Trait => unimplemented!(),
                    KeywordKind::Match => unimplemented!(),
                    KeywordKind::Else => unimplemented!(),
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

                            Ok(Expr::Assignment(left.into(), Box::new(right_stmt)))
                        } else if OperatorKind::is_compound_token(&token.kind) {
                            let operator = TokenKind::get_operator(&token.kind).unwrap();

                            self.advance();

                            let right_stmt = self.parse_statement()?;

                            let operation = Expr::Binary(left.clone().into(), operator.decompound(), Box::new(right_stmt));

                            Ok(Expr::Assignment(left.into(), operation.into()))
                        } else {
                            self.advance();

                            Ok(left)
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

    fn parse_let_statement(&mut self) -> Result<Expr, ParseError> {
        self.advance();

        let identifier = self.parse_expression()?;

        if let Some(token) = self.peek() {
            if token.kind == TokenKind::Operator(OperatorKind::Equal) {
                self.advance();

                let value_stmt = self.parse_statement()?;

                Ok(Expr::Definition(identifier.into(), Some(Box::new(value_stmt))))
            } else {
                if !self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon)) {
                    let err = ParseError::ExpectedToken(TokenKind::Punctuation(PunctuationKind::Semicolon), SyntaxPosition::After, SyntaxType::VariableDeclaration);

                    return Err(err);
                }

                Ok(Expr::Definition(identifier.into(), None))
            }
        } else {
            Err(ParseError::UnexpectedEOF)
        }
    }

    fn parse_block(&mut self) -> Result<Expr, ParseError> {
        let mut statements = Vec::new();

        while let Some(token) = self.peek() {
            match token.kind {
                TokenKind::Punctuation(PunctuationKind::RightBrace) => {
                    self.advance();
                    return Ok(Expr::Block(statements));
                }

                _ => {
                    let stmt = self.parse_statement()?;
                    statements.push(stmt.into());
                }
            }
        }

        Err(ParseError::UnexpectedEOF)
    }
    fn parse_function_declaration(&mut self) -> Result<Expr, ParseError> {
        self.advance();

        let name = if let Some(token) = self.advance() {
            if let TokenKind::Identifier(name) = token.kind {
                name
            } else {
                return Err(ParseError::ExpectedSyntax(SyntaxType::FunctionName));
            }
        } else {
            return Err(ParseError::UnexpectedEOF);
        };

        if !self.match_token(&TokenKind::Punctuation(PunctuationKind::LeftParen)) {
            let err = ParseError::ExpectedToken(TokenKind::Punctuation(PunctuationKind::LeftParen), SyntaxPosition::After, SyntaxType::FunctionName);

            return Err(err);
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
                                return Err(ParseError::ExpectedSyntax(SyntaxType::ParameterName));
                            }
                        } else {
                            return Err(ParseError::UnexpectedEOF);
                        }
                    }

                    if !self.match_token(&TokenKind::Punctuation(PunctuationKind::RightParen)) {
                        let err = ParseError::ExpectedToken(TokenKind::Punctuation(PunctuationKind::RightParen), SyntaxPosition::After, SyntaxType::ParameterName);

                        return Err(err);
                    }
                } else {
                    return Err(ParseError::ExpectedSyntax(SyntaxType::ParameterName));
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

        Ok(Expr::Function(Expr::Identifier(name).into(), parameters, Box::new(body)))
    }

    fn parse_if_statement(&mut self) -> Result<Expr, ParseError> {
        self.advance();

        let condition = self.parse_expression()?;

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

        Ok(Expr::If(condition.into(), then_branch.into(), else_branch.into()))
    }

    fn parse_while_statement(&mut self) -> Result<Expr, ParseError> {
        self.advance();

        let condition = self.parse_expression()?;

        let body = if self.match_token(&TokenKind::Punctuation(PunctuationKind::LeftBrace)) {
            Box::new(self.parse_block()?)
        } else {
            Box::new(self.parse_statement()?)
        };

        Ok(Expr::While(condition.into(), body.into()))
    }

    fn parse_for_statement(&mut self) -> Result<Expr, ParseError> {
        self.advance();

        let clause = self.parse_expression()?;

        let body = if self.match_token(&TokenKind::Punctuation(PunctuationKind::LeftBrace)) {
            self.parse_block()?
        } else {
            self.parse_statement()?
        };

        Ok(Expr::For(clause.into(), Box::new(body)))
    }

    fn parse_return_statement(&mut self) -> Result<Expr, ParseError> {
        self.advance();

        if self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon)) {
            return Ok(Expr::Return(None));
        }

        let value = self.parse_expression()?;

        if !self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon)) {
            let err = ParseError::ExpectedToken(TokenKind::Punctuation(PunctuationKind::Semicolon), SyntaxPosition::After, SyntaxType::ReturnValue);

            return Err(err);
        }

        Ok(Expr::Return(Some(value.into())))
    }

    fn parse_break_statement(&mut self) -> Result<Expr, ParseError> {
        self.advance();

        if self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon)) {
            return Ok(Expr::Break(None));
        }

        let value = self.parse_expression()?;

        if !self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon)) {
            let err = ParseError::ExpectedToken(TokenKind::Punctuation(PunctuationKind::Semicolon), SyntaxPosition::After, SyntaxType::BreakValue);

            return Err(err);
        }

        Ok(Expr::Break(Some(value.into())))
    }

    fn parse_continue_statement(&mut self) -> Result<Expr, ParseError> {
        self.advance();

        if !self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon)) {
            let err = ParseError::ExpectedToken(TokenKind::Punctuation(PunctuationKind::Semicolon), SyntaxPosition::After, SyntaxType::Continue);

            return Err(err);
        }

        Ok(Expr::Continue)
    }

    fn parse_struct_definition(&mut self) -> Result<Expr, ParseError> {
        self.advance();

        let name = if let Some(token) = self.advance() {
            if let TokenKind::Identifier(name) = token.kind {
                name
            } else {
                return Err(ParseError::ExpectedSyntax(SyntaxType::StructName));
            }
        } else {
            return Err(ParseError::UnexpectedEOF);
        };

        if !self.match_token(&TokenKind::Punctuation(PunctuationKind::LeftBrace)) {
            let err = ParseError::ExpectedToken(TokenKind::Punctuation(PunctuationKind::LeftBrace), SyntaxPosition::After, SyntaxType::StructName);

            return Err(err);
        }

        let mut fields = Vec::new();

        loop {
            match self.peek() {
                Some(_) => {
                    let field = self.parse_expression()?;

                    fields.push(field.into());

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
            let err = ParseError::ExpectedToken(TokenKind::Punctuation(PunctuationKind::RightBrace), SyntaxPosition::After, SyntaxType::StructFields);

            return Err(err);
        }

        Ok(Expr::StructDef(Expr::Identifier(name).into(), fields))
    }
    fn parse_enum_definition(&mut self) -> Result<Expr, ParseError> {
        self.advance();

        let name = if let Some(token) = self.advance() {
            if let TokenKind::Identifier(name) = token.kind {
                name
            } else {
                return Err(ParseError::ExpectedSyntax(SyntaxType::EnumName));
            }
        } else {
            return Err(ParseError::UnexpectedEOF);
        };

        if !self.match_token(&TokenKind::Punctuation(PunctuationKind::LeftBrace)) {
            let err = ParseError::ExpectedToken(TokenKind::Punctuation(PunctuationKind::LeftBrace), SyntaxPosition::After, SyntaxType::EnumName);

            return Err(err);
        }

        let mut variants = Vec::new();

        while let Some(token) = self.peek() {
            if token.kind == TokenKind::Punctuation(PunctuationKind::RightBrace) {
                break;
            }

            let variant_name = if let Some(token) = self.advance() {
                if let TokenKind::Identifier(name) = token.kind {
                    name
                } else {
                    return Err(ParseError::ExpectedSyntax(SyntaxType::EnumVariantName));
                }
            } else {
                return Err(ParseError::UnexpectedEOF);
            };

            let variant_data = if self.match_token(&TokenKind::Punctuation(PunctuationKind::LeftParen)) {
                let mut fields = Vec::new();

                if !self.match_token(&TokenKind::Punctuation(PunctuationKind::RightParen)) {
                    if let Some(token) = self.advance() {
                        if let TokenKind::Identifier(type_name) = token.kind {
                            fields.push(Expr::Identifier(type_name).into());

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
                                        fields.push(Expr::Identifier(type_name).into());
                                    } else {
                                        return Err(ParseError::ExpectedSyntax(SyntaxType::FieldType));
                                    }
                                } else {
                                    return Err(ParseError::UnexpectedEOF);
                                }
                            }
                        } else {
                            return Err(ParseError::ExpectedSyntax(SyntaxType::FieldType));
                        }
                    } else {
                        return Err(ParseError::UnexpectedEOF);
                    }

                    if !self.match_token(&TokenKind::Punctuation(PunctuationKind::RightParen)) {
                        let err = ParseError::ExpectedToken(TokenKind::Punctuation(PunctuationKind::RightParen), SyntaxPosition::After, SyntaxType::EnumVariants);

                        return Err(err);
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
                        Some(_) => {
                            let err = ParseError::ExpectedToken(TokenKind::Punctuation(PunctuationKind::RightBrace), SyntaxPosition::After, SyntaxType::EnumVariants);

                            return Err(err);
                        },
                        None => {
                            return Err(ParseError::UnexpectedEOF);
                        }
                    }
                }

                if !self.match_token(&TokenKind::Punctuation(PunctuationKind::RightBrace)) {
                    let err = ParseError::ExpectedToken(TokenKind::Punctuation(PunctuationKind::RightBrace), SyntaxPosition::After, SyntaxType::StructFields);

                    return Err(err);
                }

                Some(EnumVariant::Struct(fields))
            } else if self.match_token(&TokenKind::Operator(OperatorKind::Equal)) {
                let value = self.parse_expression()?;
                Some(EnumVariant::Discriminant(value))
            } else {
                None
            };

            variants.push((Expr::Identifier(variant_name).into(), variant_data));

            if !self.match_token(&TokenKind::Operator(OperatorKind::Comma)) {
                if let Some(token) = self.peek() {
                    if token.kind != TokenKind::Punctuation(PunctuationKind::RightBrace) {
                        let err = ParseError::ExpectedToken(TokenKind::Operator(OperatorKind::Comma), SyntaxPosition::After, SyntaxType::EnumVariant);

                        return Err(err);
                    }
                } else {
                    return Err(ParseError::UnexpectedEOF);
                }
            }
        }

        if !self.match_token(&TokenKind::Punctuation(PunctuationKind::RightBrace)) {
            let err = ParseError::ExpectedToken(TokenKind::Punctuation(PunctuationKind::RightBrace), SyntaxPosition::After, SyntaxType::EnumVariants);

            return Err(err);
        }

        Ok(Expr::Enum(Expr::Identifier(name).into(), variants))
    }
}

use crate::axo_lexer::{KeywordKind, OperatorKind, PunctuationKind, Span, Token, TokenKind};
use crate::axo_parser::error::ErrorKind;
use crate::axo_parser::state::{Context, ContextKind, Position, SyntaxRole};
use crate::axo_parser::{Composite, ControlFlow, Error, Expr, ExprKind, Parser};
use crate::axo_parser::delimiter::Delimiter;
use crate::axo_parser::expression::Expression;
use crate::axo_parser::item::Item;

pub trait Primary {
    fn parse_atom(&mut self) -> Expr;
    fn parse_leaf(&mut self) -> Result<Expr, Error>;
    fn parse_primary(&mut self) -> Result<Expr, Error>;
    fn parse_unary(&mut self, primary: fn(&mut Parser) -> Result<Expr, Error>) -> Result<Expr, Error>;
    fn parse_binary(&mut self, primary: fn(&mut Parser) -> Result<Expr, Error>, min_precedence: u8) -> Result<Expr, Error>;
    fn parse_statement(&mut self) -> Result<Expr, Error>;
}

impl Primary for Parser {
    fn parse_atom(&mut self) -> Expr {
        let token = self.next().unwrap();
        let Token { kind, span } = token.clone();

        let expr = match kind {
            TokenKind::Identifier(ident) => Expr {
                kind: ExprKind::Identifier(ident),
                span,
            },
            TokenKind::Float(_)
            | TokenKind::Integer(_)
            | TokenKind::Boolean(_)
            | TokenKind::Str(_)
            | TokenKind::Operator(_)
            | TokenKind::Char(_)
            | TokenKind::Punctuation(_)
            | TokenKind::Keyword(_)
            | TokenKind::Comment(_)
            | TokenKind::EOF => Expr {
                kind: ExprKind::Literal(token),
                span,
            },
        };

        expr
    }

    fn parse_leaf(&mut self) -> Result<Expr, Error> {
        if let Some(token) = self.peek().cloned() {
            let Token { kind, span } = token.clone();

            match kind {
                TokenKind::Keyword(ref kw) => match kw {
                    KeywordKind::If => self.parse_conditional(),
                    KeywordKind::While => self.parse_while(),
                    KeywordKind::For => self.parse_for(),
                    KeywordKind::Fn => self.parse_function(),
                    KeywordKind::Macro => self.parse_macro(),
                    KeywordKind::Use => self.parse_use(),
                    KeywordKind::Return => self.parse_return(),
                    KeywordKind::Break => self.parse_break(),
                    KeywordKind::Continue => self.parse_continue(),
                    KeywordKind::Let => self.parse_let(),
                    KeywordKind::Struct => self.parse_struct(),
                    KeywordKind::Enum => self.parse_enum(),
                    KeywordKind::Impl => self.parse_impl(),
                    KeywordKind::Trait => self.parse_trait(),
                    KeywordKind::Match => self.parse_match(),
                    KeywordKind::Else => Err(Error::new(ErrorKind::ElseWithoutConditional, span)),
                    _ => Err(Error::new(ErrorKind::UnimplementedToken(kind), span)),
                },
                TokenKind::Identifier(_)
                | TokenKind::Str(_)
                | TokenKind::Char(_)
                | TokenKind::Boolean(_)
                | TokenKind::Float(_)
                | TokenKind::Integer(_)
                | TokenKind::Operator(_) => {
                    let mut expr = self.parse_atom();

                    while let Some(token) = self.peek() {
                        match &token.kind {
                            TokenKind::Punctuation(PunctuationKind::LeftBrace) => {
                                expr = self.parse_structure(expr.clone())?;
                            }
                            TokenKind::Punctuation(PunctuationKind::LeftBracket) => {
                                expr = self.parse_index(expr)?
                            }
                            TokenKind::Punctuation(PunctuationKind::LeftParen) => {
                                expr = self.parse_invoke(expr)?;
                            }
                            _ => break,
                        }
                    }

                    Ok(expr)
                }
                _ => self.parse_primary()
            }
        } else {
            Err(Error::new(ErrorKind::UnexpectedEndOfFile, self.full_span()))
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, Error> {
        if let Some(token) = self.peek().cloned() {
            let Token { kind, span } = token.clone();

            match kind {
                TokenKind::Punctuation(PunctuationKind::LeftBrace) => self.parse_braced(),
                TokenKind::Punctuation(PunctuationKind::LeftBracket) => self.parse_bracketed(),
                TokenKind::Punctuation(PunctuationKind::LeftParen) => self.parse_parenthesized(),
                TokenKind::Operator(OperatorKind::Pipe) => self.parse_closure(),
                TokenKind::Identifier(_)
                | TokenKind::Str(_)
                | TokenKind::Char(_)
                | TokenKind::Boolean(_)
                | TokenKind::Float(_)
                | TokenKind::Integer(_)
                | TokenKind::Operator(_) => {
                    let mut expr = self.parse_atom();

                    while let Some(token) = self.peek() {
                        match &token.kind {
                            TokenKind::Punctuation(PunctuationKind::LeftBracket) => {
                                expr = self.parse_index(expr)?
                            }
                            TokenKind::Punctuation(PunctuationKind::LeftParen) => {
                                expr = self.parse_invoke(expr)?;
                            }
                            _ => break,
                        }
                    }

                    Ok(expr)
                }

                TokenKind::EOF => Err(Error::new(ErrorKind::UnexpectedEndOfFile, self.full_span())),
                kind => Err(Error::new(ErrorKind::UnexpectedToken(kind), span)),
            }
        } else {
            Err(Error::new(ErrorKind::UnexpectedEndOfFile, self.full_span()))
        }
    }
    fn parse_unary(&mut self, primary: fn(&mut Parser) -> Result<Expr, Error> ) -> Result<Expr, Error> {
        if let Some(Token {
                        kind: TokenKind::Operator(op),
                        span: Span { start, .. },
                    }) = self.peek().cloned()
        {
            if op.is_prefix() {
                let op = self.next().unwrap();

                let unary = self.parse_unary(primary)?;
                let end = unary.span.end;

                let span = self.span(start, end);

                let kind = ExprKind::Unary(op, unary.into());

                let expr = Expr { kind, span };

                return Ok(expr);
            }
        }

        let mut expr = primary(self)?;

        while let Some(Token {
                           kind: TokenKind::Operator(op),
                           span: Span { end, .. },
                       }) = self.peek().cloned()
        {
            if op.is_postfix() {
                let op = self.next().unwrap();
                let span = self.span(expr.span.start, end);

                let kind = ExprKind::Unary(op, expr.into());
                expr = Expr { kind, span };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_binary(&mut self, primary: fn(&mut Parser) -> Result<Expr, Error>, min_precedence: u8) -> Result<Expr, Error> {
        let mut left = self.parse_unary(primary)?;

        while let Some(Token { kind: TokenKind::Operator(op), .. }) = self.peek().cloned() {
            let precedence = op.precedence();

            if precedence < min_precedence {
                break;
            }

            let op_token = self.next().unwrap();

            let right = self.parse_binary(primary, precedence + 1)?;

            let start = left.span.start;
            let end = right.span.end;
            let span = self.span(start, end);

            let kind = ExprKind::Binary(left.into(), op_token, right.into());
            left = Expr { kind, span }.transform();
        }

        Ok(left)
    }

    fn parse_statement(&mut self) -> Result<Expr, Error> {
        let result = if let Some(_token) = self.peek().cloned() {
            let expr = self.parse_complex()?;

            if let Some(Token {
                            kind: TokenKind::Punctuation(PunctuationKind::Semicolon),
                            ..
                        }) = self.peek()
            {
                self.next();
                Ok(expr)
            } else {
                Ok(expr)
            }
        } else {
            Err(Error::new(ErrorKind::UnexpectedEndOfFile, self.full_span()))
        };

        result
    }
}

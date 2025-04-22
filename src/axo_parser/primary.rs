use {
    crate::{
        axo_lexer::{
            Token, TokenKind,
            KeywordKind, OperatorKind, PunctuationKind,
        },
        axo_parser::{
            delimiter::Delimiter,
            expression::Expression,
            item::ItemParser,
            error::ErrorKind,
            Expr, ExprKind,
            ParseError, Parser,
            Composite, ControlFlow
        },
        axo_span::Span,
    }
};

pub trait Primary {
    fn parse_atom(&mut self) -> Expr;
    fn parse_leaf(&mut self) -> Expr;
    fn parse_primary(&mut self) -> Expr;
    fn parse_unary(&mut self, primary: fn(&mut Parser) -> Expr) -> Expr;
    fn parse_binary(&mut self, primary: fn(&mut Parser) -> Expr, min_precedence: u8) -> Expr;
    fn parse_statement(&mut self) -> Expr;
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
            | TokenKind::EOF => {
                let Token { kind, span } = token;

                Expr {
                    kind: ExprKind::Literal(kind),
                    span,
                }
            },
        };

        expr
    }

    fn parse_leaf(&mut self) -> Expr {
        if let Some(token) = self.peek().cloned() {
            let Token { kind, span } = token.clone();

            match kind {
                TokenKind::Keyword(ref kw) => match kw {
                    KeywordKind::Var | KeywordKind::Const => self.parse_variable(),
                    KeywordKind::If => self.parse_conditional(),
                    KeywordKind::Loop => self.parse_loop(),
                    KeywordKind::While => self.parse_while(),
                    KeywordKind::For => self.parse_for(),
                    KeywordKind::Fn => self.parse_function(),
                    KeywordKind::Macro => self.parse_macro(),
                    KeywordKind::Use => self.parse_use(),
                    KeywordKind::Return => self.parse_return(),
                    KeywordKind::Break => self.parse_break(),
                    KeywordKind::Continue => self.parse_continue(),
                    KeywordKind::Struct => self.parse_struct(),
                    KeywordKind::Enum => self.parse_enum(),
                    KeywordKind::Impl => self.parse_impl(),
                    KeywordKind::Trait => self.parse_trait(),
                    KeywordKind::Match => self.parse_match(),
                    KeywordKind::Else => {
                        self.next();

                        self.error(&ParseError::new(ErrorKind::DanglingElse, span))
                    },
                    _ => {
                        self.next();

                        self.error(&ParseError::new(ErrorKind::UnimplementedToken(kind), span))
                    },
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
                                expr = self.parse_constructor(expr.clone());
                            }
                            TokenKind::Punctuation(PunctuationKind::LeftBracket) => {
                                expr = self.parse_index(expr)
                            }
                            TokenKind::Punctuation(PunctuationKind::LeftParen) => {
                                expr = self.parse_invoke(expr);
                            }
                            _ => break,
                        }
                    }

                    expr
                }
                _ => self.parse_primary()
            }
        } else {
            self.error(&ParseError::new(ErrorKind::UnexpectedEndOfFile, self.full_span()))
        }
    }

    fn parse_primary(&mut self) -> Expr {
        if let Some(token) = self.peek().cloned() {
            let Token { kind, span } = token.clone();

            match kind {
                TokenKind::Punctuation(PunctuationKind::LeftBrace) => self.parse_braced(),
                TokenKind::Punctuation(PunctuationKind::LeftBracket) => self.parse_collection(),
                TokenKind::Punctuation(PunctuationKind::LeftParen) => self.parse_group(),
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
                                expr = self.parse_index(expr);
                            }
                            TokenKind::Punctuation(PunctuationKind::LeftParen) => {
                                expr = self.parse_invoke(expr);
                            }
                            _ => break,
                        }
                    }

                    expr
                }

                TokenKind::EOF => {
                    self.error(&ParseError::new(ErrorKind::UnexpectedEndOfFile, self.full_span()))
                },
                kind => {
                    self.error(&ParseError::new(ErrorKind::UnexpectedToken(kind), span))
                },
            }
        } else {
            self.error(&ParseError::new(ErrorKind::UnexpectedEndOfFile, self.full_span()))
        }
    }
    fn parse_unary(&mut self, primary: fn(&mut Parser) -> Expr ) -> Expr {
        if let Some(Token {
                        kind: TokenKind::Operator(op),
                        span: Span { start, .. },
                    }) = self.peek().cloned()
        {
            if op.is_prefix() {
                let operator = self.next().unwrap();

                let unary = self.parse_unary(primary);
                let end = unary.span.end;

                let span = self.span(start, end);

                let kind = ExprKind::Unary {
                    operator,
                    operand: unary.into()
                };

                let expr = Expr { kind, span };

                return expr;
            }
        }

        let mut expr = primary(self);

        while let Some(Token {
                           kind: TokenKind::Operator(op),
                           span: Span { end, .. },
                       }) = self.peek().cloned()
        {
            if op.is_postfix() {
                let operator = self.next().unwrap();
                let span = self.span(expr.span.start, end);

                let kind = ExprKind::Unary {
                    operator,
                    operand: expr.into()
                };

                expr = Expr { kind, span };
            } else {
                break;
            }
        }

        expr
    }

    fn parse_binary(&mut self, primary: fn(&mut Parser) -> Expr, min_precedence: u8) -> Expr {
        let mut left = self.parse_unary(primary);

        while let Some(Token { kind: TokenKind::Operator(op), .. }) = self.peek().cloned() {
            let precedence = op.precedence();

            if let Some(precedence) = precedence {
                if precedence < min_precedence {
                    break;
                }

                let operator = self.next().unwrap();

                let right = self.parse_binary(primary, precedence + 1);

                let start = left.span.start;
                let end = right.span.end;
                let span = self.span(start, end);

                let kind = ExprKind::Binary {
                    left: left.into(),
                    operator,
                    right: right.into()
                };

                left = Expr { kind, span };

                left = left.transform();
            } else {
                break;
            }
        }

        left
    }

    fn parse_statement(&mut self) -> Expr {
        let result = if let Some(_token) = self.peek().cloned() {
            let expr = self.parse_complex();

            if let Some(Token {
                            kind: TokenKind::Punctuation(PunctuationKind::Semicolon),
                            ..
                        }) = self.peek()
            {
                self.next();
                expr
            } else {
                expr
            }
        } else {
            self.error(&ParseError::new(ErrorKind::UnexpectedEndOfFile, self.full_span()))
        };

        result
    }
}

use crate::axo_lexer::{KeywordKind, OperatorKind, PunctuationKind, Span, Token, TokenKind};
use crate::axo_parser::error::{Error, ErrorKind};
use crate::axo_parser::state::{Context, ContextKind, Position, SyntaxRole};
use crate::axo_parser::{Composite, ControlFlow, Declaration, Expr, ExprKind, Parser};

pub trait Primary {
    fn parse_atom(&mut self) -> Expr;
    fn parse_leaf(&mut self) -> Result<Expr, Error>;
    fn parse_primary(&mut self) -> Result<Expr, Error>;
    fn parse_unary(&mut self, primary: fn(&mut Parser) -> Result<Expr, Error>) -> Result<Expr, Error>;
    fn parse_factor(&mut self, primary: fn(&mut Parser) -> Result<Expr, Error>) -> Result<Expr, Error>;
    fn parse_term(&mut self, primary: fn(&mut Parser) -> Result<Expr, Error>) -> Result<Expr, Error>;
    fn parse_basic(&mut self) -> Result<Expr, Error>;
    fn parse_complex(&mut self) -> Result<Expr, Error>;
    fn parse_statement(&mut self) -> Result<Expr, Error>;
    fn parse_array(&mut self) -> Result<Expr, Error>;
    fn parse_tuple(&mut self) -> Result<Expr, Error>;
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
            | TokenKind::Invalid(_)
            | TokenKind::EOF => Expr {
                kind: ExprKind::Literal(token),
                span,
            },
        };

        expr
    }
    //Higher level than primary
    fn parse_leaf(&mut self) -> Result<Expr, Error> {
        if let Some(token) = self.peek().cloned() {
            let Token { kind, span } = token.clone();

            match kind {
                TokenKind::Keyword(ref kw) => match kw {
                    KeywordKind::If => self.parse_conditional(),
                    KeywordKind::Else => Err(Error::new(ErrorKind::ElseWithoutConditional, span)),
                    KeywordKind::While => self.parse_while(),
                    KeywordKind::For => self.parse_for(),
                    KeywordKind::Fn => self.parse_function(),
                    KeywordKind::Return => self.parse_return(),
                    KeywordKind::Break => self.parse_break(),
                    KeywordKind::Continue => self.parse_continue(),
                    KeywordKind::Let => self.parse_let(),
                    KeywordKind::Struct => self.parse_struct_definition(),
                    KeywordKind::Enum => self.parse_enum(),
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
                                expr = self.parse_struct(expr.clone())?;
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

    //Parsing main parts of expression
    fn parse_primary(&mut self) -> Result<Expr, Error> {
        if let Some(token) = self.peek().cloned() {
            let Token { kind, span } = token.clone();

            match kind {
                TokenKind::Punctuation(PunctuationKind::LeftBrace) => self.parse_block(),
                TokenKind::Punctuation(PunctuationKind::LeftBracket) => self.parse_array(),
                TokenKind::Punctuation(PunctuationKind::LeftParen) => self.parse_tuple(),
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

    fn parse_factor(&mut self, primary: fn(&mut Parser) -> Result<Expr, Error> ) -> Result<Expr, Error> {
        let mut left = self.parse_unary(primary)?;

        while let Some(Token {
            kind: TokenKind::Operator(op),
            ..
        }) = self.peek().cloned()
        {
            match op {
                op if op.is_factor() => {
                    let op = self.next().unwrap();

                    let right = self.parse_unary(primary)?;

                    let start = left.span.start;
                    let end = right.span.end;
                    let span = self.span(start, end);

                    let kind = ExprKind::Binary(left.into(), op, right.into());

                    left = Expr { kind, span }.transform();
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_term(&mut self, primary: fn(&mut Parser) -> Result<Expr, Error> ) -> Result<Expr, Error> {
        let mut left = self.parse_factor(primary)?;

        while let Some(Token {
            kind: TokenKind::Operator(op),
            ..
        }) = self.peek().cloned()
        {
            match op {
                op if op.is_term() => {
                    let op = self.next().unwrap();

                    let right = self.parse_factor(primary)?;

                    let start = left.span.start;
                    let end = right.span.end;
                    let span = self.span(start, end);

                    let kind = ExprKind::Binary(left.into(), op, right.into());

                    left = Expr { kind, span }.transform();
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_basic(&mut self) -> Result<Expr, Error> {
        let mut left = self.parse_term(Parser::parse_primary)?;

        while let Some(Token {
            kind: TokenKind::Operator(op),
            ..
        }) = self.peek().cloned()
        {
            if op.is_expression() {
                let op = self.next().unwrap();

                let right = self.parse_term(Parser::parse_primary)?;

                let start = left.span.start;
                let end = right.span.end;
                let span = self.span(start, end);

                let kind = ExprKind::Binary(left.into(), op, right.into());

                left = Expr { kind, span }.transform();
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_complex(&mut self) -> Result<Expr, Error> {
        let mut left = self.parse_term(Parser::parse_leaf)?;

        while let Some(Token {
                           kind: TokenKind::Operator(op),
                           ..
                       }) = self.peek().cloned()
        {
            if op.is_expression() {
                let op = self.next().unwrap();

                let right = self.parse_term(Parser::parse_leaf)?;

                let start = left.span.start;
                let end = right.span.end;
                let span = self.span(start, end);

                let kind = ExprKind::Binary(left.into(), op, right.into());

                left = Expr { kind, span }.transform();
            } else {
                break;
            }
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

    fn parse_array(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::Array, Some(SyntaxRole::Element));

        let bracket = self.next().unwrap();

        let Token {
            span: Span { start, .. },
            ..
        } = bracket;

        let mut elements = Vec::new();

        let mut err_end = start;

        while let Some(token) = self.peek().cloned() {
            match token {
                Token {
                    kind: TokenKind::Punctuation(PunctuationKind::RightBracket),
                    span: Span { end, .. },
                } => {
                    self.next();

                    self.pop_context();

                    return if elements.len() == 1 {
                        Ok(elements.pop().unwrap())
                    } else {
                        Ok(Expr {
                            kind: ExprKind::Array(elements),
                            span: self.span(start, end),
                        })
                    };
                }
                Token {
                    kind: TokenKind::Punctuation(PunctuationKind::Comma),
                    span: Span { end, .. },
                } => {
                    err_end = end;

                    self.next();
                }
                _ => {
                    let expr = self.parse_complex()?;
                    elements.push(expr);
                }
            }
        }

        let err_span = self.span(start, err_end);

        Err(Error::new(ErrorKind::UnclosedDelimiter(bracket), err_span))
    }

    fn parse_tuple(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::Tuple, Some(SyntaxRole::Parameter));

        let parenthesis = self.next().unwrap();

        let Token {
            span: Span { start, .. },
            ..
        } = parenthesis;

        let mut parameters = Vec::new();

        let mut err_end = (0usize, 0usize);

        while let Some(token) = self.peek().cloned() {
            match token {
                Token {
                    kind: TokenKind::Punctuation(PunctuationKind::RightParen),
                    span: Span { end, .. },
                } => {
                    self.next();

                    self.pop_context();

                    return if parameters.len() == 1 {
                        Ok(parameters.pop().unwrap())
                    } else {
                        Ok(Expr {
                            kind: ExprKind::Tuple(parameters),
                            span: self.span(start, end),
                        })
                    };
                }
                Token {
                    kind: TokenKind::Punctuation(PunctuationKind::Comma),
                    span: Span { end, .. },
                } => {
                    err_end = end;

                    self.next();
                }
                _ => {
                    let expr = self.parse_complex()?;
                    parameters.push(expr);
                }
            }
        }

        let err_span = self.span(start, err_end);

        Err(Error::new(
            ErrorKind::UnclosedDelimiter(parenthesis),
            err_span,
        ))
    }
}

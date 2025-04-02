use crate::axo_lexer::{KeywordKind, OperatorKind, PunctuationKind, Span, Token, TokenKind};
use crate::axo_parser::error::{ParseError};
use crate::axo_parser::{Composite, ControlFlow, Declaration, Expr, ExprKind, Parser};
use crate::axo_parser::state::{Position, Context};

pub trait Primary {
    fn parse_unary(&mut self) -> Result<Expr, ParseError>;
    fn parse_factor(&mut self) -> Result<Expr, ParseError>;
    fn parse_term(&mut self) -> Result<Expr, ParseError>;
    fn parse_expression(&mut self) -> Result<Expr, ParseError>;
    fn parse_statement(&mut self) -> Result<Expr, ParseError>;
    fn parse_primary(&mut self) -> Result<Expr, ParseError>;
    fn parse_tuple(&mut self) -> Result<Expr, ParseError>;
    fn parse_array(&mut self) -> Result<Expr, ParseError>;
}

impl Primary for Parser {
    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if let Some(Token {
                        kind: TokenKind::Operator(op),
                        span: Span { start, .. },
                    }) = self.peek().cloned()
        {
            if op.is_prefix() {
                let op = self.next().unwrap();

                let unary = self.parse_unary()?;
                let end = unary.span.end;

                let span = self.span(start, end);

                let kind = ExprKind::Unary(op, unary.into());

                let expr = Expr { kind, span };

                return Ok(expr);
            }
        }

        let mut expr = self.parse_primary()?;

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
    fn parse_factor(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;

        while let Some(Token {
            kind: TokenKind::Operator(op),
            ..
        }) = self.peek().cloned()
        {
            match op {
                op if op.is_factor() => {
                    let op = self.next().unwrap();

                    let right = self.parse_unary()?;

                    let start = left.span.start;
                    let end = right.span.end;
                    let span = self.span(start, end);

                    let kind = ExprKind::Binary(left.into(), op, right.into());

                    left = Expr { kind, span };
                }
                OperatorKind::Colon => {
                    self.next();

                    let right = self.parse_unary()?;

                    let start = left.span.start;
                    let end = right.span.end;
                    let span = self.span(start, end);

                    let kind = ExprKind::Typed(left.into(), right.into());

                    left = Expr { kind, span };
                }
                OperatorKind::Dot => {
                    self.next();

                    let field = self.parse_expression()?;

                    let span = self.span(left.span.start, field.span.end);

                    let kind = ExprKind::Member(left.into(), field.into());
                    left = Expr { kind, span };
                }
                _ => break,
            }
        }

        Ok(left)
    }
    fn parse_term(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_factor()?;

        while let Some(Token {
            kind: TokenKind::Operator(op),
            ..
        }) = self.peek().cloned()
        {
            match op {
                OperatorKind::ColonEqual => {
                    self.next();
                    let right = self.parse_statement()?;
                    let span = self.span(left.span.start, right.span.end);

                    left = Expr {
                        kind: ExprKind::Definition(left.into(), Some(right.into())),
                        span,
                    };
                }
                OperatorKind::Equal => {
                    self.next();
                    let right = self.parse_statement()?;
                    let span = self.span(left.span.start, right.span.end);

                    left = Expr {
                        kind: ExprKind::Assignment(left.into(), right.into()),
                        span,
                    };
                }
                ref operator if operator.is_compound() => {
                    let token = self.next().unwrap();
                    let right = self.parse_statement()?;
                    let span = self.span(left.span.start, right.span.end);
                    let operation = Expr {
                        kind: ExprKind::Binary(
                            left.clone().into(),
                            OperatorKind::decompound_token(&token),
                            right.into(),
                        ),
                        span: span.clone(),
                    };

                    left = Expr {
                        kind: ExprKind::Assignment(left.into(), operation.into()),
                        span,
                    };
                }
                op if op.is_term() => {
                    let op = self.next().unwrap();

                    let right = self.parse_factor()?;

                    let start = left.span.start;
                    let end = right.span.end;
                    let span = self.span(start, end);

                    let kind = ExprKind::Binary(left.into(), op, right.into());

                    left = Expr { kind, span };
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_term()?;

        while let Some(Token {
            kind: TokenKind::Operator(op),
            ..
        }) = self.peek().cloned()
        {
            if op.is_expression() {
                let op = self.next().unwrap();

                let right = self.parse_term()?;

                let start = left.span.start;
                let end = right.span.end;
                let span = self.span(start, end);

                let kind = ExprKind::Binary(left.into(), op, right.into());

                left = Expr { kind, span };
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_statement(&mut self) -> Result<Expr, ParseError> {
        self.enter(Context::Statement);

        let result = if let Some(token) = self.peek().cloned() {
            let Token { kind, .. } = token.clone();

            let expr = match kind {
                TokenKind::Keyword(kw) => {
                    match kw {
                        KeywordKind::If => self.parse_if_statement(),
                        KeywordKind::While => self.parse_while_statement(),
                        KeywordKind::For => self.parse_for_statement(),
                        KeywordKind::Fn => self.parse_function(),
                        KeywordKind::Return => self.parse_return_statement(),
                        KeywordKind::Break => self.parse_break_statement(),
                        KeywordKind::Continue => self.parse_continue_statement(),
                        KeywordKind::Let => self.parse_let(),
                        KeywordKind::Struct => self.parse_struct_definition(),
                        KeywordKind::Enum => self.parse_enum(),
                        _ => {
                            println!("{}", kw);
                            Err(ParseError::UnimplementedFeature)
                        }
                    }?
                }
                TokenKind::Punctuation(PunctuationKind::LeftBrace) => self.parse_block()?,

                _ => {
                    let left = self.parse_expression()?;

                    if let Some(token) = self.peek().cloned() {
                        match token.kind {

                            _ => left,
                        }
                    } else {
                        return Err(ParseError::UnexpectedEndOfFile);
                    }
                }
            };

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
            Err(ParseError::UnexpectedEndOfFile)
        };

        self.exit();

        result
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        if let Some(token) = self.peek().cloned() {
            let Token { kind, span } = token.clone();

            match kind {
                TokenKind::Keyword(kw) => {
                    match kw {
                        KeywordKind::If => self.parse_if_statement(),
                        KeywordKind::While => self.parse_while_statement(),
                        KeywordKind::For => self.parse_for_statement(),
                        KeywordKind::Fn => self.parse_function(),
                        KeywordKind::Return => self.parse_return_statement(),
                        KeywordKind::Break => self.parse_break_statement(),
                        KeywordKind::Continue => self.parse_continue_statement(),
                        KeywordKind::Let => self.parse_let(),
                        KeywordKind::Struct => self.parse_struct_definition(),
                        KeywordKind::Enum => self.parse_enum(),
                        _ => {
                            println!("{}", kw);
                            Err(ParseError::UnimplementedFeature)
                        }
                    }
                }
                TokenKind::Punctuation(PunctuationKind::LeftBracket) => self.parse_array(),
                TokenKind::Punctuation(PunctuationKind::LeftParen) => self.parse_tuple(),
                TokenKind::Operator(OperatorKind::Pipe) => self.parse_closure(),
                TokenKind::Identifier(name) => {
                    self.next();
                    let kind = ExprKind::Identifier(name.clone());
                    let mut expr = Expr { kind, span };

                    while let Some(token) = self.peek() {
                        match &token.kind {
                            TokenKind::Punctuation(PunctuationKind::LeftBrace) => {
                                if self.current_context() != Context::Clause {
                                    expr = self.parse_struct(expr)?;
                                } else {
                                    break;
                                }
                            }
                            TokenKind::Punctuation(PunctuationKind::LeftBracket) => {

                                expr = self.parse_index(expr)?;
                            }
                            TokenKind::Punctuation(PunctuationKind::LeftParen) => {
                                expr = self.parse_invoke(expr)?;
                                return Ok(expr);
                            }
                            _ => break,
                        }
                    }

                    Ok(expr)
                }
                TokenKind::Str(_)
                | TokenKind::Char(_)
                | TokenKind::Boolean(_)
                | TokenKind::Float(_)
                | TokenKind::Integer(_)
                | TokenKind::Operator(_) => {
                    self.next();
                    let kind = ExprKind::Literal(token.clone());
                    let mut expr = Expr { kind, span };

                    while let Some(token) = self.peek() {
                        match &token.kind {
                            TokenKind::Punctuation(PunctuationKind::LeftBrace) => {
                                if self.current_context() != Context::Clause {
                                    expr = self.parse_struct(expr)?;
                                } else {
                                    break;
                                }
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

                TokenKind::EOF => Err(ParseError::UnexpectedEndOfFile),
                token => Err(ParseError::InvalidSyntaxPattern(format!(
                    "Unexpected token: {:?}",
                    token
                ))),
            }
        } else {
            Err(ParseError::UnexpectedEndOfFile)
        }
    }

    fn parse_tuple(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let mut parameters = Vec::new();

        while let Some(token) = self.peek().cloned() {
            match token {
                Token {
                    kind: TokenKind::Punctuation(PunctuationKind::RightParen),
                    span: Span { end, .. },
                } => {
                    self.next();

                    return if parameters.len() == 1 {
                        Ok(parameters.pop().unwrap())
                    } else {
                        Ok(Expr {
                            kind: ExprKind::Tuple(parameters),
                            span: self.span(start, end),
                        })
                    }
                }
                Token {
                    kind: TokenKind::Punctuation(PunctuationKind::Comma),
                    ..
                } => {
                    self.next();
                }
                _ => {
                    let expr = self.parse_expression()?;
                    parameters.push(expr);
                }
            }
        }

        let err = ParseError::ExpectedTokenNotFound(
            TokenKind::Punctuation(PunctuationKind::RightParen),
            Position::After,
            Context::TupleElements,
        );

        Err(err)
    }

    fn parse_array(&mut self) -> Result<Expr, ParseError> {
        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let mut elements = Vec::new();

        while let Some(token) = self.peek().cloned() {
            match token {
                Token {
                    kind: TokenKind::Punctuation(PunctuationKind::RightBracket),
                    span: Span { end, .. },
                } => {
                    self.next();

                    return Ok(Expr {
                        kind: ExprKind::Array(elements),
                        span: self.span(start, end),
                    });
                }
                Token {
                    kind: TokenKind::Punctuation(PunctuationKind::Comma),
                    ..
                } => {
                    self.next();
                }
                _ => {
                    let expr = self.parse_expression()?;
                    elements.push(expr.into());
                }
            }
        }

        let err = ParseError::ExpectedTokenNotFound(
            TokenKind::Punctuation(PunctuationKind::RightBrace),
            Position::After,
            Context::ArrayElements,
        );

        Err(err)
    }
}

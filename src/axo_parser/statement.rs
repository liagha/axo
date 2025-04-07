use std::cmp::PartialEq;
use crate::axo_lexer::{KeywordKind, OperatorKind, PunctuationKind, Span, Token, TokenKind};
use crate::axo_parser::error::{Error, ErrorKind};
use crate::axo_parser::expression::{Expr, ExprKind, Expression};
use crate::axo_parser::{ItemKind, Parser, Primary};
use crate::axo_parser::state::{Position, Context, ContextKind, SyntaxRole};

pub trait ControlFlow {
    fn parse_delimited<F>(
        &mut self,
        context_kind: ContextKind,
        syntax_role: Option<SyntaxRole>,
        open_kind: TokenKind,
        close_kind: TokenKind,
        separator: TokenKind,
        forced_separator: bool,
        item_parser: F,
    ) -> Result<(Vec<Expr>, Span), Error>
    where
        F: FnMut(&mut Parser) -> Result<Expr, Error>;
    fn parse_let(&mut self) -> Result<Expr, Error>;
    fn parse_block(&mut self) -> Result<Expr, Error>;
    fn parse_match(&mut self) -> Result<Expr, Error>;
    fn parse_conditional(&mut self) -> Result<Expr, Error>;
    fn parse_while(&mut self) -> Result<Expr, Error>;
    fn parse_for(&mut self) -> Result<Expr, Error>;
    fn parse_return(&mut self) -> Result<Expr, Error>;
    fn parse_break(&mut self) -> Result<Expr, Error>;
    fn parse_continue(&mut self) -> Result<Expr, Error>;
}

impl ControlFlow for Parser {
    fn parse_delimited<F>(
        &mut self,
        context_kind: ContextKind,
        syntax_role: Option<SyntaxRole>,
        _open_kind: TokenKind,
        close_kind: TokenKind,
        separator: TokenKind,
        forced_separator: bool,
        mut item_parser: F,
    ) -> Result<(Vec<Expr>, Span), Error>
    where
        F: FnMut(&mut Parser) -> Result<Expr, Error>
    {
        self.push_context(context_kind, syntax_role);

        let open_token = self.next().unwrap();
        let Span { start, .. } = open_token.span;

        let mut items = Vec::new();
        let mut err_end = start;

        while let Some(token) = self.peek().cloned() {
            match token.kind {
                kind if kind == close_kind => {
                    let close_token = self.next().unwrap();
                    let Span { end, .. } = close_token.span;

                    self.pop_context();

                    return Ok((items, self.span(start, end)));
                }
                kind if kind == separator => {
                    err_end = token.span.end;
                    self.next();
                }
                _ => {
                    let item = item_parser(self)?;
                    let Expr { span: Span { start: item_start, .. }, .. } = item;

                    items.push(item.clone());

                    err_end = item.span.end;

                    if forced_separator {
                        if let Some(peek) = self.peek() {
                            if peek.kind == separator {
                                err_end = token.span.end;

                                self.next();
                            } else if peek.kind != close_kind {
                                self.next();
                                return Err(Error::new(
                                    ErrorKind::MissingSeparator(separator),
                                    self.span(item_start, err_end),
                                ))
                            }
                        } else {

                        }
                    }
                }
            }
        }

        Err(Error::new(
            ErrorKind::UnclosedDelimiter(open_token),
            self.span(start, err_end),
        ))
    }

    fn parse_let(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::Variable, Some(SyntaxRole::Declaration));

        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let expr = self.parse_complex()?;

        let Expr { kind, span: Span { end, .. } } = expr.clone();

        let span = self.span(start, end);

        self.pop_context();

        match kind {
            ExprKind::Assignment(target, value) => {
                Ok(Expr { kind: ExprKind::Definition(target, Some(value)), span })
            }
            _ => {
                Ok(Expr { kind: ExprKind::Definition(expr.into(), None), span })
            }
        }
    }

    fn parse_block(&mut self) -> Result<Expr, Error> {
        let brace = self.next().unwrap();

        let Token {
            span: Span { start, .. },
            ..
        } = brace;

        let mut statements = Vec::new();

        let mut err_end = start;

        while let Some(token) = self.peek().cloned() {
            match token {
                Token { kind: TokenKind::Punctuation(PunctuationKind::RightBrace), .. } => {
                    let Token {
                        span: Span { end, .. },
                        ..
                    } = self.next().unwrap();

                    let kind = ExprKind::Block(statements);
                    let expr = Expr {
                        kind,
                        span: self.span(start, end),
                    };

                    return Ok(expr);
                }
                Token { kind: TokenKind::Punctuation(PunctuationKind::Semicolon), span: Span { end, .. } } => {
                    err_end = end;

                    self.next();
                }
                _ => {
                    let stmt = self.parse_statement()?;
                    statements.push(stmt.into());
                }
            }
        }

        Err(Error::new(ErrorKind::UnclosedDelimiter(brace), self.span(start, err_end)))
    }

    fn parse_match(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::Match, Some(SyntaxRole::Clause));

        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let clause = self.parse_basic()?;

        self.pop_context();

        let body = if let Some(Token { kind: TokenKind::Punctuation(PunctuationKind::LeftBrace), .. }) = self.peek() {
            let (exprs, span) = self.parse_delimited(
                ContextKind::Match,
                Some(SyntaxRole::Body),
                TokenKind::Punctuation(PunctuationKind::LeftBrace),
                TokenKind::Punctuation(PunctuationKind::RightBrace),
                TokenKind::Punctuation(PunctuationKind::Comma),
                true,
                Parser::parse_complex
            )?;

            Expr { kind: ExprKind::Block(exprs), span }
        } else {
            self.parse_complex()?
        };

        let end = body.span.end;
        let kind = ExprKind::Match(clause.into(), body.into());
        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        Ok(expr)
    }

    fn parse_conditional(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::If, Some(SyntaxRole::Condition));

        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let condition = self.parse_basic()?;

        self.pop_context();

        self.push_context(ContextKind::If, Some(SyntaxRole::Then));

        let then_branch = self.parse_statement()?;

        self.pop_context();

        let (else_branch, end) = if self.match_token(&TokenKind::Keyword(KeywordKind::Else)) {
            self.push_context(ContextKind::If, Some(SyntaxRole::Else));

            let expr = self.parse_statement()?;
            let end = expr.span.end;

            self.pop_context();

            (Some(expr.into()), end)
        } else {
            (None, then_branch.span.end)
        };

        let kind = ExprKind::Conditional(condition.into(), then_branch.into(), else_branch.into());
        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        Ok(expr)
    }

    fn parse_while(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::While, Some(SyntaxRole::Condition));

        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let condition = self.parse_basic()?;

        self.pop_context();

        self.push_context(ContextKind::While, Some(SyntaxRole::Body));

        let body = self.parse_statement()?;

        self.pop_context();

        let end = body.span.end;
        let kind = ExprKind::While(condition.into(), body.into());
        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        Ok(expr)
    }

    fn parse_for(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::For, Some(SyntaxRole::Clause));

        let Token {
            span: Span { start, .. },
            ..
        } = self.next().unwrap();

        let clause = self.parse_basic()?;

        self.pop_context();

        self.push_context(ContextKind::For, Some(SyntaxRole::Body));

        let body = self.parse_statement()?;

        self.pop_context();

        let end = body.span.end;
        let kind = ExprKind::For(clause.into(), body.into());
        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        Ok(expr)
    }


    fn parse_return(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::Return, None);

        let Token {
            span: Span { start, end, .. },
            ..
        } = self.next().unwrap();

        let (value, end) = if self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon))
        {
            (None, end)
        } else {
            self.push_context(ContextKind::Return, Some(SyntaxRole::Value));

            let expr = self.parse_complex()?;
            let end = expr.span.end;

            self.pop_context();

            (Some(expr.into()), end)
        };

        self.pop_context();

        let kind = ExprKind::Return(value);
        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        Ok(expr)
    }

    fn parse_break(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::Break, None);

        let Token {
            span: Span { start, end, .. },
            ..
        } = self.next().unwrap();

        let (value, end) = if self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon))
        {
            (None, end)
        } else {
            self.push_context(ContextKind::Break, Some(SyntaxRole::Value));

            let expr = self.parse_complex()?;
            let end = expr.span.end;

            self.pop_context();

            (Some(expr.into()), end)
        };

        self.pop_context();

        let kind = ExprKind::Break(value);
        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        Ok(expr)
    }

    fn parse_continue(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::Continue, None);

        let Token {
            span: Span { start, end, .. },
            ..
        } = self.next().unwrap();

        let (value, end) = if self.match_token(&TokenKind::Punctuation(PunctuationKind::Semicolon))
        {
            (None, end)
        } else {
            self.push_context(ContextKind::Continue, Some(SyntaxRole::Value));

            let expr = self.parse_complex()?;
            let end = expr.span.end;

            self.pop_context();

            (Some(expr.into()), end)
        };

        self.pop_context();

        let kind = ExprKind::Continue(value);
        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        Ok(expr)
    }
}
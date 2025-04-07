use crate::axo_lexer::{PunctuationKind, Span, Token, TokenKind};
use crate::axo_parser::{Expr, ExprKind, Parser, Primary};
use crate::axo_parser::error::{Error, ErrorKind};
use crate::axo_parser::expression::Expression;
use crate::axo_parser::state::{ContextKind, SyntaxRole};

pub trait Delimiter {
    fn parse_delimited<F>(
        &mut self,
        context_kind: ContextKind,
        syntax_role: Option<SyntaxRole>,
        _open_kind: TokenKind,
        close_kind: TokenKind,
        separator: TokenKind,
        forced_separator: bool,
        item_parser: F,
    ) -> Result<(Vec<Expr>, Span), Error>
    where
        F: FnMut(&mut Parser) -> Result<Expr, Error>;
    fn parse_braced(&mut self) -> Result<Expr, Error>;
    fn parse_bracketed(&mut self) -> Result<Expr, Error>;
    fn parse_parenthesized(&mut self) -> Result<Expr, Error>;
}

impl Delimiter for Parser {
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
    fn parse_braced(&mut self) -> Result<Expr, Error> {
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

    fn parse_bracketed(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::Bracketed, None);

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

    fn parse_parenthesized(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::Parenthesized, None);

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
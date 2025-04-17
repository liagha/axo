use crate::axo_lexer::{OperatorKind, PunctuationKind, Token, TokenKind};
use crate::axo_parser::error::ErrorKind;
use crate::axo_parser::{ControlFlow, ParseError, Expr, ExprKind, Parser, Primary};
use crate::axo_parser::delimiter::Delimiter;
use crate::axo_parser::expression::Expression;
use crate::axo_span::Span;

pub trait Composite {
    fn parse_index(&mut self, left: Expr) -> Expr;
    fn parse_invoke(&mut self, name: Expr) -> Expr;
    fn parse_constructor(&mut self, struct_name: Expr) -> Expr;
    fn parse_closure(&mut self) -> Expr;
}

impl Composite for Parser {
    fn parse_index(&mut self, left: Expr) -> Expr {
        let bracket = self.next().unwrap();

        let Expr {
            span: Span { start, .. },
            ..
        } = left;

        let index = self.parse_complex();

        let err_end = index.span.end;

        let result = if let Some(Token {
            kind: TokenKind::Punctuation(PunctuationKind::RightBracket),
            span: Span { end, .. },
        }) = self.next()
        {
            let kind = ExprKind::Index {
                expr: left.into(),
                index: index.into()
            };

            let span = self.span(start, end);
            let expr = Expr { kind, span };

            expr
        } else {
            let err_span = self.span(start, err_end);
            
            self.error(&ParseError::new(ErrorKind::UnclosedDelimiter(bracket), err_span))
        };

        result
    }

    fn parse_invoke(&mut self, name: Expr) -> Expr {
        let Expr {
            span: Span { start, .. },
            ..
        } = name;

        let parameters = self.parse_group();

        let result = match parameters {
            Expr {
                kind: ExprKind::Group(parameters),
                span: Span { end, .. },
            } => {
                let kind = ExprKind::Invoke {
                    target: name.into(),
                    parameters
                };

                let expr = Expr {
                    kind,
                    span: self.span(start, end),
                };

                expr
            }
            Expr {
                span: Span { end, .. },
                ..
            } => {
                let kind = ExprKind::Invoke {
                    target: name.into(),
                    parameters: vec![parameters],
                };

                let expr = Expr {
                    kind,
                    span: self.span(start, end),
                };

                expr
            }
        };

        result
    }

    fn parse_constructor(&mut self, struct_name: Expr) -> Expr {
        let Expr {
            span: Span { start, .. },
            ..
        } = struct_name;

        let body = if let Some(Token { kind: TokenKind::Punctuation(PunctuationKind::LeftBrace), .. }) = self.peek() {
            let (exprs, span) = self.parse_delimited(
                TokenKind::Punctuation(PunctuationKind::LeftBrace),
                TokenKind::Punctuation(PunctuationKind::RightBrace),
                TokenKind::Punctuation(PunctuationKind::Comma),
                true,
                Parser::parse_complex
            );

            Expr { kind: ExprKind::Block(exprs), span }
        } else {
            self.parse_complex()
        };

        let end = body.span.end;

        let kind = ExprKind::Constructor {
            name: struct_name.into(),
            body: body.into()
        };

        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        expr
    }

    fn parse_closure(&mut self) -> Expr {
        let pipe = self.next().unwrap();

        let Token {
            span: Span { start, .. },
            ..
        } = pipe;

        let mut parameters = Vec::new();

        let mut err_end = start;

        while let Some(token) = self.peek().cloned() {
            match token {
                Token {
                    kind: TokenKind::Operator(OperatorKind::Pipe),
                    span: Span { end, .. },
                } => {
                    self.next();

                    let body = self.parse_statement();

                    return Expr {
                        kind: ExprKind::Closure {
                            parameters,
                            body: body.into()
                        },
                        span: self.span(start, end),
                    };
                }
                Token {
                    kind: TokenKind::Punctuation(PunctuationKind::Comma),
                    span: Span { end, .. }
                } => {
                    err_end = end;

                    self.next();
                }
                _ => {
                    let expr = self.parse_complex();
                    parameters.push(expr.into());
                }
            }
        }

        self.error(&ParseError::new(ErrorKind::UnclosedDelimiter(pipe), self.span(start, err_end)))
    }
}

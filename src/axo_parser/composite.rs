use {
    crate::{
        axo_lexer::{
            OperatorKind, PunctuationKind,
            Token, TokenKind
        },
        axo_parser::{
            error::ErrorKind,
            delimiter::Delimiter,
            expression::Expression,
            ParseError, Expr, ExprKind,
            Parser, Primary, ControlFlow,
        },
        axo_span::Span,
    },
};

pub trait Composite {
    fn parse_index(&mut self, left: Expr) -> Expr;
    fn parse_invoke(&mut self, name: Expr) -> Expr;
    fn parse_constructor(&mut self, struct_name: Expr) -> Expr;
    fn parse_closure(&mut self) -> Expr;
}

impl Composite for Parser {
    fn parse_index(&mut self, left: Expr) -> Expr {
        let index = self.parse_complex();

        let Expr {
            span: Span { start, .. },
            ..
        } = left;

        let Expr {
            span: Span { end, .. },
            ..
        } = index;

        let result = {
            let kind = ExprKind::Index {
                expr: left.into(),
                index: index.into()
            };

            let span = self.span(start, end);
            let expr = Expr { kind, span };

            expr
        };

        result
    }

    fn parse_invoke(&mut self, name: Expr) -> Expr {
        let Expr {
            span: Span { start, .. },
            ..
        } = name;

        let parameters = self.parse_parenthesized();

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

        let body = self.parse_complex();

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

use crate::axo_lexer::{OperatorKind, PunctuationKind, Span, Token, TokenKind};
use crate::axo_parser::error::ErrorKind;
use crate::axo_parser::{ControlFlow, Error, Expr, ExprKind, Parser, Primary};
use crate::axo_parser::delimiter::Delimiter;
use crate::axo_parser::expression::Expression;
use crate::axo_parser::state::{Position, Context, ContextKind, SyntaxRole};

pub trait Composite {
    fn parse_index(&mut self, left: Expr) -> Result<Expr, Error>;
    fn parse_invoke(&mut self, name: Expr) -> Result<Expr, Error>;
    fn parse_structure(&mut self, struct_name: Expr) -> Result<Expr, Error>;
    fn parse_closure(&mut self) -> Result<Expr, Error>;
}

impl Composite for Parser {
    fn parse_index(&mut self, left: Expr) -> Result<Expr, Error> {
        self.push_context(ContextKind::Index, None);

        let bracket = self.next().unwrap();

        let Expr {
            span: Span { start, .. },
            ..
        } = left;

        self.push_context(ContextKind::Index, Some(SyntaxRole::Value));

        let index = self.parse_complex()?;

        self.pop_context();

        self.pop_context();

        let err_end = index.span.end;

        let result = if let Some(Token {
            kind: TokenKind::Punctuation(PunctuationKind::RightBracket),
            span: Span { end, .. },
        }) = self.next()
        {
            let kind = ExprKind::Index(left.into(), index.into());
            let span = self.span(start, end);
            let expr = Expr { kind, span };

            Ok(expr)
        } else {
            let err_span = self.span(start, err_end);
            
            Err(Error::new(ErrorKind::UnclosedDelimiter(bracket), err_span))
        };

        result
    }

    fn parse_invoke(&mut self, name: Expr) -> Result<Expr, Error> {
        self.push_context(ContextKind::Call, None);

        let Expr {
            span: Span { start, .. },
            ..
        } = name;

        self.push_context(ContextKind::Call, Some(SyntaxRole::Parameter));

        let parameters = self.parse_parenthesized()?;

        self.pop_context();

        self.pop_context();

        let result = match parameters {
            Expr {
                kind: ExprKind::Tuple(parameters),
                span: Span { end, .. },
            } => {
                let kind = ExprKind::Invoke(name.into(), parameters);
                let expr = Expr {
                    kind,
                    span: self.span(start, end),
                };

                Ok(expr)
            }
            Expr {
                span: Span { end, .. },
                ..
            } => {
                let kind = ExprKind::Invoke(name.into(), vec![parameters]);
                let expr = Expr {
                    kind,
                    span: self.span(start, end),
                };

                Ok(expr)
            }
        };

        result
    }

    fn parse_structure(&mut self, struct_name: Expr) -> Result<Expr, Error> {
        self.push_context(ContextKind::Struct, None);

        let Expr {
            span: Span { start, .. },
            ..
        } = struct_name;

        self.push_context(ContextKind::Struct, Some(SyntaxRole::Field));

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

        let kind = ExprKind::Struct(struct_name.into(), body.into());

        let expr = Expr {
            kind,
            span: self.span(start, end),
        };

        Ok(expr)
    }

    fn parse_closure(&mut self) -> Result<Expr, Error> {
        self.push_context(ContextKind::Closure, None);

        let pipe = self.next().unwrap();

        let Token {
            span: Span { start, .. },
            ..
        } = pipe;

        self.push_context(ContextKind::Closure, Some(SyntaxRole::Parameter));

        let mut parameters = Vec::new();

        let mut err_end = start;

        while let Some(token) = self.peek().cloned() {
            match token {
                Token {
                    kind: TokenKind::Operator(OperatorKind::Pipe),
                    span: Span { end, .. },
                } => {
                    self.next();

                    self.push_context(ContextKind::Closure, Some(SyntaxRole::Body));

                    self.pop_context();

                    self.pop_context();

                    self.pop_context();

                    let body = self.parse_statement()?;

                    return Ok(Expr {
                        kind: ExprKind::Closure(parameters, body.into()),
                        span: self.span(start, end),
                    });
                }
                Token {
                    kind: TokenKind::Punctuation(PunctuationKind::Comma),
                    span: Span { end, .. }
                } => {
                    err_end = end;

                    self.next();
                }
                _ => {
                    let expr = self.parse_complex()?;
                    parameters.push(expr.into());
                }
            }
        }

        Err(Error::new(ErrorKind::UnclosedDelimiter(pipe), self.span(start, err_end)))
    }
}

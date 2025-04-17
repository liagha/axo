use crate::axo_lexer::{PunctuationKind, Token, TokenKind};
use crate::axo_parser::{ParseError, Expr, ExprKind, Parser, Primary};
use crate::axo_parser::error::ErrorKind;
use crate::axo_parser::expression::Expression;
use crate::axo_span::{Span, Spanned};

pub trait Delimiter {
    fn parse_delimited<F, R>(
        &mut self,
        _open_kind: TokenKind,
        close_kind: TokenKind,
        separator: TokenKind,
        forced_separator: bool,
        item_parser: F,
    ) -> (Vec<R>, Span)
    where
        R: Spanned + Clone,
        F: FnMut(&mut Parser) -> R;
    fn parse_braced(&mut self) -> Expr;
    fn parse_collection(&mut self) -> Expr;
    fn parse_group(&mut self) -> Expr;
}

impl Delimiter for Parser {
    fn parse_delimited<F, R>(
        &mut self,
        _open_kind: TokenKind,
        close_kind: TokenKind,
        separator: TokenKind,
        forced_separator: bool,
        mut item_parser: F,
    ) -> (Vec<R>, Span)
    where
        R: Spanned + Clone,
        F: FnMut(&mut Parser) -> R
    {
        let open_token = self.next().unwrap();
        let Span { start, .. } = open_token.span;

        let mut items = Vec::new();
        let mut err_end = start;

        while let Some(token) = self.peek().cloned() {
            match token.kind {
                kind if kind == close_kind => {
                    let close_token = self.next().unwrap();
                    let Span { end, .. } = close_token.span;

                    

                    return (items, self.span(start, end));
                }
                kind if kind == separator => {
                    err_end = token.span.end;
                    self.next();
                }
                _ => {
                    let item = item_parser(self);
                    let item_start = item.span().start;

                    items.push(item.clone());

                    err_end = item.span().end;

                    if forced_separator {
                        if let Some(peek) = self.peek() {
                            if peek.kind == separator {
                                err_end = token.span.end;

                                self.next();
                            } else if peek.kind != close_kind {
                                self.next();

                                self.error(&ParseError::new(
                                    ErrorKind::MissingSeparator(separator),
                                    self.span(item_start, err_end),
                                ));

                                return (items, self.span(start, err_end));
                            }
                        } else {

                        }
                    }
                }
            }
        }

        self.error(&ParseError::new(
            ErrorKind::UnclosedDelimiter(open_token),
            self.span(start, err_end),
        ));

        (items, self.span(start, err_end))
    }
    fn parse_braced(&mut self) -> Expr {
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

                    return expr;
                }
                Token { kind: TokenKind::Punctuation(PunctuationKind::Semicolon), span: Span { end, .. } } => {
                    err_end = end;

                    self.next();
                }
                _ => {
                    let stmt = self.parse_statement();
                    statements.push(stmt.into());
                }
            }
        }

        self.error(&ParseError::new(ErrorKind::UnclosedDelimiter(brace), self.span(start, err_end)))
    }

    fn parse_collection(&mut self) -> Expr {
        if let Some(token) = self.peek() {
            if token.kind != TokenKind::Punctuation(PunctuationKind::LeftBracket) {
                return self.error(&ParseError::new(
                    ErrorKind::ExpectedToken(TokenKind::Punctuation(PunctuationKind::LeftBracket)),
                    token.span.clone(),
                ));
            }
        }

        let (elements, span) = self.parse_delimited(
            TokenKind::Punctuation(PunctuationKind::LeftBracket),
            TokenKind::Punctuation(PunctuationKind::RightBracket),
            TokenKind::Punctuation(PunctuationKind::Comma),
            false,
            |parser| parser.parse_complex(),
        );

        // Return a single element if there's only one, otherwise return a collection
        if elements.len() == 1 {
            elements.into_iter().next().unwrap()
        } else {
            Expr {
                kind: ExprKind::Collection(elements),
                span,
            }
        }
    }

    fn parse_group(&mut self) -> Expr {
        if let Some(token) = self.peek() {
            if token.kind != TokenKind::Punctuation(PunctuationKind::LeftParen) {
                return self.error(&ParseError::new(
                    ErrorKind::ExpectedToken(TokenKind::Punctuation(PunctuationKind::LeftParen)),
                    token.span.clone(),
                ));
            }
        }

        let (parameters, span) = self.parse_delimited(
            TokenKind::Punctuation(PunctuationKind::LeftParen),
            TokenKind::Punctuation(PunctuationKind::RightParen),
            TokenKind::Punctuation(PunctuationKind::Comma),
            false,
            |parser| parser.parse_complex(),
        );

        // Return a single parameter if there's only one, otherwise return a group
        if parameters.len() == 1 {
            parameters.into_iter().next().unwrap()
        } else {
            Expr {
                kind: ExprKind::Group(parameters),
                span,
            }
        }
    }
}
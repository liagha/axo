use {
    crate::{
        axo_lexer::{
            PunctuationKind, Token, TokenKind
        },
        axo_parser::{
            error::ErrorKind,
            expression::Expression,
            Expr, ExprKind,
            ParseError, Parser, Primary,
        },
        axo_span::{
            Span, Spanned
        }
    }
};

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
        F: FnMut(&mut Parser) -> R,
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
        if let Some(token) = self.peek() {
            if token.kind != TokenKind::Punctuation(PunctuationKind::LeftBrace) {
                return self.error(&ParseError::new(
                    ErrorKind::ExpectedToken(TokenKind::Punctuation(PunctuationKind::LeftBrace)),
                    token.span.clone(),
                ));
            }
        }

        let brace = self.next().unwrap();
        let Token {
            span: Span { start, .. },
            ..
        } = brace;

        let mut items = Vec::new();
        let mut err_end = start;
        let mut separator_type: Option<TokenKind> = None;
        let mut has_inconsistent_separators = false;
        let mut inconsistent_separator_span = None;

        while let Some(token) = self.peek().cloned() {
            match token.kind {
                TokenKind::Punctuation(PunctuationKind::RightBrace) => {
                    let Token {
                        span: Span { end, .. },
                        ..
                    } = self.next().unwrap();

                    if has_inconsistent_separators {
                        let error_span = self.span(start, end);
                        return self.error(&ParseError::new(
                            ErrorKind::InconsistentSeparators,
                            inconsistent_separator_span.unwrap_or(error_span),
                        ));
                    }

                    let kind = match separator_type {
                        Some(TokenKind::Punctuation(PunctuationKind::Comma)) | None => {
                            ExprKind::Bundle(items)
                        }
                        Some(TokenKind::Punctuation(PunctuationKind::Semicolon)) => {
                            ExprKind::Block(items)
                        }
                        _ => unreachable!(), // This shouldn't happen with our current token types
                    };

                    return Expr {
                        kind,
                        span: self.span(start, end),
                    };
                }
                TokenKind::Punctuation(PunctuationKind::Comma) => {
                    if let Some(TokenKind::Punctuation(PunctuationKind::Semicolon)) = separator_type
                    {
                        // Mark as inconsistent but continue parsing
                        has_inconsistent_separators = true;
                        if inconsistent_separator_span.is_none() {
                            inconsistent_separator_span = Some(token.span.clone());
                        }
                    }

                    separator_type = Some(TokenKind::Punctuation(PunctuationKind::Comma));
                    err_end = token.span.end;
                    self.next();
                }
                TokenKind::Punctuation(PunctuationKind::Semicolon) => {
                    // Check for inconsistent separators
                    if let Some(TokenKind::Punctuation(PunctuationKind::Comma)) = separator_type {
                        // Mark as inconsistent but continue parsing
                        has_inconsistent_separators = true;
                        if inconsistent_separator_span.is_none() {
                            inconsistent_separator_span = Some(token.span.clone());
                        }
                    }
                    separator_type = Some(TokenKind::Punctuation(PunctuationKind::Semicolon));
                    err_end = token.span.end;
                    self.next();
                }
                _ => {
                    // Parse an expression or statement depending on the separator type
                    let item =
                        if separator_type == Some(TokenKind::Punctuation(PunctuationKind::Comma)) {
                            self.parse_complex() // For bundles, parse expressions
                        } else {
                            self.parse_statement().into() // For blocks, parse statements
                        };

                    err_end = item.span().end;
                    items.push(item);
                }
            }
        }

        // If we get here, we have an unclosed delimiter
        self.error(&ParseError::new(
            ErrorKind::UnclosedDelimiter(brace),
            self.span(start, err_end),
        ))
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

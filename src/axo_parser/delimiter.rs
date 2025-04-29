use {
    crate::{
        axo_lexer::{
            PunctuationKind, Token, TokenKind
        },
        axo_parser::{
            error::ErrorKind,
            Element, ElementKind,
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
    fn parse_braced(&mut self) -> Element;
    fn parse_bracketed(&mut self) -> Element;
    fn parse_parenthesized(&mut self) -> Element;
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

    fn parse_braced(&mut self) -> Element {
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

        let err_end = start;
        let mut items = Vec::new();
        let mut separator = Option::<PunctuationKind>::None;

        while let Some(Token { kind, span }) = self.peek().cloned() {
            match kind {
                TokenKind::Punctuation(PunctuationKind::RightBrace) => {
                    self.next();

                    return if separator == Some(PunctuationKind::Semicolon) {
                        let kind = ElementKind::Scope(items.clone());
                        let span = self.span(start, span.end);

                        Element {
                            kind,
                            span
                        }
                    } else {
                        let kind = if items.is_empty() {
                            ElementKind::Scope(items.clone())
                        } else {
                            ElementKind::Bundle(items.clone())
                        };

                        let span = self.span(start, span.end);

                        Element {
                            kind,
                            span
                        }
                    }
                }
                TokenKind::Punctuation(PunctuationKind::Comma) => {
                    self.next();

                    if let Some(separator) = separator {
                        if separator != PunctuationKind::Comma {
                            self.error(&ParseError::new(
                                ErrorKind::InconsistentSeparators,
                                span,
                            ));
                        }
                    } else {
                        separator = Some(PunctuationKind::Comma);
                    }
                }
                TokenKind::Punctuation(PunctuationKind::Semicolon) => {
                    self.next();

                    if let Some(separator) = separator {
                        if separator != PunctuationKind::Semicolon {
                            self.error(&ParseError::new(
                                ErrorKind::InconsistentSeparators,
                                span,
                            ));
                        }
                    } else {
                        separator = Some(PunctuationKind::Semicolon);
                    }
                }
                _ => {
                    let expr = self.parse_complex();

                    items.push(expr);
                }
            }
        }

        self.error(&ParseError::new(
            ErrorKind::UnclosedDelimiter(brace),
            self.span(start, err_end),
        ))
    }

    fn parse_bracketed(&mut self) -> Element {
        if let Some(token) = self.peek() {
            if token.kind != TokenKind::Punctuation(PunctuationKind::LeftBracket) {
                return self.error(&ParseError::new(
                    ErrorKind::ExpectedToken(TokenKind::Punctuation(PunctuationKind::LeftBracket)),
                    token.span.clone(),
                ));
            }
        }

        let brace = self.next().unwrap();

        let Token {
            span: Span { start, .. },
            ..
        } = brace;

        let err_end = start;
        let mut items = Vec::new();
        let mut separator = Option::<PunctuationKind>::None;

        while let Some(Token { kind, span }) = self.peek().cloned() {
            match kind {
                TokenKind::Punctuation(PunctuationKind::RightBracket) => {
                    self.next();

                    return if separator == Some(PunctuationKind::Semicolon) {
                        let kind = ElementKind::Series(items.clone());
                        let span = self.span(start, span.end);

                        Element {
                            kind,
                            span
                        }
                    } else {
                        let kind = ElementKind::Collection(items.clone());
                        let span = self.span(start, span.end);

                        Element {
                            kind,
                            span
                        }
                    }
                }
                TokenKind::Punctuation(PunctuationKind::Comma) => {
                    self.next();

                    if let Some(separator) = separator {
                        if separator != PunctuationKind::Comma {
                            self.error(&ParseError::new(
                                ErrorKind::InconsistentSeparators,
                                span,
                            ));
                        }
                    } else {
                        separator = Some(PunctuationKind::Comma);
                    }
                }
                TokenKind::Punctuation(PunctuationKind::Semicolon) => {
                    self.next();

                    if let Some(separator) = separator {
                        if separator != PunctuationKind::Semicolon {
                            self.error(&ParseError::new(
                                ErrorKind::InconsistentSeparators,
                                span,
                            ));
                        }
                    } else {
                        separator = Some(PunctuationKind::Semicolon);
                    }
                }
                _ => {
                    let expr = self.parse_complex();

                    items.push(expr);
                }
            }
        }

        self.error(&ParseError::new(
            ErrorKind::UnclosedDelimiter(brace),
            self.span(start, err_end),
        ))
    }

    fn parse_parenthesized(&mut self) -> Element {
        if let Some(token) = self.peek() {
            if token.kind != TokenKind::Punctuation(PunctuationKind::LeftParen) {
                return self.error(&ParseError::new(
                    ErrorKind::ExpectedToken(TokenKind::Punctuation(PunctuationKind::LeftParen)),
                    token.span.clone(),
                ));
            }
        }

        let brace = self.next().unwrap();

        let Token {
            span: Span { start, .. },
            ..
        } = brace;

        let err_end = start;
        let mut items = Vec::new();
        let mut separator = Option::<PunctuationKind>::None;

        while let Some(Token { kind, span }) = self.peek().cloned() {
            match kind {
                TokenKind::Punctuation(PunctuationKind::RightParen) => {
                    self.next();

                    return if separator == Some(PunctuationKind::Semicolon) {
                        let kind = ElementKind::Sequence(items.clone());
                        let span = self.span(start, span.end);

                        Element {
                            kind,
                            span
                        }
                    } else {
                        let kind = ElementKind::Group(items.clone());
                        let span = self.span(start, span.end);

                        Element {
                            kind,
                            span
                        }
                    }
                }
                TokenKind::Punctuation(PunctuationKind::Comma) => {
                    self.next();

                    if let Some(separator) = separator {
                        if separator != PunctuationKind::Comma {
                            self.error(&ParseError::new(
                                ErrorKind::InconsistentSeparators,
                                span,
                            ));
                        }
                    } else {
                        separator = Some(PunctuationKind::Comma);
                    }
                }
                TokenKind::Punctuation(PunctuationKind::Semicolon) => {
                    self.next();

                    if let Some(separator) = separator {
                        if separator != PunctuationKind::Semicolon {
                            self.error(&ParseError::new(
                                ErrorKind::InconsistentSeparators,
                                span,
                            ));
                        }
                    } else {
                        separator = Some(PunctuationKind::Semicolon);
                    }
                }
                _ => {
                    let expr = self.parse_complex();

                    items.push(expr);
                }
            }
        }

        self.error(&ParseError::new(
            ErrorKind::UnclosedDelimiter(brace),
            self.span(start, err_end),
        ))
    }
}
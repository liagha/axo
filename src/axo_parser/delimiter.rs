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
use crate::Peekable;

pub trait Delimiter {
    fn parse_delimited<F, R>(
        &mut self,
        opening: TokenKind,
        closing: TokenKind,
        separators: &[TokenKind],
        trailing: bool,
        function: F,
    ) -> (Vec<R>, Span)
    where
        R: Spanned + Clone,
        F: FnMut(&mut Parser) -> R;

    fn parse_braced<F>(&mut self, function: F) -> Element
    where
        F: FnMut(&mut Parser) -> Element;
    fn parse_bracketed<F>(&mut self, function: F) -> Element
    where
        F: FnMut(&mut Parser) -> Element;
    fn parse_parenthesized<F>(&mut self, function: F) -> Element
    where
        F: FnMut(&mut Parser) -> Element;
}

impl Delimiter for Parser {
    fn parse_delimited<F, R>(
        &mut self,
        opening: TokenKind,
        closing: TokenKind,
        separators: &[TokenKind],
        trailing: bool,
        mut function: F,
    ) -> (Vec<R>, Span)
    where
        R: Spanned + Clone,
        F: FnMut(&mut Parser) -> R,
    {
        let open = if let Some(open) = self.next().clone() {
            if open.kind == opening {
                open
            } else {
                let span = open.span.clone();

                self.error(&ParseError::new(
                    ErrorKind::InvalidDelimiter,
                    span,
                ));

                open
            }
        } else {
            let span = self.current_span();

            self.error(&ParseError::new(
                ErrorKind::InvalidDelimiter,
                span.clone(),
            ));

            return (Vec::new(), span)
        };

        let Span { start, .. } = open.span.clone();

        let mut items = Vec::new();
        let mut end = start.clone();

        while let Some(token) = self.peek().cloned() {
            match token.kind {
                kind if kind == closing => {
                    let close_token = self.next().unwrap();
                    let Span { end, .. } = close_token.span;

                    return (items, self.span(start, end));
                }

                kind if separators.contains(&kind) => {
                    end = token.span.end;
                    self.next();
                }

                _ => {
                    let item = function(self);
                    let item_start = item.span().start;

                    items.push(item.clone());

                    end = item.span().end;

                    if trailing {
                        if let Some(peek) = self.peek() {
                            if separators.contains(&peek.kind) {
                                end = token.span.end;
                                self.next();
                            } else if peek.kind != closing {
                                self.next();

                                self.error(&ParseError::new(
                                    ErrorKind::MissingSeparators(separators.into()),
                                    self.span(item_start, end.clone()),
                                ));

                                return (items, self.span(start, end));
                            }
                        }
                    }
                }
            }
        }

        self.error(&ParseError::new(
            ErrorKind::UnclosedDelimiter(open),
            self.span(start.clone(), end.clone()),
        ));

        (items, self.span(start, end))
    }

    fn parse_braced<F>(&mut self, mut function: F) -> Element
    where
        F: FnMut(&mut Parser) -> Element
    {
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
        } = brace.clone();

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
                    let element = function(self);

                    items.push(element);
                }
            }
        }

        self.error(&ParseError::new(
            ErrorKind::UnclosedDelimiter(brace.clone()),
            brace.span.clone(),
        ))
    }

    fn parse_bracketed<F>(&mut self, mut function: F) -> Element
    where
        F: FnMut(&mut Parser) -> Element
    {
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
        } = brace.clone();

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
                    let element = function(self);

                    items.push(element);
                }
            }
        }

        self.error(&ParseError::new(
            ErrorKind::UnclosedDelimiter(brace.clone()),
            brace.span.clone(),
        ))
    }

    fn parse_parenthesized<F>(&mut self, mut parser: F) -> Element
    where
        F: FnMut(&mut Parser) -> Element
    {
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
        } = brace.clone();

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
                        if items.len() == 1 {
                            items.pop().unwrap()
                        } else {
                            let kind = ElementKind::Group(items.clone());
                            let span = self.span(start, span.end);

                            Element {
                                kind,
                                span
                            }
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
                    let element = parser(self);

                    items.push(element);
                }
            }
        }

        self.error(&ParseError::new(
            ErrorKind::UnclosedDelimiter(brace.clone()),
            brace.span.clone(),
        ))
    }
}
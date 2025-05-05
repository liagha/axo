#![allow(dead_code)]

use {
    crate::{
        axo_lexer::{OperatorKind, PunctuationKind, Token, TokenKind},
        axo_parser::{error::ErrorKind, Element, ElementKind, ParseError, Primary, ItemKind},
        axo_span::{
            Span,
            position::Position,
        },
    },
    crate::Path,
};

#[derive(Clone)]
pub struct Parser {
    pub input: Vec<Token>,
    pub position: Position,
    pub output: Vec<Element>,
    pub errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, file: Path) -> Self {
        Parser {
            input: tokens,
            position: Position::new(file),
            output: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn error(&mut self, error: &ParseError) -> Element {
        self.errors.push(error.clone());

        let current = (self.position.line, self.position.column);

        Element {
            kind: ElementKind::Invalid(error.clone()),
            span: self.span(current, current),
        }
    }

    pub fn span(&self, start: (usize, usize), end: (usize, usize)) -> Span {
        Span {
            file: self.position.file.clone(),
            start,
            end,
        }
    }

    pub fn full_span(&self) -> Span {
        let end = if let Some(end) = self.input.last() {
            end.span.end
        } else {
            (1, 1)
        };

        Span {
            file: self.position.file.clone(),
            start: (1, 1),
            end,
        }
    }

    pub fn next(&mut self) -> Option<Token> {
        while self.position.index < self.input.len() {
            let token = self.input[self.position.index].clone();
            self.position.index += 1;

            match &token.kind {
                TokenKind::Punctuation(PunctuationKind::Newline) => {
                    self.position.line += 1;
                    self.position.column = 0;
                    continue;
                }
                TokenKind::Comment(_) => {
                    self.position.column += 1;

                    continue;
                }
                _ => {
                    self.position.column += 1;

                    return Some(token);
                }
            }
        }

        None
    }

    pub fn peek(&self) -> Option<&Token> {
        let mut current = self.position.index;

        while let Some(token) = self.input.get(current) {
            match token.kind {
                TokenKind::Punctuation(PunctuationKind::Newline) => {
                    current += 1;
                }
                TokenKind::Comment(_) => {
                    current += 1;
                }
                _ => {
                    return Some(token);
                }
            }
        }

        None
    }

    pub fn peek_with<R>(&self, handler: fn(&Parser) -> R) -> R {
        let parser = self.clone();

        handler(&parser)
    }

    pub fn peek_ahead(&self, forward: usize) -> Option<&Token> {
        let mut current = self.position.index + forward;

        while let Some(token) = self.input.get(current) {
            match token.kind {
                TokenKind::Punctuation(PunctuationKind::Newline) => {
                    current += 1;
                }
                TokenKind::Comment(_) => {
                    current += 1;
                }
                _ => {
                    return Some(token);
                }
            }
        }

        None
    }

    pub fn match_token(&mut self, expected: &TokenKind) -> bool {
        if let Some(token) = self.input.get(self.position.index) {
            if token.kind == TokenKind::Punctuation(PunctuationKind::Newline) {
                self.position.index += 1;
                self.position.line += 1;
                self.position.column = 0;

                return false;
            }

            if &token.kind == expected {
                self.next();

                return true;
            }
        }

        false
    }

    pub fn peek_is_any(&self, kinds: &[TokenKind]) -> bool {
        if let Some(token) = self.peek() {
            kinds.contains(&token.kind)
        } else {
            false
        }
    }

    pub fn match_any(&mut self, kinds: &[TokenKind]) -> bool {
        if let Some(token) = self.peek() {
            if kinds.contains(&token.kind) {
                self.next();
                return true;
            }
        }
        false
    }

    pub fn is_at_end(&self) -> bool {
        self.position.index >= self.input.len()
    }

    pub fn skip_until(&mut self, delimiters: &[TokenKind]) {
        while !self.is_at_end() {
            if let Some(token) = self.peek() {
                if delimiters.contains(&token.kind) {
                    break;
                }
                self.next();
            }
        }
    }

    pub fn parse_program(&mut self) -> Vec<Element> {
        // let start = (0, 0);

        let mut items = Vec::new();
        let mut separator = Option::<PunctuationKind>::None;

        while let Some(Token { kind, span }) = self.peek().cloned() {
            match kind {
                TokenKind::Punctuation(PunctuationKind::Comma) => {
                    self.next();

                    if let Some(separator) = separator {
                        if separator != PunctuationKind::Comma {
                            self.error(&ParseError::new(ErrorKind::InconsistentSeparators, span));
                        }
                    } else {
                        separator = Some(PunctuationKind::Comma);
                    }
                }
                TokenKind::Punctuation(PunctuationKind::Semicolon) => {
                    self.next();

                    if let Some(separator) = separator {
                        if separator != PunctuationKind::Semicolon {
                            self.error(&ParseError::new(ErrorKind::InconsistentSeparators, span));
                        }
                    } else {
                        separator = Some(PunctuationKind::Semicolon);
                    }
                }
                _ => {
                    let element = self.parse_complex();

                    items.push(element.clone());

                    self.output.push(element);
                }
            }
        }

        if separator == Some(PunctuationKind::Semicolon) {
            items
        } else {
            items
        }
    }
}

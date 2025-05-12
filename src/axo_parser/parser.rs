#![allow(dead_code)]

use {
    crate::{
        axo_lexer::{OperatorKind, PunctuationKind, Token, TokenKind},
        axo_parser::{error::ErrorKind, Element, ElementKind, ParseError, Primary, ItemKind},
        axo_data::peekable::Peekable,
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

impl Peekable<Token> for Parser {
    fn peek(&self) -> Option<&Token> {
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

    fn peek_ahead(&self, forward: usize) -> Option<&Token> {
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

    fn next(&mut self) -> Option<Token> {
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

    fn position(&self) -> Position {
        self.position.clone()
    }

    fn set_index(&mut self, index: usize) {
        self.position.index = index
    }

    fn set_line(&mut self, line: usize) {
        self.position.line = line
    }

    fn set_column(&mut self, column: usize) {
        self.position.column = column
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }
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

        let current = self.position.clone();

        Element {
            kind: ElementKind::Invalid(error.clone()),
            span: self.span(current.clone(), current),
        }
    }
    
    pub fn current_span(&self) -> Span {
        self.point_span(self.position.clone())
    }
    
    pub fn point_span(&self, point: Position) -> Span {
        Span {
            start: point.clone(),
            end: point,
        }
    }

    pub fn span(&self, start: Position, end: Position) -> Span {
        Span {
            start,
            end,
        }
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

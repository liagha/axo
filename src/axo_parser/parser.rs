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
    pub index: usize,
    pub position: Position,
    pub output: Vec<Element>,
    pub errors: Vec<ParseError>,
}

impl Peekable<Token> for Parser {
    fn peek_ahead(&self, forward: usize) -> Option<&Token> {
        let mut current = self.index + forward;

        while let Some(token) = self.input.get(current) {
            match token.kind {
                TokenKind::Punctuation(PunctuationKind::Newline) | TokenKind::Punctuation(PunctuationKind::Space) => {
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

    fn peek_behind(&self, n: usize) -> Option<&Token> {
        let mut current = self.index;

        if current < n {
            return None;
        }
        
        while let Some(token) = self.input.get(current - n) {
            if current < n {
                return None;
            }

            match token.kind {
                TokenKind::Punctuation(PunctuationKind::Newline) | TokenKind::Punctuation(PunctuationKind::Space) => {
                    current -= n;
                }
                TokenKind::Comment(_) => {
                    current -= n;
                }
                _ => {
                    return Some(token);
                }
            }
        }

        None
    }

    fn restore(&mut self) {
        self.restore_position(
            Position {
                line: 1,
                column: 1,
                file: self.position.file.clone()
            }
        )
    }

    fn next(&mut self) -> Option<Token> {
        while self.index < self.input.len() {
            let token = self.input[self.index].clone();
            self.index += 1;

            match &token.kind {
                TokenKind::Punctuation(PunctuationKind::Newline) => {
                    self.position.line += 1;
                    self.position.column = 0;
                    
                    continue;
                }
                TokenKind::Punctuation(PunctuationKind::Space) => {
                    self.position.column += 1;
                    
                    continue;
                }
                TokenKind::Comment(_) => {
                    self.position.column += 1;

                    continue;
                }
                _ => {
                    self.position = token.span.end.clone();

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
        self.index = index
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
            index: 0,
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
        Span::point(self.position.clone())
    }
    
    pub fn span(&self, start: Position, end: Position) -> Span {
        Span {
            start,
            end,
        }
    }
    
    pub fn match_token(&mut self, token: &TokenKind) -> bool {
        if let Some(peek) = self.peek() {
            if &peek.kind == token {
                self.next();
                
                true
            } else { 
                false
            }
        } else {
            false
        }
    }

    pub fn parse(&mut self) -> Vec<Element> {
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
                TokenKind::Punctuation(PunctuationKind::SemiColon) => {
                    self.next();

                    if let Some(separator) = separator {
                        if separator != PunctuationKind::SemiColon {
                            self.error(&ParseError::new(ErrorKind::InconsistentSeparators, span));
                        }
                    } else {
                        separator = Some(PunctuationKind::SemiColon);
                    }
                }
                _ => {
                    let element = self.parse_complex();

                    items.push(element.clone());

                    self.output.push(element);
                }
            }
        }

        if separator == Some(PunctuationKind::SemiColon) {
            items
        } else {
            items
        }
    }
}

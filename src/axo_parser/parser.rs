#![allow(dead_code)]

use {
    crate::{
        axo_lexer::{OperatorKind, PunctuationKind, Token, TokenKind},
        axo_parser::{error::ErrorKind, Element, ElementKind, ParseError, ItemKind},
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
        let mut current = self.index;
        let mut found = 0;

        while current < self.input.len() {
            let token = &self.input[current];

            if self.should_skip_token(&token.kind) {
                current += 1;
                continue;
            }

            if found == forward {
                return Some(token);
            }

            found += 1;
            current += 1;
        }

        None
    }

    fn peek_behind(&self, backward: usize) -> Option<&Token> {
        if self.index == 0 {
            return None;
        }

        let mut current = self.index - 1;
        let mut found = 0;

        loop {
            if current >= self.input.len() {
                if current == 0 {
                    break;
                }
                current -= 1;
                continue;
            }

            let token = &self.input[current];

            if self.should_skip_token(&token.kind) {
                if current == 0 {
                    break;
                }
                current -= 1;
                continue;
            }

            if found == backward {
                return Some(token);
            }

            found += 1;

            if current == 0 {
                break;
            }
            current -= 1;
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

            if self.should_skip_token(&token.kind) {
                self.update_position_for_skipped(&token);
                continue;
            }

            self.position = token.span.end.clone();
            return Some(token);
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
    fn should_skip_token(&self, kind: &TokenKind) -> bool {
        if let TokenKind::Punctuation(punctuation) = kind {
            if let PunctuationKind::Newline = punctuation {
                return true;
            }
            if let PunctuationKind::Space = punctuation {
                return true;
            }
        }

        if let TokenKind::Comment(_) = kind {
            return true;
        }

        false
    }

    fn update_position_for_skipped(&mut self, token: &Token) {
        if let TokenKind::Punctuation(PunctuationKind::Newline) = &token.kind {
            self.position.line += 1;
            self.position.column = 1;
            return;
        }

        if let TokenKind::Punctuation(PunctuationKind::Space) = &token.kind {
            self.position.column += 1;
            return;
        }

        if let TokenKind::Comment(_) = &token.kind {
            self.position.column += token.span.end.column - token.span.start.column;
            return;
        }
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
}

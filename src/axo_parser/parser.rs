#![allow(dead_code)]

use std::path::PathBuf;
use crate::axo_lexer::{OperatorKind, PunctuationKind, Span, Token, TokenKind};
use crate::axo_parser::error::{ParseError};
use crate::axo_parser::{Expr, ExprKind, Primary};
use crate::axo_parser::state::{Position, Context};

pub struct Parser {
    tokens: Vec<Token>,
    pub state: Vec<Context>,
    pub file: PathBuf,
    pub position: usize,
    pub line: usize,
    pub column: usize,
    pub expressions: Vec<Expr>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, file: PathBuf) -> Self {
        Parser {
            file,
            tokens,
            state: Vec::new(),
            position: 0,
            line: 1,
            column: 1,
            expressions: Vec::new(),
        }
    }

    pub fn enter(&mut self, context: Context) {
        self.state.push(context);
    }

    pub fn exit(&mut self) {
        self.state.pop();
    }

    pub fn switch(&mut self, context: Context) {
        self.exit();

        self.enter(context);
    }

    pub(crate) fn current_context(&self) -> Context {
        self.state.last().cloned().unwrap_or(Context::Default)
    }

    pub fn span(&self, start: (usize, usize), end: (usize, usize)) -> Span {
        Span {
            file: self.file.clone(),
            start,
            end
        }
    }

    pub fn next(&mut self) -> Option<Token> {
        while self.position < self.tokens.len() {
            let token = self.tokens[self.position].clone();
            self.position += 1;

            match &token.kind {
                TokenKind::Punctuation(PunctuationKind::Newline) => {
                    self.line += 1;
                    self.column = 0;
                    continue;
                }
                TokenKind::Comment(_) => {
                    self.column += 1;

                    continue;
                }
                _ => {
                    self.column += 1;

                    return Some(token);
                }
            }
        }

        None
    }

    pub fn peek(&self) -> Option<&Token> {
        let mut current = self.position;

        while let Some(token) = self.tokens.get(current) {
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
        if let Some(token) = self.tokens.get(self.position) {
            if token.kind == TokenKind::Punctuation(PunctuationKind::Newline) {
                self.position += 1;
                self.line += 1;
                self.column = 0;

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
        self.position >= self.tokens.len()
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

    pub fn parse_program(&mut self) -> Result<Vec<Expr>, ParseError> {
        self.enter(Context::Program);

        let mut statements = Vec::new();

        while let Some(token) = self.peek() {
            if token.kind == TokenKind::EOF {
                break;
            } else if token.kind == TokenKind::Punctuation(PunctuationKind::Semicolon) {
                self.next();
            }

            let stmt = self.parse_statement()?;

            self.expressions.push(stmt.clone());
            statements.push(stmt);
        }

        self.exit();

        Ok(statements)
    }
}

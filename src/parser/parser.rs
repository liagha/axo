#![allow(dead_code)]

use std::path::PathBuf;
use crate::lexer::{OperatorKind, PunctuationKind, Span, Token, TokenKind};
use crate::parser::error::{ParseError, SyntaxPosition, SyntaxType};
use crate::parser::{Expr, ExprKind, Primary};

pub struct Parser {
    tokens: Vec<Token>,
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
            position: 0,
            line: 1,
            column: 1,
            expressions: Vec::new(),
        }
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

    pub fn expect(
        &mut self,
        expected: TokenKind,
        context: SyntaxType,
    ) -> Result<Token, ParseError> {
        if let Some(token) = self.next() {
            if token.kind == expected {
                Ok(token)
            } else {
                Err(ParseError::ExpectedTokenNotFound(
                    expected,
                    SyntaxPosition::Before,
                    context,
                ))
            }
        } else {
            Err(ParseError::UnexpectedEndOfFile)
        }
    }

    pub fn expect_any(
        &mut self,
        expected: &[TokenKind],
        context: SyntaxType,
    ) -> Result<Token, ParseError> {
        if let Some(token) = self.next() {
            if expected.contains(&token.kind) {
                Ok(token)
            } else {
                Err(ParseError::ExpectedTokenNotFound(
                    expected[0].clone(),
                    SyntaxPosition::Before,
                    context,
                ))
            }
        } else {
            Err(ParseError::UnexpectedEndOfFile)
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }

    pub fn current_span(&self) -> Option<Span> {
        self.peek().map(|t| t.span.clone())
    }

    pub fn previous_span(&self) -> Option<Span> {
        if self.position > 0 {
            self.tokens.get(self.position - 1).map(|t| t.span.clone())
        } else {
            None
        }
    }

    pub fn unexpected_token(&self, token: Token, context: SyntaxType) -> ParseError {
        ParseError::UnexpectedToken(token, SyntaxPosition::Inside, context)
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

    pub fn next_is_punct(&self, punct: PunctuationKind) -> bool {
        if let Some(token) = self.peek() {
            if let TokenKind::Punctuation(kind) = &token.kind {
                return kind == &punct;
            }
        }
        false
    }

    pub fn next_is_op(&self, op: OperatorKind) -> bool {
        if let Some(token) = self.peek() {
            if let TokenKind::Operator(kind) = &token.kind {
                return kind == &op;
            }
        }
        false
    }

    pub fn next_is_ident(&self, text: &str) -> bool {
        if let Some(token) = self.peek() {
            if let TokenKind::Identifier(ident) = &token.kind {
                return ident == text;
            }
        }
        false
    }

    pub fn parse_comma_separated<T, F>(
        &mut self,
        parse_fn: F,
        terminator: TokenKind,
        context: SyntaxType,
    ) -> Result<Vec<T>, ParseError>
    where
        F: Fn(&mut Self) -> Result<T, ParseError>,
    {
        let mut items = Vec::new();

        while !self.is_at_end() && !self.peek_is_any(&[terminator.clone()]) {
            let item = parse_fn(self)?;
            items.push(item);

            if self.next_is_punct(PunctuationKind::Comma) {
                self.next();
            } else if !self.peek_is_any(&[terminator.clone()]) {
                return Err(ParseError::ExpectedSeparator(
                    TokenKind::Punctuation(PunctuationKind::Comma),
                    context,
                ));
            }
        }

        Ok(items)
    }

    pub fn parse_delimited<T, F>(
        &mut self,
        opener: TokenKind,
        closer: TokenKind,
        parse_fn: F,
        context: SyntaxType,
    ) -> Result<T, ParseError>
    where
        F: Fn(&mut Self) -> Result<T, ParseError>,
    {
        self.expect(opener, context.clone())?;
        let result = parse_fn(self)?;
        self.expect(closer, context)?;
        Ok(result)
    }

    pub fn parse_program(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut statements = Vec::new();

        while let Some(token) = self.peek() {
            if token.kind == TokenKind::EOF {
                break;
            }

            let stmt = self.parse_statement()?;

            self.expressions.push(stmt.clone());
            statements.push(stmt);
        }

        Ok(statements)
    }
}

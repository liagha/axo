use crate::axo_lexer::error::{Error, ErrorKind};
use crate::axo_lexer::TokenKind;
use crate::axo_lexer::{Span, Token};
use crate::axo_lexer::number::NumberLexer;
use std::path::PathBuf;
use crate::axo_lexer::handler::Handler;
use crate::axo_lexer::literal::LiteralLexer;
use crate::axo_lexer::operator::OperatorLexer;
use crate::axo_lexer::punctuation::PunctuationLexer;
use crate::axo_lexer::symbol::SymbolLexer;

pub struct Lexer {
    file: PathBuf,
    chars: Vec<char>,
    position: usize,
    pub line: usize,
    pub column: usize,
    pub tokens: Vec<Token>,
}

impl Lexer {
    pub fn new(input: String, file: PathBuf) -> Lexer {
        let chars: Vec<char> = input.chars().collect();

        Lexer {
            file,
            chars,
            position: 0,
            line: 1,
            column: 0,
            tokens: Vec::new(),
        }
    }

    pub fn peek(&self) -> Option<char> {
        if self.position < self.chars.len() {
            Some(self.chars[self.position])
        } else {
            None
        }
    }

    pub fn peek_ahead(&self, n: usize) -> Option<char> {
        let pos = self.position + n;

        if pos < self.chars.len() {
            Some(self.chars[pos])
        } else {
            None
        }
    }

    pub fn next(&mut self) -> Option<char> {
        if self.position < self.chars.len() {
            let ch = self.chars[self.position];

            self.position += 1;

            if ch == '\n' {
                self.line += 1;
                self.column = 0;
            } else {
                self.column += 1;
            }

            Some(ch)
        } else {
            None
        }
    }

    pub fn create_span(&self, start: (usize, usize), end: (usize, usize)) -> Span {
        Span {
            start,
            end,
            file: self.file.clone(),
        }
    }

    pub fn push_token(&mut self, kind: TokenKind, span: Span) {
        self.tokens.push(Token { kind, span });
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, Error> {
        while let Some(ch) = self.peek() {
            match ch {
                ch if ch.is_whitespace() && ch != '\n' => {
                    self.next();

                    continue
                },

                ch if ch.is_digit(10) => self.handle_number()?,

                ch if ch.is_alphabetic() || ch == '_' => self.handle_identifier()?,

                '.' => {
                    if let Some(ch) = self.peek_ahead(1) {
                        if ch.is_digit(10) {
                            self.handle_number()?;
                        } else {
                            self.handle_operator()?;
                        }
                    }
                },

                '\'' => self.handle_character()?,

                '"' => self.handle_string()?,

                '/' => self.handle_comment()?,

                ch if ch.is_operator() => self.handle_operator()?,

                ch if ch.is_punctuation() => self.handle_punctuation(),

                _ => {
                    self.next();

                    let start = (self.line, self.column);
                    let end = (self.line, self.column);
                    let span = self.create_span(start, end);

                    self.push_token(TokenKind::Invalid(ch.to_string()), span.clone());

                    return Err(Error::new(ErrorKind::InvalidChar, span));
                }
            }
        }

        let file_end = (self.line, self.column);
        let span = self.create_span(file_end, file_end);

        self.push_token(TokenKind::EOF, span);

        Ok(self.tokens.clone())
    }
}
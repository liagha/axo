use crate::axo_lexer::error::{CharParseError, LexError};
use crate::axo_lexer::{Lexer, TokenKind};
use crate::axo_lexer::symbol::SymbolLexer;

pub trait LiteralLexer {
    fn handle_character(&mut self) -> Result<(), LexError>;
    fn handle_string(&mut self) -> Result<(), LexError>;
}

impl LiteralLexer for Lexer {
    fn handle_character(&mut self) -> Result<(), LexError> {
        self.next();

        let mut content = Vec::new();
        let mut closed = false;
        let mut is_escaped = false;

        let start = (self.line, self.column);

        while let Some(next_ch) = self.next() {
            if is_escaped {
                match self.handle_escape_sequence(false) {
                    Ok(escaped_char) => content.push(escaped_char),
                    Err(e) => return Err(e),
                }
                is_escaped = false;
            } else if next_ch == '\\' {
                is_escaped = true;
            } else if next_ch == '\'' {
                let end = (self.line, self.column);
                let span = self.create_span(start, end);

                if content.len() == 1 {
                    let ch = content[0];
                    self.push_token(TokenKind::Char(ch), span);
                    closed = true;
                    break;
                } else {
                    return Err(LexError::CharParseError(CharParseError::InvalidCharLiteral));
                }
            } else {
                content.push(next_ch);
            }
        }

        if !closed {
            let end = (self.line, self.column);
            let span = self.create_span(start, end);

            let content_string: String = content.into_iter().collect();
            self.push_token(TokenKind::Invalid(format!("'{}", content_string)), span);

            return Err(LexError::UnClosedChar);
        }

        Ok(())
    }

    fn handle_string(&mut self) -> Result<(), LexError> {
        self.next();

        let mut content = Vec::new();
        let mut closed = false;
        let start = (self.line, self.column);

        let mut is_escaped = false;

        while let Some(next_ch) = self.next() {
            if is_escaped {
                match self.handle_escape_sequence(true) {
                    Ok(escaped_char) => content.push(escaped_char),
                    Err(e) => return Err(e),
                }
                is_escaped = false;
            } else if next_ch == '\\' {
                is_escaped = true;
            } else if next_ch == '"' {
                let end = (self.line, self.column);
                let span = self.create_span(start, end);

                let content_string: String = content.clone().into_iter().collect();
                self.push_token(TokenKind::Str(content_string), span);

                closed = true;
                break;
            } else {
                content.push(next_ch);
            }
        }

        if !closed {
            let end = (self.line, self.column);
            let span = self.create_span(start, end);

            let content_string: String = content.clone().into_iter().collect();

            self.push_token(TokenKind::Invalid(format!("\"{}\"", content_string)), span);

            return Err(LexError::UnClosedString);
        }

        Ok(())
    }
}
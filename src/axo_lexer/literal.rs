use crate::axo_lexer::error::{CharParseError, Error, ErrorKind};
use crate::axo_lexer::{Lexer, TokenKind};
use crate::axo_lexer::symbol::SymbolLexer;

pub trait LiteralLexer {
    fn handle_character(&mut self) -> Result<(), Error>;
    fn handle_string(&mut self) -> Result<(), Error>;
}

impl LiteralLexer for Lexer {
    fn handle_character(&mut self) -> Result<(), Error> {
        self.next(); // Consume opening quote

        let start = (self.line, self.column);
        let mut content = Vec::new();
        let mut closed = false;
        let mut is_escaped = false;

        while let Some(next_ch) = self.peek() {
            if next_ch == '\'' && !is_escaped {
                self.next(); // Consume closing quote
                closed = true;
                break;
            }

            self.next(); // Consume character

            if is_escaped {
                let escape_start = (self.line, self.column);
                match self.handle_escape_sequence(false) {
                    Ok(escaped_char) => content.push(escaped_char),
                    Err(Error { kind, .. }) => {
                        let escape_end = (self.line, self.column);
                        let escape_span = self.create_span(escape_start, escape_end);
                        return Err(Error::new(kind, escape_span));
                    }
                }
                is_escaped = false;
            } else if next_ch == '\\' {
                is_escaped = true;
            } else {
                content.push(next_ch);
            }
        }

        let end = (self.line, self.column);
        let span = self.create_span(start, end);

        if !closed {
            return Err(Error::new(ErrorKind::UnClosedChar, span));
        }

        // Validate character literal
        match content.len() {
            0 => {
                Err(Error::new(ErrorKind::CharParseError(CharParseError::EmptyCharLiteral), span))
            }
            1 => {
                let ch = content[0];
                self.push_token(TokenKind::Char(ch), span);
                Ok(())
            }
            _ => {
                Err(Error::new(ErrorKind::CharParseError(CharParseError::InvalidCharLiteral), span))
            }
        }
    }

    fn handle_string(&mut self) -> Result<(), Error> {
        self.next(); // Consume opening quote

        let start = (self.line, self.column);
        let mut content = Vec::new();
        let mut closed = false;
        let mut is_escaped = false;

        while let Some(next_ch) = self.peek() {
            if next_ch == '"' && !is_escaped {
                self.next(); // Consume closing quote
                closed = true;
                break;
            }

            self.next(); // Consume character

            if is_escaped {
                let escape_start = (self.line, self.column);
                match self.handle_escape_sequence(true) {
                    Ok(escaped_char) => content.push(escaped_char),
                    Err(Error { kind, .. }) => {
                        let escape_end = (self.line, self.column);
                        let escape_span = self.create_span(escape_start, escape_end);
                        return Err(Error::new(kind, escape_span));
                    }
                }
                is_escaped = false;
            } else if next_ch == '\\' {
                is_escaped = true;
            } else {
                content.push(next_ch);
            }
        }

        let end = (self.line, self.column);
        let span = self.create_span(start, end);

        if !closed {
            return Err(Error::new(ErrorKind::UnClosedString, span));
        }

        let content_string: String = content.into_iter().collect();
        self.push_token(TokenKind::Str(content_string), span);
        Ok(())
    }
}
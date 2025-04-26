use {
    crate::{
        axo_lexer::{
            LexError, Lexer, TokenKind,
            symbol::SymbolLexer,
            error::{
                CharParseError, ErrorKind
            },
        }
    }
};

pub trait LiteralLexer {
    fn handle_character(&mut self) -> Result<(), LexError>;
    fn handle_string(&mut self) -> Result<(), LexError>;
}

impl LiteralLexer for Lexer {
    fn handle_character(&mut self) -> Result<(), LexError> {
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
                match self.handle_escape(false) {
                    Ok(escaped_char) => content.push(escaped_char),
                    Err(LexError { kind, .. }) => {
                        let escape_end = (self.line, self.column);
                        let escape_span = self.create_span(escape_start, escape_end);
                        return Err(LexError::new(kind, escape_span));
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
            return Err(LexError::new(ErrorKind::UnterminatedChar, span));
        }

        // Validate character literal
        match content.len() {
            0 => {
                Err(LexError::new(ErrorKind::CharParseError(CharParseError::EmptyCharLiteral), span))
            }
            1 => {
                let ch = content[0];
                self.push_token(TokenKind::Character(ch), span);
                Ok(())
            }
            _ => {
                Err(LexError::new(ErrorKind::CharParseError(CharParseError::InvalidCharLiteral), span))
            }
        }
    }

    fn handle_string(&mut self) -> Result<(), LexError> {
        let quote_char = self.next().unwrap(); // Consume opening quote
        let start = (self.line, self.column);
        let mut content = Vec::new();
        let mut closed = false;
        let mut is_escaped = false;
        let is_multiline = quote_char == '`';

        while let Some(next_ch) = self.peek() {
            self.next();

            if is_escaped {
                let escape_start = (self.line, self.column);
                match self.handle_escape(!is_multiline) {
                    Ok(escaped_char) => content.push(escaped_char),
                    Err(LexError { kind, .. }) => {
                        let escape_end = (self.line, self.column);
                        let escape_span = self.create_span(escape_start, escape_end);
                        return Err(LexError::new(kind, escape_span));
                    }
                }
                is_escaped = false;
            } else if next_ch == '\\' && !is_multiline {
                is_escaped = true;
            } else {
                content.push(next_ch);
            }

            if next_ch == quote_char && !is_escaped {
                self.next();
                closed = true;
                break;
            }

            if !is_multiline && (next_ch == '\n' || next_ch == '\r') && !is_escaped {
                let end = (self.line, self.column);
                let span = self.create_span(start, end);
                return Err(LexError::new(ErrorKind::UnterminatedDoubleQuoteString, span));
            }
        }

        let end = (self.line, self.column);
        let span = self.create_span(start, end);

        if !closed {
            return Err(LexError::new(
                ErrorKind::UnterminatedBackTickString,
                span,
            ));
        }

        let content_string: String = content.into_iter().collect();
        self.push_token(
            TokenKind::String(content_string),
            span,
        );
        Ok(())
    }
}
use crate::axo_lexer::error::{CharParseError, LexError};
use crate::axo_lexer::{Lexer, OperatorKind, PunctuationKind, TokenKind};

pub trait Handler {
    fn handle_character(&mut self) -> Result<(), LexError>;
    fn handle_string(&mut self) -> Result<(), LexError>;
    fn handle_identifier(&mut self) -> Result<(), LexError>;
    fn handle_comment(&mut self) -> Result<(), LexError>;
    fn handle_operator(&mut self) -> Result<(), LexError>;
    fn handle_punctuation(&mut self);
    fn handle_escape_sequence(&mut self, is_string: bool) -> Result<char, LexError>;
}

impl Handler for Lexer {
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

    fn handle_identifier(&mut self) -> Result<(), LexError> {
        let ch = self.next().unwrap();

        let mut name = ch.to_string();
        let start = (self.line, self.column);

        while let Some(next_ch) = self.peek() {
            if next_ch.is_alphanumeric() || next_ch == '_' {
                name.push(self.next().unwrap());
            } else {
                break;
            }
        }

        let end = (self.line, self.column);
        let span = self.create_span(start, end);

        match TokenKind::from_str(name.as_str()) {
            Some(token_kind) => self.push_token(token_kind, span),
            _ => self.push_token(TokenKind::Identifier(name), span),
        }

        Ok(())
    }

    fn handle_comment(&mut self) -> Result<(), LexError> {
        self.next();

        let start = (self.line, self.column);

        if let Some(next_ch) = self.peek() {
            match next_ch {
                '/' => {
                    let mut comment = Vec::new();
                    self.next();

                    while let Some(next_ch) = self.peek() {
                        if next_ch == '\n' {
                            break;
                        }

                        comment.push(next_ch);
                        self.next();
                    }

                    let end = (self.line, self.column);
                    let span = self.create_span(start, end);

                    let comment_string: String = comment.into_iter().collect();
                    self.push_token(TokenKind::Comment(comment_string), span);
                }
                '*' => {
                    let mut comment = Vec::new();
                    self.next();

                    let mut closed = false;
                    let mut last_char = '*';

                    while let Some(next_ch) = self.next() {
                        if last_char == '*' && next_ch == '/' {
                            closed = true;
                            if !comment.is_empty() {
                                comment.pop(); // Remove the last '*'
                            }
                            break;
                        }

                        comment.push(next_ch);

                        last_char = next_ch;
                    }

                    let end = (self.line, self.column);
                    let span = self.create_span(start, end);

                    let comment_string: String = comment.into_iter().collect();
                    if closed {
                        self.push_token(TokenKind::Comment(comment_string), span);
                    } else {
                        self.push_token(TokenKind::Invalid(comment_string), span);
                        return Err(LexError::UnClosedComment);
                    }
                }
                _ => {
                    let end = (self.line, self.column);
                    let span = self.create_span(start, end);

                    self.push_token(TokenKind::Operator(OperatorKind::Slash), span);
                }
            }
        } else {
            let end = (self.line, self.column);
            let span = self.create_span(start, end);

            self.push_token(TokenKind::Operator(OperatorKind::Slash), span);
        }

        Ok(())
    }

    fn handle_operator(&mut self) -> Result<(), LexError> {
        let mut operator = Vec::new();

        let ch = self.next().unwrap();

        operator.push(ch);

        let start = (self.line, self.column);

        while let Some(next_ch) = self.peek() {
            if OperatorKind::is_operator(next_ch) {
                operator.push(self.next().unwrap());
            } else {
                break;
            }
        }

        let end = (self.line, self.column);
        let span = self.create_span(start, end);

        let operator_string: String = operator.iter().collect();
        if OperatorKind::Unknown != OperatorKind::from_str(&operator_string) {
            let op = OperatorKind::from_str(&operator_string);
            self.push_token(TokenKind::Operator(op), span);
        } else {
            for (i, c) in operator.iter().enumerate() {
                let single_char_span = self.create_span(
                    (self.line, self.column + i + 1),
                    (self.line, self.column + i + 2),
                );
                self.push_token(
                    TokenKind::Operator(OperatorKind::from_str(c.to_string().as_str())),
                    single_char_span,
                );
            }
        }

        Ok(())
    }

    fn handle_punctuation(&mut self) {
        let ch = self.next().unwrap();

        let start = (self.line, self.column);
        let end = (self.line, self.column);
        let span = self.create_span(start, end);

        self.push_token(TokenKind::Punctuation(PunctuationKind::from_char(&ch)), span);
    }

    fn handle_escape_sequence(&mut self, is_string: bool) -> Result<char, LexError> {
        let error_type = if is_string {
            |err| LexError::StringParseError(err)
        } else {
            |err| LexError::CharParseError(err)
        };

        if let Some(next_ch) = self.next() {
            match next_ch {
                'n' => Ok('\n'),
                'r' => Ok('\r'),
                't' => Ok('\t'),
                '\\' => Ok('\\'),
                '\'' => Ok('\''),
                '"' => Ok('"'),
                '0' => Ok('\0'),
                'x' => {
                    let mut hex = String::new();
                    for _ in 0..2 {
                        if let Some(next_hex) = self.peek() {
                            if next_hex.is_digit(16) {
                                hex.push(self.next().unwrap());
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }

                    if let Ok(hex_value) = u32::from_str_radix(&hex, 16) {
                        if let Some(ch) = std::char::from_u32(hex_value) {
                            Ok(ch)
                        } else {
                            Err(error_type(CharParseError::InvalidEscapeSequence))
                        }
                    } else {
                        Err(error_type(CharParseError::InvalidEscapeSequence))
                    }
                }
                'u' => {
                    if self.peek() == Some('{') {
                        self.next();

                        let mut hex = String::new();
                        let mut closed_brace = false;

                        for _ in 0..6 {
                            if let Some(next_hex) = self.peek() {
                                if next_hex.is_digit(16) {
                                    hex.push(self.next().unwrap());
                                } else if next_hex == '}' {
                                    self.next();
                                    closed_brace = true;
                                    break;
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }

                        if !closed_brace {
                            return Err(error_type(CharParseError::UnClosedEscapeSequence));
                        }

                        if let Ok(hex_value) = u32::from_str_radix(&hex, 16) {
                            if let Some(ch) = std::char::from_u32(hex_value) {
                                Ok(ch)
                            } else {
                                Err(error_type(CharParseError::InvalidEscapeSequence))
                            }
                        } else {
                            Err(error_type(CharParseError::InvalidEscapeSequence))
                        }
                    } else {
                        Err(error_type(CharParseError::InvalidEscapeSequence))
                    }
                }
                _ => Ok(next_ch),
            }
        } else {
            Err(error_type(CharParseError::InvalidEscapeSequence))
        }
    }
}
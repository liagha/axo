use crate::axo_data::peekable::Peekable;
use crate::axo_lexer::error::{CharParseError, ErrorKind};
use crate::axo_lexer::operator::OperatorLexer;
use crate::axo_lexer::{LexError, Lexer, TokenKind};
use crate::axo_lexer::punctuation::PunctuationLexer;
extern crate alloc;

pub trait SymbolLexer {
    fn handle_operator(&mut self);
    fn handle_punctuation(&mut self);
    fn handle_escape(&mut self, is_string: bool) -> Result<char, LexError>;
}

impl SymbolLexer for Lexer {
    fn handle_operator(&mut self) {
        let mut operator = Vec::new();

        let ch = self.next().unwrap();

        operator.push(ch);

        let start = (self.position.line, self.position.column);

        while let Some(next_ch) = self.peek() {
            if next_ch.is_operator() {
                operator.push(self.next().unwrap());
            } else {
                break;
            }
        }

        let end = (self.position.line, self.position.column);
        let span = self.create_span(start, end);

        let operator = operator.iter().collect::<String>().to_operator();

        self.push_token(TokenKind::Operator(operator), span);
    }

    fn handle_punctuation(&mut self) {
        let ch = self.next().unwrap();

        let start = (self.position.line, self.position.column);
        let end = (self.position.line, self.position.column);
        let span = self.create_span(start, end);

        self.push_token(TokenKind::Punctuation(ch.to_punctuation()), span);
    }

    fn handle_escape(&mut self, is_string: bool) -> Result<char, LexError> {
        let start = (self.position.line, self.position.column);

        let error_type = if is_string {
            |err| ErrorKind::StringParseError(err)
        } else {
            |err| ErrorKind::CharParseError(err)
        };

        if let Some(next_ch) = self.next() {
            match next_ch {
                'n' => Ok('\n'),
                'r' => Ok('\r'),
                't' => Ok('\t'),
                '\\' => Ok('\\'),
                '\'' => Ok('\''),
                '"' => Ok('\"'),
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

                    let end = (self.position.line, self.position.column);

                    if let Ok(hex_value) = u32::from_str_radix(&hex, 16) {
                        if let Some(ch) = crate::char::from_u32(hex_value) {
                            Ok(ch)
                        } else {
                            Err(LexError::new(error_type(CharParseError::InvalidEscapeSequence), self.create_span(start, end)))
                        }
                    } else {
                        Err(LexError::new(error_type(CharParseError::InvalidEscapeSequence), self.create_span(start, end)))
                    }
                }
                'u' => {
                    if self.peek() == Some(&'{') {
                        self.next();

                        let mut hex = String::new();
                        let mut closed_brace = false;

                        for _ in 0..6 {
                            if let Some(next_hex) = self.peek().cloned() {
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

                        let end = (self.position.line, self.position.column);

                        if !closed_brace {
                            return Err(LexError::new(error_type(CharParseError::UnterminatedEscapeSequence), self.create_span(start, end)));
                        }

                        if let Ok(hex_value) = u32::from_str_radix(&hex, 16) {
                            if let Some(ch) = crate::char::from_u32(hex_value) {
                                Ok(ch)
                            } else {
                                Err(LexError::new(error_type(CharParseError::InvalidEscapeSequence), self.create_span(start, end)))
                            }
                        } else {
                            Err(LexError::new(error_type(CharParseError::InvalidEscapeSequence), self.create_span(start, end)))
                        }
                    } else {
                        let end = (self.position.line, self.position.column);

                        Err(LexError::new(error_type(CharParseError::InvalidEscapeSequence), self.create_span(start, end)))
                    }
                }
                _ => Ok(next_ch),
            }
        } else {
            let end = (self.position.line, self.position.column);

            Err(LexError::new(error_type(CharParseError::InvalidEscapeSequence), self.create_span(start, end)))
        }
    }
}
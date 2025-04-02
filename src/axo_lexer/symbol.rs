use crate::axo_lexer::error::{CharParseError, LexError};
use crate::axo_lexer::operator::OperatorLexer;
use crate::axo_lexer::{Lexer, OperatorKind, TokenKind};
use crate::axo_lexer::punctuation::PunctuationLexer;

pub trait SymbolLexer {
    fn handle_operator(&mut self) -> Result<(), LexError>;
    fn handle_punctuation(&mut self);
    fn handle_escape_sequence(&mut self, is_string: bool) -> Result<char, LexError>;
}

impl SymbolLexer for Lexer {
    fn handle_operator(&mut self) -> Result<(), LexError> {
        let mut operator = Vec::new();

        let ch = self.next().unwrap();

        operator.push(ch);

        let start = (self.line, self.column);

        while let Some(next_ch) = self.peek() {
            if next_ch.is_operator() {
                operator.push(self.next().unwrap());
            } else {
                break;
            }
        }

        let end = (self.line, self.column);
        let span = self.create_span(start, end);

        let operator_string: String = operator.iter().collect();
        if OperatorKind::Unknown != operator_string.to_operator() {
            let op = operator_string.to_operator();
            self.push_token(TokenKind::Operator(op), span);
        } else {
            for (i, c) in operator.iter().enumerate() {
                let single_char_span = self.create_span(
                    (self.line, self.column + i + 1),
                    (self.line, self.column + i + 2),
                );
                self.push_token(
                    TokenKind::Operator(c.to_operator()),
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

        self.push_token(TokenKind::Punctuation(ch.to_punctuation()), span);
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
use crate::axo_lexer::error::{IntParseError, LexError};
use crate::axo_lexer::{Lexer, Token, TokenKind};

pub trait NumberLexer {
    fn handle_number(&mut self) -> Result<(), LexError>;
    fn lex_number(number: &str) -> Result<TokenKind, LexError>;
}

impl NumberLexer for Lexer {
    fn handle_number(&mut self) -> Result<(), LexError> {
        let first = self.next().unwrap();

        let mut number = first.to_string();

        let start = (self.line, self.column);

        while let Some(ch) = self.peek() {
            match ch {
                ch if ch.is_digit(10) => {
                    let digit = self.next().unwrap();

                    number.push(digit);
                }
                '.' => {
                    if let Some(ch) = self.peek_ahead(1) {
                        if ch.is_digit(10) {
                            self.next();
                            number.push('.');

                            self.next();
                            number.push(ch);
                        } else {
                            break;
                        }
                    } else {
                        self.next();
                        number.push('.');
                    }
                }
                '_' => {
                    self.next();
                }
                _ => break,
            }
        }

        let end = (self.line, self.column);

        let kind = Self::lex_number(&number)?;

        self.tokens.push(Token {
            kind,
            span: self.create_span(start, end),
        });

        Ok(())
    }

    fn lex_number(number: &str) -> Result<TokenKind, LexError> {
        if number.len() > 2 {
            match &number[0..2] {
                "0x" | "0X" => {
                    let hex_part = &number[2..];
                    if hex_part.chars().all(|c| c.is_digit(16)) {
                        if let Ok(num) = i64::from_str_radix(hex_part, 16) {
                            return Ok(TokenKind::Integer(num));
                        }
                    }
                    return Err(LexError::IntParseError(IntParseError::InvalidHexadecimal));
                }
                "0o" | "0O" => {
                    let oct_part = &number[2..];
                    if oct_part.chars().all(|c| c.is_digit(8)) {
                        if let Ok(num) = i64::from_str_radix(oct_part, 8) {
                            return Ok(TokenKind::Integer(num));
                        }
                    }
                    return Err(LexError::IntParseError(IntParseError::InvalidOctal));
                }
                "0b" | "0B" => {
                    let bin_part = &number[2..];
                    if bin_part.chars().all(|c| c.is_digit(2)) {
                        if let Ok(num) = i64::from_str_radix(bin_part, 2) {
                            return Ok(TokenKind::Integer(num));
                        }
                    }
                    return Err(LexError::IntParseError(IntParseError::InvalidBinary));
                }
                _ => {}
            }
        }

        if number.contains('e') || number.contains('E') {
            let parts: Vec<&str> = if number.contains('e') {
                number.split('e').collect()
            } else {
                number.split('E').collect()
            };

            if parts.len() == 2 {
                let base = parts[0];
                let exponent = parts[1];

                let base_valid = base.is_empty() || base == "." || base.parse::<f64>().is_ok();

                let exponent_valid = exponent.is_empty()
                    || exponent == "+"
                    || exponent == "-"
                    || exponent.parse::<i32>().is_ok()
                    || (exponent.starts_with('+') && exponent[1..].parse::<i32>().is_ok())
                    || (exponent.starts_with('-') && exponent[1..].parse::<i32>().is_ok());

                if base_valid && exponent_valid {
                    return match number.parse::<f64>() {
                        Ok(num) => Ok(TokenKind::Float(num)),
                        Err(_) => Err(LexError::FloatParseError(
                            IntParseError::InvalidScientificNotation,
                        )),
                    };
                }
            }
        }

        if number.contains('.') {
            match number.parse::<f64>() {
                Ok(num) => Ok(TokenKind::Float(num)),
                Err(e) => Err(LexError::NumberParse(e.to_string())),
            }
        } else {
            match number.parse::<i64>() {
                Ok(num) => Ok(TokenKind::Integer(num)),
                Err(e) => Err(LexError::NumberParse(e.to_string())),
            }
        }
    }
}
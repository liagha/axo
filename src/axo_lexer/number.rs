use crate::axo_lexer::error::{IntParseError, ErrorKind, Error};
use crate::axo_lexer::{Lexer, Token, TokenKind, Span};

pub trait NumberLexer {
    fn handle_number(&mut self) -> Result<(), Error>;
    fn lex_number(&self, number: &str, span: Span) -> Result<TokenKind, Error>;
}

impl NumberLexer for Lexer {
    fn handle_number(&mut self) -> Result<(), Error> {
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
        let span = self.create_span(start, end);

        // Pass the span to lex_number for proper error reporting
        match self.lex_number(&number, span.clone()) {
            Ok(kind) => {
                self.tokens.push(Token {
                    kind,
                    span,
                });
                Ok(())
            },
            Err(mut err) => {
                // The error should already have the span from lex_number
                // But let's ensure it's set properly
                if err.span.start == (0, 0) && err.span.end == (0, 0) {
                    err = Error::new(err.kind, span);
                }
                Err(err)
            }
        }
    }

    fn lex_number(&self, number: &str, span: Span) -> Result<TokenKind, Error> {
        if number.len() > 2 {
            match &number[0..2] {
                "0x" | "0X" => {
                    let hex_part = &number[2..];
                    if hex_part.chars().all(|c| c.is_digit(16)) {
                        if let Ok(num) = i128::from_str_radix(hex_part, 16) {
                            return Ok(TokenKind::Integer(num));
                        }
                    }
                    return Err(Error::new(ErrorKind::IntParseError(IntParseError::InvalidHexadecimal), span));
                }
                "0o" | "0O" => {
                    let oct_part = &number[2..];
                    if oct_part.chars().all(|c| c.is_digit(8)) {
                        if let Ok(num) = i128::from_str_radix(oct_part, 8) {
                            return Ok(TokenKind::Integer(num));
                        }
                    }
                    return Err(Error::new(ErrorKind::IntParseError(IntParseError::InvalidOctal), span));
                }
                "0b" | "0B" => {
                    let bin_part = &number[2..];
                    if bin_part.chars().all(|c| c.is_digit(2)) {
                        if let Ok(num) = i128::from_str_radix(bin_part, 2) {
                            return Ok(TokenKind::Integer(num));
                        }
                    }
                    return Err(Error::new(ErrorKind::IntParseError(IntParseError::InvalidBinary), span));
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
                        Ok(num) => Ok(TokenKind::Float(num.into())),
                        Err(_) => Err(Error::new(
                            ErrorKind::FloatParseError(IntParseError::InvalidScientificNotation),
                            span
                        )),
                    };
                }
            }
        }

        if number.contains('.') {
            match number.parse::<f64>() {
                Ok(num) => Ok(TokenKind::Float(num.into())),
                Err(e) => Err(Error::new(ErrorKind::NumberParse(e.to_string()), span)),
            }
        } else {
            match number.parse::<i128>() {
                Ok(num) => Ok(TokenKind::Integer(num)),
                Err(e) => Err(Error::new(ErrorKind::NumberParse(e.to_string()), span)),
            }
        }
    }
}
use crate::axo_lexer::error::{IntParseError, ErrorKind};
use crate::axo_lexer::{Lexer, Token, TokenKind, Span, Error};
use lexical_core::parse;

pub trait NumberLexer {
    fn handle_number(&mut self) -> Result<(), Error>;
    fn lex_number(&self, number: &str, span: Span) -> Result<TokenKind, Error>;
}

impl NumberLexer for Lexer {
    fn handle_number(&mut self) -> Result<(), Error> {
        let first = self.next().unwrap();

        let mut number = first.to_string();

        let start = (self.line, self.column);

        if first == '0' {
            if let Some(prefix) = self.peek() {
                match prefix {
                    'x' | 'X' | 'o' | 'O' | 'b' | 'B' => {
                        let prefix_char = self.next().unwrap();
                        number.push(prefix_char);

                        let radix = match prefix_char {
                            'x' | 'X' => 16,
                            'o' | 'O' => 8,
                            'b' | 'B' => 2,
                            _ => unreachable!()
                        };

                        while let Some(ch) = self.peek() {
                            if ch.is_digit(radix) ||
                                (radix == 16 && (('a'..='f').contains(&ch) || ('A'..='F').contains(&ch))) {
                                let digit = self.next().unwrap();
                                number.push(digit);
                            } else if ch == '_' {
                                self.next();
                            } else {
                                break;
                            }
                        }
                    },
                    _ => {}
                }
            }
        }

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
                'e' | 'E' => {
                    if !number.is_empty() {
                        if let Some(next_ch) = self.peek_ahead(1) {
                            if next_ch.is_digit(10) || next_ch == '+' || next_ch == '-' {
                                if next_ch == '+' || next_ch == '-' {
                                    if let Some(digit_after) = self.peek_ahead(2) {
                                        if !digit_after.is_digit(10) {
                                            break;
                                        }
                                    } else {
                                        break;
                                    }
                                }
                                let e_char = self.next().unwrap();
                                number.push(e_char);

                                if next_ch == '+' || next_ch == '-' {
                                    let sign = self.next().unwrap();
                                    number.push(sign);
                                }
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    } else {
                        break;
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

        match self.lex_number(&number, span.clone()) {
            Ok(kind) => {
                self.tokens.push(Token {
                    kind,
                    span,
                });
                Ok(())
            },
            Err(mut err) => {
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
                    if hex_part.chars().all(|c| c.is_digit(16) || ('A'..='F').contains(&c)) {
                        let parsed_value = i128::from_str_radix(hex_part, 16);
                        match parsed_value {
                            Ok(num) => return Ok(TokenKind::Integer(num)),
                            Err(_) => {}
                        }
                    }
                    return Err(Error::new(ErrorKind::IntParseError(IntParseError::InvalidHexadecimal), span));
                }
                "0o" | "0O" => {
                    let oct_part = &number[2..];
                    if oct_part.chars().all(|c| c.is_digit(8)) {
                        let parsed_value = i128::from_str_radix(oct_part, 8);
                        match parsed_value {
                            Ok(num) => return Ok(TokenKind::Integer(num)),
                            Err(_) => {}
                        }
                    }
                    return Err(Error::new(ErrorKind::IntParseError(IntParseError::InvalidOctal), span));
                }
                "0b" | "0B" => {
                    let bin_part = &number[2..];
                    if bin_part.chars().all(|c| c.is_digit(2)) {
                        let parsed_value = i128::from_str_radix(bin_part, 2);
                        match parsed_value {
                            Ok(num) => return Ok(TokenKind::Integer(num)),
                            Err(_) => {}
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

                let base_valid = base.is_empty() || base == "." ||
                    parse::<f64>(base.as_bytes()).is_ok();

                let exponent_valid = exponent.is_empty()
                    || exponent == "+"
                    || exponent == "-"
                    || parse::<i32>(exponent.as_bytes()).is_ok()
                    || (exponent.starts_with('+') && parse::<i32>(exponent[1..].as_bytes()).is_ok())
                    || (exponent.starts_with('-') && parse::<i32>(exponent[1..].as_bytes()).is_ok());

                if base_valid && exponent_valid {
                    return match parse::<f64>(number.as_bytes()) {
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
            match parse::<f64>(number.as_bytes()) {
                Ok(num) => Ok(TokenKind::Float(num.into())),
                Err(e) => Err(Error::new(ErrorKind::NumberParse(e.to_string()), span)),
            }
        } else {
            match parse::<i128>(number.as_bytes()) {
                Ok(num) => Ok(TokenKind::Integer(num)),
                Err(e) => Err(Error::new(ErrorKind::NumberParse(e.to_string()), span)),
            }
        }
    }
}
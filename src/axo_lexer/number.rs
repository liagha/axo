use crate::parser;
use crate::axo_lexer::error::ErrorKind;
use crate::axo_lexer::{Lexer, Token, TokenKind, Span, LexError};

pub trait NumberLexer {
    fn handle_number(&mut self) -> Result<(), LexError>;
    fn lex_number(&self, number: &str, span: Span) -> Result<TokenKind, LexError>;
}

impl NumberLexer for Lexer {
    fn handle_number(&mut self) -> Result<(), LexError> {
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
                    err = LexError::new(err.kind, span);
                }
                Err(err)
            }
        }
    }

    fn lex_number(&self, number: &str, span: Span) -> Result<TokenKind, LexError> {
        if number.contains('.') {
            let parser = parser::<f64>();

            match parser.parse(number) {
                Ok(num) => Ok(TokenKind::Float(num.into())),
                Err(e) => Err(LexError::new(ErrorKind::NumberParse(e), span)),
            }
        } else {
            let parser = parser::<i128>();

            match parser.parse(number) {
                Ok(num) => Ok(TokenKind::Integer(num)),
                Err(e) => Err(LexError::new(ErrorKind::NumberParse(e), span)),
            }
        }
    }
}
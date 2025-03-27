use crate::lexer::error::{LexError, IntParseError, CharParseError};
use crate::lexer::Token;
use crate::lexer::{TokenKind, OperatorKind, PunctuationKind};

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: (usize, usize),  // (line, column)
    pub end: (usize, usize),    // (line, column)
}

pub struct Lexer {
    input: String,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: String) -> Lexer {
        Lexer { input, line: 1, column: 0 }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut chars = self.input.chars().peekable();
        let mut tokens = Vec::new();

        while let Some(ch) = chars.next() {
            self.column += 1;

            match ch {
                '\n' => {
                    let start = (self.line, self.column);

                    self.line += 1;
                    self.column = 0;

                    let end = (self.line, self.column);

                    let span = Span { start, end };

                    let token = Token {
                        kind: TokenKind::Punctuation(PunctuationKind::from_char(&ch)),
                        span
                    };

                    tokens.push(token);
                }

                ch if ch.is_whitespace() => continue,

                ch if ch.is_digit(10) || ch == '.' => {
                    let mut number = ch.to_string();
                    let start = (self.line, self.column);

                    while let Some(&next_ch) = chars.peek() {
                        if next_ch.is_digit(10) || next_ch == '.' {
                            let next_digit = chars.next().unwrap();

                            number.push(next_digit);

                            self.column += 1;
                        } else if next_ch == '_' {
                            chars.next();

                            self.column += 1;
                        } else {
                            break;
                        }
                    }

                    let end = (self.line, self.column);
                    let span = Span { start, end };

                    if number == "." {
                        tokens.push(Token {
                            kind: TokenKind::Operator(OperatorKind::Dot),
                            span
                        });
                    } else if number == ".." {
                        tokens.push(Token {
                            kind: TokenKind::Operator(OperatorKind::DotDot),
                            span
                        });
                    } else if number.ends_with("..") {
                        let num_part = number.trim_end_matches("..");
                        if !num_part.is_empty() {
                            let num_span = Span {
                                start: start,
                                end: (self.line, self.column + num_part.len())
                            };
                            tokens.push(Token {
                                kind: Self::lex_number(num_part, self.line, self.column)?,
                                span: num_span                            });
                        }

                        let op_span = Span {
                            start: (self.line, self.column + number.len() - 2),
                            end
                        };
                        tokens.push(Token {
                            kind: TokenKind::Operator(OperatorKind::DotDot),
                            span: op_span
                        });
                    } else if number.contains("..") {
                        let parts: Vec<&str> = number.split("..").collect();

                        if parts.len() == 2 {
                            if !parts[0].is_empty() {
                                let first_span = Span {
                                    start,
                                    end: (self.line, self.column + parts[0].len())
                                };
                                tokens.push(Token {
                                    kind: Self::lex_number(parts[0], self.line, self.column)?,
                                    span: first_span
                                });
                            }

                            let op_span = Span {
                                start: (self.line, self.column + parts[0].len()),
                                end: (self.line, self.column + parts[0].len() + 2)
                            };
                            tokens.push(Token {
                                kind: TokenKind::Operator(OperatorKind::DotDot),
                                span: op_span
                            });

                            if !parts[1].is_empty() {
                                let second_span = Span {
                                    start: (self.line, self.column + parts[0].len() + 2),
                                    end
                                };
                                tokens.push(Token {
                                    kind: Self::lex_number(parts[1], self.line, self.column + parts[0].len() + 2)?,
                                    span: second_span
                                });
                            }
                        } else {
                            return Err(LexError::IntParseError(IntParseError::InvalidRange));
                        }
                    } else {
                        tokens.push(Token {
                            kind: Self::lex_number(&number, self.line, self.column)?,
                            span
                        });
                    }
                }

                ch if ch.is_alphabetic() || ch == '_' => {
                    let mut name = ch.to_string();
                    let start = (self.line, self.column);

                    while let Some(&next_ch) = chars.peek() {
                        if next_ch.is_alphanumeric() || next_ch == '_' {
                            name.push(chars.next().unwrap());
                            self.column += 1;
                        } else {
                            break;
                        }
                    }

                    let end = (self.line, self.column);
                    let span = Span { start, end };

                    match TokenKind::from_str(name.as_str()) {
                        Some(token_kind) => tokens.push(Token { kind: token_kind, span }),
                        _ => tokens.push(Token { kind: TokenKind::Identifier(name), span }),
                    }
                }

                '\'' => {
                    let mut content = String::new();
                    let mut closed = false;
                    let mut is_escaped = false;

                    let start = (self.line, self.column);

                    while let Some(next_ch) = chars.next() {
                        self.column += 1;

                        if is_escaped {
                            match next_ch {
                                'n' => content.push('\n'),
                                'r' => content.push('\r'),
                                't' => content.push('\t'),
                                '\\' => content.push('\\'),
                                '\'' => content.push('\''),
                                '"' => content.push('"'),
                                '0' => content.push('\0'),
                                'x' => {
                                    let mut hex = String::new();
                                    for _ in 0..2 {
                                        if let Some(&next_hex) = chars.peek() {
                                            if next_hex.is_digit(16) {
                                                hex.push(chars.next().unwrap());
                                                self.column += 1;
                                            } else {
                                                break;
                                            }
                                        } else {
                                            break;
                                        }
                                    }

                                    if let Ok(hex_value) = u32::from_str_radix(&hex, 16) {
                                        if let Some(ch) = std::char::from_u32(hex_value) {
                                            content.push(ch);
                                        } else {
                                            return Err(LexError::CharParseError(CharParseError::InvalidEscapeSequence));
                                        }
                                    } else {
                                        return Err(LexError::CharParseError(CharParseError::InvalidEscapeSequence));
                                    }
                                },
                                'u' => {
                                    if chars.peek() == Some(&'{') {
                                        chars.next(); 
                                        self.column += 1;

                                        let mut hex = String::new();
                                        let mut closed_brace = false;

                                        for _ in 0..6 {
                                            if let Some(&next_hex) = chars.peek() {
                                                if next_hex.is_digit(16) {
                                                    hex.push(chars.next().unwrap());
                                                    self.column += 1;
                                                } else if next_hex == '}' {
                                                    chars.next(); 
                                                    self.column += 1;
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
                                            return Err(LexError::CharParseError(CharParseError::UnClosedEscapeSequence));
                                        }

                                        if let Ok(hex_value) = u32::from_str_radix(&hex, 16) {
                                            if let Some(ch) = std::char::from_u32(hex_value) {
                                                content.push(ch);
                                            } else {
                                                return Err(LexError::CharParseError(CharParseError::InvalidEscapeSequence));
                                            }
                                        } else {
                                            return Err(LexError::CharParseError(CharParseError::InvalidEscapeSequence));
                                        }
                                    } else {
                                        return Err(LexError::CharParseError(CharParseError::InvalidEscapeSequence));
                                    }
                                },
                                _ => content.push(next_ch),
                            }
                            is_escaped = false;
                        } else if next_ch == '\\' {
                            is_escaped = true;
                        } else if next_ch == '\'' {
                            let end = (self.line, self.column);
                            let span = Span { start, end };

                            if content.chars().count() == 1 {
                                let ch = content.chars().next().unwrap();
                                tokens.push(Token { kind: TokenKind::Char(ch), span });
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
                        let span = Span { start, end };

                        tokens.push(Token {
                            kind: TokenKind::Invalid(format!("'{}", content)),
                            span
                        });

                        return Err(LexError::UnClosedChar);
                    }
                }

                '"' => {
                    let mut content = String::new();
                    let mut closed = false;
                    let start = (self.line, self.column);

                    let mut is_escaped = false;

                    while let Some(next_ch) = chars.next() {
                        self.column += 1;

                        if is_escaped {
                            match next_ch {
                                'n' => content.push('\n'),
                                'r' => content.push('\r'),
                                't' => content.push('\t'),
                                '\\' => content.push('\\'),
                                '\'' => content.push('\''),
                                '"' => content.push('"'),
                                '0' => content.push('\0'),
                                'x' => {
                                    let mut hex = String::new();
                                    for _ in 0..2 {
                                        if let Some(&next_hex) = chars.peek() {
                                            if next_hex.is_digit(16) {
                                                hex.push(chars.next().unwrap());
                                                self.column += 1;
                                            } else {
                                                break;
                                            }
                                        } else {
                                            break;
                                        }
                                    }

                                    if let Ok(hex_value) = u32::from_str_radix(&hex, 16) {
                                        if let Some(ch) = std::char::from_u32(hex_value) {
                                            content.push(ch);
                                        } else {
                                            return Err(LexError::StringParseError(CharParseError::InvalidEscapeSequence));
                                        }
                                    } else {
                                        return Err(LexError::StringParseError(CharParseError::InvalidEscapeSequence));
                                    }
                                },
                                'u' => {
                                    if chars.peek() == Some(&'{') {
                                        chars.next(); 
                                        self.column += 1;

                                        let mut hex = String::new();
                                        let mut closed_brace = false;

                                        for _ in 0..6 {
                                            if let Some(&next_hex) = chars.peek() {
                                                if next_hex.is_digit(16) {
                                                    hex.push(chars.next().unwrap());
                                                    self.column += 1;
                                                } else if next_hex == '}' {
                                                    chars.next(); 
                                                    self.column += 1;
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
                                            return Err(LexError::StringParseError(CharParseError::UnClosedEscapeSequence));
                                        }

                                        if let Ok(hex_value) = u32::from_str_radix(&hex, 16) {
                                            if let Some(ch) = std::char::from_u32(hex_value) {
                                                content.push(ch);
                                            } else {
                                                return Err(LexError::StringParseError(CharParseError::InvalidEscapeSequence));
                                            }
                                        } else {
                                            return Err(LexError::StringParseError(CharParseError::InvalidEscapeSequence));
                                        }
                                    } else {
                                        return Err(LexError::StringParseError(CharParseError::InvalidEscapeSequence));
                                    }
                                },
                                _ => content.push(next_ch),
                            }
                            is_escaped = false;
                        } else if next_ch == '\\' {
                            is_escaped = true;
                        } else if next_ch == '"' {
                            let end = (self.line, self.column);
                            let span = Span { start, end };

                            tokens.push(Token {
                                kind: TokenKind::Str(content.clone()),
                                span
                            });

                            closed = true;
                            break;
                        } else {
                            content.push(next_ch);
                        }
                    }

                    if !closed {
                        let end = (self.line, self.column);
                        let span = Span { start, end };

                        tokens.push(Token {
                            kind: TokenKind::Invalid(format!("\"{}\"", content)),
                            span
                        });

                        return Err(LexError::UnClosedString);
                    }
                }

                '/' => {
                    let start = (self.line, self.column);

                    if let Some(&next_ch) = chars.peek() {
                        match next_ch {
                            '/' => {
                                let mut comment = String::new();
                                chars.next();
                                self.column += 1;

                                while let Some(&next_ch) = chars.peek() {
                                    if next_ch == '\n' { break; }

                                    comment.push(next_ch);
                                    chars.next();
                                    self.column += 1;
                                }

                                let end = (self.line, self.column);
                                let span = Span { start, end };

                                tokens.push(Token {
                                    kind: TokenKind::Comment(comment),
                                    span
                                });
                            },
                            '*' => {
                                let mut comment = String::new();
                                chars.next();
                                self.column += 1;

                                let mut closed = false;
                                let mut last_char = '*';

                                while let Some(next_ch) = chars.next() {
                                    if last_char == '*' && next_ch == '/' {
                                        closed = true;
                                        comment.pop();
                                        break;
                                    }

                                    self.column += 1;
                                    comment.push(next_ch);

                                    last_char = next_ch;
                                }

                                let end = (self.line, self.column);
                                let span = Span { start, end };

                                if closed {
                                    tokens.push(Token {
                                        kind: TokenKind::Comment(comment),
                                        span
                                    });
                                } else {
                                    tokens.push(Token {
                                        kind: TokenKind::Invalid(comment),
                                        span
                                    });

                                    return Err(LexError::UnClosedComment);
                                }
                            },
                            _ => {
                                let end = (self.line, self.column);
                                let span = Span { start, end };

                                tokens.push(Token {
                                    kind: TokenKind::Operator(OperatorKind::Slash),
                                    span
                                });
                            }
                        }
                    } else {
                        let end = (self.line, self.column);
                        let span = Span { start, end };

                        tokens.push(Token {
                            kind: TokenKind::Operator(OperatorKind::Slash),
                            span
                        });
                    }
                }

                ch if OperatorKind::is_operator(ch) => {
                    let mut operator = ch.to_string();
                    let start = (self.line, self.column);

                    while let Some(&next_ch) = chars.peek() {
                        if OperatorKind::is_operator(next_ch) {
                            operator.push(chars.next().unwrap());
                            self.column += 1;
                        } else {
                            break;
                        }
                    }

                    let end = (self.line, self.column);
                    let span = Span { start, end };

                    if OperatorKind::Unknown != OperatorKind::from_str(&operator) {
                        let op = OperatorKind::from_str(&operator);

                        tokens.push(Token {
                            kind: TokenKind::Operator(op),
                            span
                        });
                    } else {
                        for (i, c) in operator.chars().enumerate() {
                            let single_char_span = Span {
                                start: (self.line, self.column + i + 1),
                                end: (self.line, self.column + i + 2),
                            };
                            tokens.push(Token {
                                kind: TokenKind::Operator(OperatorKind::from_str(c.to_string().as_str())),
                                span: single_char_span,
                            });
                        }
                    }
                }


                ch if PunctuationKind::is_punctuation(ch) => {
                    let start = (self.line, self.column);
                    let end = (self.line, self.column);
                    let span = Span { start, end };

                    let punc = PunctuationKind::from_char(&ch);

                    tokens.push(Token {
                        kind: TokenKind::Punctuation(punc),
                        span
                    });
                }

                _ => {
                    let start = (self.line, self.column);
                    let end = (self.line, self.column);
                    let span = Span { start, end };

                    tokens.push(Token {
                        kind: TokenKind::Invalid(ch.to_string()),
                        span
                    });

                    return Err(LexError::InvalidChar);
                }
            }
        }

        let line_count = self.input.lines().count();
        let last_line_length = self.input.lines().last().map_or(0, |line| line.len());
        let start = (line_count, last_line_length + 1);
        let end = (line_count, last_line_length + 1);
        let span = Span { start, end };

        tokens.push(Token {
            kind: TokenKind::EOF,
            span
        });

        Ok(tokens)
    }

    fn lex_number(number: &str, line: usize, column: usize) -> Result<TokenKind, LexError> {
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
                },
                "0o" | "0O" => {
                    let oct_part = &number[2..];
                    if oct_part.chars().all(|c| c.is_digit(8)) {
                        if let Ok(num) = i64::from_str_radix(oct_part, 8) {
                            return Ok(TokenKind::Integer(num));
                        }
                    }
                    return Err(LexError::IntParseError(IntParseError::InvalidOctal));
                },
                "0b" | "0B" => {
                    let bin_part = &number[2..];
                    if bin_part.chars().all(|c| c.is_digit(2)) {
                        if let Ok(num) = i64::from_str_radix(bin_part, 2) {
                            return Ok(TokenKind::Integer(num));
                        }
                    }
                    return Err(LexError::IntParseError(IntParseError::InvalidBinary));
                },
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

                let base_valid = base.is_empty()
                    || base == "."
                    || base.parse::<f64>().is_ok();

                let exponent_valid = exponent.is_empty()
                    || exponent == "+"
                    || exponent == "-"
                    || exponent.parse::<i32>().is_ok()
                    || (exponent.starts_with('+') && exponent[1..].parse::<i32>().is_ok())
                    || (exponent.starts_with('-') && exponent[1..].parse::<i32>().is_ok());

                if base_valid && exponent_valid {
                    return match number.parse::<f64>() {
                        Ok(num) => Ok(TokenKind::Float(num)),
                        Err(_) => Err(LexError::FloatParseError(IntParseError::InvalidScientificNotation))
                    }
                }
            }
        }

        if number.contains('.') {
            match number.parse::<f64>() {
                Ok(num) => Ok(TokenKind::Float(num)),
                Err(e) => Err(LexError::NumberParse(e.to_string()))
            }
        } else {
            match number.parse::<i64>() {
                Ok(num) => Ok(TokenKind::Integer(num)),
                Err(e) => Err(LexError::NumberParse(e.to_string()))
            }
        }
    }
}

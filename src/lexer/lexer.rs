use crate::errors::LexError;
use crate::lexer::{TokenKind, OperatorKind, PunctuationKind};

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: (usize, usize),  // (line, column)
    pub end: (usize, usize),    // (line, column)
}

#[derive(Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub struct Lexer {
    input: String,
}

impl Lexer {
    pub fn new(input: String) -> Lexer {
        Lexer { input }
    }

    pub fn tokenize(&self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        for (line_number, line) in self.input.lines().enumerate() {
            let mut chars = line.chars().peekable();
            let mut column_number = 0;

            while let Some(ch) = chars.next() {
                let start_column = column_number;
                column_number += 1;

                match ch {
                    ch if ch.is_whitespace() => continue,

                    ch if ch.is_digit(10) || ch == '.' => {
                        let mut number = ch.to_string();
                        let start_position = (line_number + 1, start_column + 1);

                        while let Some(&next_ch) = chars.peek() {
                            if next_ch.is_digit(10) || next_ch == '.' {
                                number.push(chars.next().unwrap());
                                column_number += 1;
                            } else if next_ch == '_' {
                                // Allow underscores in numbers for readability (e.g., 1_000_000)
                                chars.next();
                                column_number += 1;
                            } else {
                                break;
                            }
                        }

                        let end_position = (line_number + 1, column_number);
                        let span = Span { start: start_position, end: end_position };

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
                                    start: start_position,
                                    end: (line_number + 1, start_column + 1 + num_part.len())
                                };
                                tokens.push(Token {
                                    kind: Self::lex_number(num_part, line_number + 1, start_column + 1)?,
                                    span: num_span
                                });
                            }

                            let op_span = Span {
                                start: (line_number + 1, start_column + 1 + number.len() - 2),
                                end: end_position
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
                                        start: start_position,
                                        end: (line_number + 1, start_column + 1 + parts[0].len())
                                    };
                                    tokens.push(Token {
                                        kind: Self::lex_number(parts[0], line_number + 1, start_column + 1)?,
                                        span: first_span
                                    });
                                }

                                let op_span = Span {
                                    start: (line_number + 1, start_column + 1 + parts[0].len()),
                                    end: (line_number + 1, start_column + 1 + parts[0].len() + 2)
                                };
                                tokens.push(Token {
                                    kind: TokenKind::Operator(OperatorKind::DotDot),
                                    span: op_span
                                });

                                if !parts[1].is_empty() {
                                    let second_span = Span {
                                        start: (line_number + 1, start_column + 1 + parts[0].len() + 2),
                                        end: end_position
                                    };
                                    tokens.push(Token {
                                        kind: Self::lex_number(parts[1], line_number + 1, start_column + 1 + parts[0].len() + 2)?,
                                        span: second_span
                                    });
                                }
                            } else {
                                return Err(LexError::IntParseError(format!("Invalid range syntax at line {}, column {}", line_number + 1, start_column + 1)));
                            }
                        } else {
                            tokens.push(Token {
                                kind: Self::lex_number(&number, line_number + 1, start_column + 1)?,
                                span
                            });
                        }
                    }

                    ch if ch.is_alphabetic() || ch == '_' => {
                        let mut name = ch.to_string();
                        let start_position = (line_number + 1, start_column + 1);

                        while let Some(&next_ch) = chars.peek() {
                            if next_ch.is_alphanumeric() || next_ch == '_' {
                                name.push(chars.next().unwrap());
                                column_number += 1;
                            } else {
                                break;
                            }
                        }

                        let end_position = (line_number + 1, column_number);
                        let span = Span { start: start_position, end: end_position };

                        match TokenKind::from_str(name.as_str()) {
                            Some(token_kind) => tokens.push(Token { kind: token_kind, span }),
                            _ => tokens.push(Token { kind: TokenKind::Identifier(name), span }),
                        }
                    }

                    '\'' => {
                        let mut string_content = String::new();
                        let mut closed = false;
                        let start_position = (line_number + 1, start_column + 1);

                        // Handle escape sequences
                        let mut is_escaped = false;

                        while let Some(next_ch) = chars.next() {
                            column_number += 1;

                            if is_escaped {
                                // Process escape sequences
                                match next_ch {
                                    'n' => string_content.push('\n'),
                                    'r' => string_content.push('\r'),
                                    't' => string_content.push('\t'),
                                    '\\' => string_content.push('\\'),
                                    '\'' => string_content.push('\''),
                                    '"' => string_content.push('"'),
                                    '0' => string_content.push('\0'),
                                    'x' => {
                                        // Handle hexadecimal escape sequence \xHH
                                        let mut hex = String::new();
                                        for _ in 0..2 {
                                            if let Some(&next_hex) = chars.peek() {
                                                if next_hex.is_digit(16) {
                                                    hex.push(chars.next().unwrap());
                                                    column_number += 1;
                                                } else {
                                                    break;
                                                }
                                            } else {
                                                break;
                                            }
                                        }

                                        if let Ok(hex_value) = u32::from_str_radix(&hex, 16) {
                                            if let Some(ch) = std::char::from_u32(hex_value) {
                                                string_content.push(ch);
                                            } else {
                                                return Err(LexError::CharParseError(format!("Invalid hex escape sequence '\\x{}' at line {}, column {}", hex, line_number + 1, start_column + 1)));
                                            }
                                        } else {
                                            return Err(LexError::CharParseError(format!("Invalid hex escape sequence '\\x{}' at line {}, column {}", hex, line_number + 1, start_column + 1)));
                                        }
                                    },
                                    'u' => {
                                        // Handle Unicode escape sequence \u{H...} (1-6 hex digits)
                                        if chars.peek() == Some(&'{') {
                                            chars.next(); // consume '{'
                                            column_number += 1;

                                            let mut hex = String::new();
                                            let mut closed_brace = false;

                                            for _ in 0..6 {
                                                if let Some(&next_hex) = chars.peek() {
                                                    if next_hex.is_digit(16) {
                                                        hex.push(chars.next().unwrap());
                                                        column_number += 1;
                                                    } else if next_hex == '}' {
                                                        chars.next(); // consume '}'
                                                        column_number += 1;
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
                                                return Err(LexError::CharParseError(format!("Unclosed Unicode escape sequence at line {}, column {}", line_number + 1, start_column + 1)));
                                            }

                                            if let Ok(hex_value) = u32::from_str_radix(&hex, 16) {
                                                if let Some(ch) = std::char::from_u32(hex_value) {
                                                    string_content.push(ch);
                                                } else {
                                                    return Err(LexError::CharParseError(format!("Invalid Unicode escape sequence '\\u{{{}}}' at line {}, column {}", hex, line_number + 1, start_column + 1)));
                                                }
                                            } else {
                                                return Err(LexError::CharParseError(format!("Invalid Unicode escape sequence '\\u{{{}}}' at line {}, column {}", hex, line_number + 1, start_column + 1)));
                                            }
                                        } else {
                                            return Err(LexError::CharParseError(format!("Invalid Unicode escape sequence at line {}, column {}", line_number + 1, start_column + 1)));
                                        }
                                    },
                                    _ => string_content.push(next_ch),
                                }
                                is_escaped = false;
                            } else if next_ch == '\\' {
                                is_escaped = true;
                            } else if next_ch == '\'' {
                                let end_position = (line_number + 1, column_number);
                                let span = Span { start: start_position, end: end_position };

                                if string_content.chars().count() == 1 {
                                    let ch = string_content.chars().next().unwrap();
                                    tokens.push(Token { kind: TokenKind::Char(ch), span });
                                    closed = true;
                                    break;
                                } else {
                                    return Err(LexError::CharParseError(format!("Invalid character literal '{}' at line {}, column {} - character literals must contain exactly one character", string_content, line_number + 1, start_column + 1)));
                                }
                            } else {
                                string_content.push(next_ch);
                            }
                        }

                        if !closed {
                            let end_position = (line_number + 1, column_number);
                            let span = Span { start: start_position, end: end_position };

                            tokens.push(Token {
                                kind: TokenKind::Invalid(format!("'{}", string_content)),
                                span
                            });

                            return Err(LexError::UnClosedChar(format!("Unclosed character literal at line {}, column {}", line_number + 1, start_column + 1)));
                        }
                    }

                    '"' => {
                        let mut string_content = String::new();
                        let mut closed = false;
                        let start_position = (line_number + 1, start_column + 1);

                        // Handle escape sequences
                        let mut is_escaped = false;

                        while let Some(next_ch) = chars.next() {
                            column_number += 1;

                            if is_escaped {
                                // Process escape sequences (same as for character literals)
                                match next_ch {
                                    'n' => string_content.push('\n'),
                                    'r' => string_content.push('\r'),
                                    't' => string_content.push('\t'),
                                    '\\' => string_content.push('\\'),
                                    '\'' => string_content.push('\''),
                                    '"' => string_content.push('"'),
                                    '0' => string_content.push('\0'),
                                    'x' => {
                                        // Handle hexadecimal escape sequence \xHH
                                        let mut hex = String::new();
                                        for _ in 0..2 {
                                            if let Some(&next_hex) = chars.peek() {
                                                if next_hex.is_digit(16) {
                                                    hex.push(chars.next().unwrap());
                                                    column_number += 1;
                                                } else {
                                                    break;
                                                }
                                            } else {
                                                break;
                                            }
                                        }

                                        if let Ok(hex_value) = u32::from_str_radix(&hex, 16) {
                                            if let Some(ch) = std::char::from_u32(hex_value) {
                                                string_content.push(ch);
                                            } else {
                                                return Err(LexError::StringParseError(format!("Invalid hex escape sequence '\\x{}' at line {}, column {}", hex, line_number + 1, start_column + 1)));
                                            }
                                        } else {
                                            return Err(LexError::StringParseError(format!("Invalid hex escape sequence '\\x{}' at line {}, column {}", hex, line_number + 1, start_column + 1)));
                                        }
                                    },
                                    'u' => {
                                        // Handle Unicode escape sequence \u{H...} (1-6 hex digits)
                                        if chars.peek() == Some(&'{') {
                                            chars.next(); // consume '{'
                                            column_number += 1;

                                            let mut hex = String::new();
                                            let mut closed_brace = false;

                                            for _ in 0..6 {
                                                if let Some(&next_hex) = chars.peek() {
                                                    if next_hex.is_digit(16) {
                                                        hex.push(chars.next().unwrap());
                                                        column_number += 1;
                                                    } else if next_hex == '}' {
                                                        chars.next(); // consume '}'
                                                        column_number += 1;
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
                                                return Err(LexError::StringParseError(format!("Unclosed Unicode escape sequence at line {}, column {}", line_number + 1, start_column + 1)));
                                            }

                                            if let Ok(hex_value) = u32::from_str_radix(&hex, 16) {
                                                if let Some(ch) = std::char::from_u32(hex_value) {
                                                    string_content.push(ch);
                                                } else {
                                                    return Err(LexError::StringParseError(format!("Invalid Unicode escape sequence '\\u{{{}}}' at line {}, column {}", hex, line_number + 1, start_column + 1)));
                                                }
                                            } else {
                                                return Err(LexError::StringParseError(format!("Invalid Unicode escape sequence '\\u{{{}}}' at line {}, column {}", hex, line_number + 1, start_column + 1)));
                                            }
                                        } else {
                                            return Err(LexError::StringParseError(format!("Invalid Unicode escape sequence at line {}, column {}", line_number + 1, start_column + 1)));
                                        }
                                    },
                                    _ => string_content.push(next_ch),
                                }
                                is_escaped = false;
                            } else if next_ch == '\\' {
                                is_escaped = true;
                            } else if next_ch == '"' {
                                let end_position = (line_number + 1, column_number);
                                let span = Span { start: start_position, end: end_position };

                                tokens.push(Token {
                                    kind: TokenKind::Str(string_content.clone()),
                                    span
                                });

                                closed = true;
                                break;
                            } else {
                                string_content.push(next_ch);
                            }
                        }

                        if !closed {
                            let end_position = (line_number + 1, column_number);
                            let span = Span { start: start_position, end: end_position };

                            tokens.push(Token {
                                kind: TokenKind::Invalid(format!("\"{}\"", string_content)),
                                span
                            });

                            return Err(LexError::UnClosedString(format!("Unclosed string literal at line {}, column {}", line_number + 1, start_column + 1)));
                        }
                    }

                    // Handle comments
                    '/' => {
                        let start_position = (line_number + 1, start_column + 1);

                        if let Some(&next_ch) = chars.peek() {
                            match next_ch {
                                '/' => {
                                    // Line comment
                                    let mut comment = String::from("//");
                                    chars.next(); // consume the second '/'
                                    column_number += 1;

                                    while let Some(&next_ch) = chars.peek() {
                                        comment.push(next_ch);
                                        chars.next();
                                        column_number += 1;
                                    }

                                    let end_position = (line_number + 1, column_number);
                                    let span = Span { start: start_position, end: end_position };

                                    tokens.push(Token {
                                        kind: TokenKind::Comment(comment),
                                        span
                                    });
                                },
                                '*' => {
                                    // Block comment
                                    let mut comment = String::from("/*");
                                    chars.next(); // consume the '*'
                                    column_number += 1;

                                    let mut closed = false;
                                    let mut last_char = '*';

                                    while let Some(next_ch) = chars.next() {
                                        column_number += 1;
                                        comment.push(next_ch);

                                        if last_char == '*' && next_ch == '/' {
                                            closed = true;
                                            break;
                                        }

                                        last_char = next_ch;
                                    }

                                    let end_position = (line_number + 1, column_number);
                                    let span = Span { start: start_position, end: end_position };

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

                                        return Err(LexError::UnClosedComment(format!("Unclosed block comment at line {}, column {}", line_number + 1, start_column + 1)));
                                    }
                                },
                                _ => {
                                    // Just a division operator
                                    let end_position = (line_number + 1, column_number);
                                    let span = Span { start: start_position, end: end_position };

                                    tokens.push(Token {
                                        kind: TokenKind::Operator(OperatorKind::Slash),
                                        span
                                    });
                                }
                            }
                        } else {
                            // Just a division operator at the end of the line
                            let end_position = (line_number + 1, column_number);
                            let span = Span { start: start_position, end: end_position };

                            tokens.push(Token {
                                kind: TokenKind::Operator(OperatorKind::Slash),
                                span
                            });
                        }
                    }

                    // Handle operators
                    ch if OperatorKind::is_operator(ch) => {
                        let mut operator = ch.to_string();
                        let start_position = (line_number + 1, start_column + 1);

                        while let Some(&next_ch) = chars.peek() {
                            if OperatorKind::is_operator(next_ch) {
                                operator.push(chars.next().unwrap());
                                column_number += 1;
                            } else {
                                break;
                            }
                        }

                        let end_position = (line_number + 1, column_number);
                        let span = Span { start: start_position, end: end_position };

                        let op = OperatorKind::from_str(&operator);
                        tokens.push(Token {
                            kind: TokenKind::Operator(op),
                            span
                        });
                    }

                    // Handle punctuation
                    ch if PunctuationKind::is_punctuation(ch) => {
                        let start_position = (line_number + 1, start_column + 1);
                        let end_position = (line_number + 1, column_number);
                        let span = Span { start: start_position, end: end_position };

                        let punc = PunctuationKind::from_str(&*ch.to_string());
                        tokens.push(Token {
                            kind: TokenKind::Punctuation(punc),
                            span
                        });
                    }

                    // Handle invalid characters
                    _ => {
                        let start_position = (line_number + 1, start_column + 1);
                        let end_position = (line_number + 1, column_number);
                        let span = Span { start: start_position, end: end_position };

                        tokens.push(Token {
                            kind: TokenKind::Invalid(ch.to_string()),
                            span
                        });

                        return Err(LexError::InvalidChar(format!("Invalid character '{}' at line {}, column {}", ch, line_number + 1, start_column + 1)));
                    }
                }
            }

            // Add newline token after each line except the last one
            if line_number != self.input.lines().count() - 1 {
                let start_position = (line_number + 1, column_number + 1);
                let end_position = (line_number + 1, column_number + 1);
                let span = Span { start: start_position, end: end_position };

                tokens.push(Token {
                    kind: TokenKind::Punctuation(PunctuationKind::Newline),
                    span
                });
            }
        }

        // Add EOF token at the end
        let line_count = self.input.lines().count();
        let last_line_length = self.input.lines().last().map_or(0, |line| line.len());
        let start_position = (line_count, last_line_length + 1);
        let end_position = (line_count, last_line_length + 1);
        let span = Span { start: start_position, end: end_position };

        tokens.push(Token {
            kind: TokenKind::EOF,
            span
        });

        Ok(tokens)
    }

    fn lex_number(number: &str, line: usize, column: usize) -> Result<TokenKind, LexError> {
        if number.is_empty() {
            return Err(LexError::IntParseError(format!("Empty string at line {}, column {}", line, column)));
        }

        // Check for hex, octal, and binary literals
        if number.len() > 2 {
            match &number[0..2] {
                "0x" | "0X" => {
                    // Hexadecimal
                    let hex_part = &number[2..];
                    if hex_part.chars().all(|c| c.is_digit(16)) {
                        if let Ok(num) = i64::from_str_radix(hex_part, 16) {
                            return Ok(TokenKind::Integer(num));
                        }
                    }
                    return Err(LexError::IntParseError(format!("Invalid hexadecimal literal '{}' at line {}, column {}", number, line, column)));
                },
                "0o" | "0O" => {
                    // Octal
                    let oct_part = &number[2..];
                    if oct_part.chars().all(|c| c.is_digit(8)) {
                        if let Ok(num) = i64::from_str_radix(oct_part, 8) {
                            return Ok(TokenKind::Integer(num));
                        }
                    }
                    return Err(LexError::IntParseError(format!("Invalid octal literal '{}' at line {}, column {}", number, line, column)));
                },
                "0b" | "0B" => {
                    // Binary
                    let bin_part = &number[2..];
                    if bin_part.chars().all(|c| c.is_digit(2)) {
                        if let Ok(num) = i64::from_str_radix(bin_part, 2) {
                            return Ok(TokenKind::Integer(num));
                        }
                    }
                    return Err(LexError::IntParseError(format!("Invalid binary literal '{}' at line {}, column {}", number, line, column)));
                },
                _ => {}
            }
        }

        // Scientific notation detection
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
                        Err(_) => Err(LexError::FloatParseError(format!("Invalid scientific notation '{}' at line {}, column {}", number, line, column)))
                    }
                }
            }
        }

        // Standard float or integer handling
        if number.contains('.') {
            match number.parse::<f64>() {
                Ok(num) => Ok(TokenKind::Float(num)),
                Err(e) => Err(LexError::FloatParseError(format!("{} at line {}, column {}", e, line, column)))
            }
        } else {
            match number.parse::<i64>() {
                Ok(num) => Ok(TokenKind::Integer(num)),
                Err(e) => Err(LexError::IntParseError(format!("{} at line {}, column {}", e.to_string(), line, column)))
            }
        }
    }
}
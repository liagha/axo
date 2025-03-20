use crate::errors::LexError;
use crate::tokens::{Token, Operator, Punctuation};

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
                column_number += 1;

                match ch {
                    ch if ch.is_whitespace() => continue,

                    ch if ch.is_digit(10) || ch == '.' => {
                        let mut number = ch.to_string();
                        while let Some(&next_ch) = chars.peek() {
                            if next_ch.is_digit(10) || next_ch == '.' {
                                number.push(chars.next().unwrap());
                                column_number += 1;
                            } else if next_ch == '_' {
                                chars.next();
                                column_number += 1;
                            } else {
                                break;
                            }
                        }

                        if number == "." {
                            tokens.push(Token::Operator(Operator::Dot));
                        } else if number == ".." {
                            tokens.push(Token::Operator(Operator::DotDot));
                        } else if number.ends_with("..") {
                            let num_part = number.trim_end_matches("..");
                            if !num_part.is_empty() {
                                tokens.push(Self::lex_number(num_part, line_number + 1, column_number)?);
                            }
                            tokens.push(Token::Operator(Operator::DotDot));
                        } else if number.contains("..") {
                            let parts: Vec<&str> = number.split("..").collect();
                            if parts.len() == 2 {
                                if !parts[0].is_empty() {
                                    tokens.push(Self::lex_number(parts[0], line_number + 1, column_number)?);
                                }
                                tokens.push(Token::Operator(Operator::DotDot));
                                if !parts[1].is_empty() {
                                    tokens.push(Self::lex_number(parts[1], line_number + 1, column_number)?);
                                }
                            } else {
                                return Err(LexError::IntParseError(format!("Invalid range syntax at line {}, column {}", line_number + 1, column_number)));
                            }
                        } else {
                            tokens.push(Self::lex_number(&number, line_number + 1, column_number)?);
                        }
                    }

                    ch if ch.is_alphabetic() || ch == '_' => {
                        let mut name = ch.to_string();
                        while let Some(&next_ch) = chars.peek() {
                            if next_ch.is_alphanumeric() || next_ch == '_' {
                                name.push(chars.next().unwrap());
                                column_number += 1;
                            } else {
                                break;
                            }
                        }

                        match Token::from_str(name.as_str())  {
                            Some(token) => tokens.push(token),
                            _ => tokens.push(Token::Identifier(name)),
                        }
                    }

                    '\'' => {
                        let mut string_content = String::new();
                        let mut closed = false;

                        while let Some(next_ch) = chars.next() {
                            column_number += 1;
                            if next_ch == '\'' {
                                if let Ok(char) = string_content.parse::<char>() {
                                    tokens.push(Token::Char(char));
                                    closed = true;
                                    break;
                                } else {
                                    return Err(LexError::CharParseError(format!("Invalid character literal at line {}, column {}", line_number + 1, column_number)));
                                }
                            } else {
                                string_content.push(next_ch);
                            }
                        }

                        if !closed {
                            tokens.push(Token::Invalid(format!("\"{}", string_content)));
                            return Err(LexError::UnClosedChar(format!("Unclosed character literal at line {}, column {}", line_number + 1, column_number)));
                        }
                    }

                    '"' => {
                        let mut string_content = String::new();
                        let mut closed = false;

                        while let Some(next_ch) = chars.next() {
                            column_number += 1;
                            if next_ch == '"' {
                                tokens.push(Token::Str(string_content.clone()));
                                closed = true;
                                break;
                            } else {
                                string_content.push(next_ch);
                            }
                        }

                        if !closed {
                            tokens.push(Token::Invalid(format!("\"{}", string_content)));
                            return Err(LexError::UnClosedString(format!("Unclosed string literal at line {}, column {}", line_number + 1, column_number)));
                        }
                    }

                    ch if Operator::is_operator(ch) => {
                        let mut operator = ch.to_string();
                        while let Some(&next_ch) = chars.peek() {
                            if Operator::is_operator(next_ch) {
                                operator.push(chars.next().unwrap());
                                column_number += 1;
                            } else {
                                break;
                            }
                        }

                        let op = Operator::from_str(&operator);
                        tokens.push(Token::Operator(op));
                    }

                    ch if Punctuation::is_punctuation(ch) => {
                        let punc = Punctuation::from_str(&*ch.to_string());
                        tokens.push(Token::Punctuation(punc));
                    }

                    _ => {
                        tokens.push(Token::Invalid(ch.to_string()));
                        return Err(LexError::InvalidChar(format!("Invalid character '{}' at line {}, column {}", ch, line_number + 1, column_number)));
                    }
                }
            }

            if line_number != self.input.lines().count() - 1 {
                tokens.push(Token::Punctuation(Punctuation::Newline));
            }
        }

        tokens.push(Token::EOF);

        Ok(tokens)
    }

    fn lex_number(number: &str, line: usize, column: usize) -> Result<Token, LexError> {
        if number.is_empty() {
            return Err(LexError::IntParseError(format!("Empty string at line {}, column {}", line, column)));
        }

        if number.contains(".") {
            match number.parse::<f64>() {
                Ok(num) => Ok(Token::Float(num)),
                Err(e) => Err(LexError::FloatParseError(format!("{} at line {}, column {}", e, line, column)))
            }
        } else {
            match number.parse::<i64>() {
                Ok(num) => Ok(Token::Integer(num)),
                Err(e) => Err(LexError::IntParseError(format!("{} at line {}, column {}", e.to_string(), line, column)))
            }
        }
    }
}

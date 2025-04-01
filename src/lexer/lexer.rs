use crate::lexer::error::{CharParseError, IntParseError, LexError};
use crate::lexer::{OperatorKind, PunctuationKind, TokenKind};
use crate::lexer::{Span, Token};
use std::path::PathBuf;

pub struct Lexer {
    file: PathBuf,
    chars: Vec<char>,
    position: usize,
    pub line: usize,
    pub column: usize,
    pub tokens: Vec<Token>,
}

impl Lexer {
    pub fn new(input: String, file: PathBuf) -> Lexer {
        let chars: Vec<char> = input.chars().collect();

        Lexer {
            file,
            chars,
            position: 0,
            line: 1,
            column: 0,
            tokens: Vec::new(),
        }
    }

    fn peek(&self) -> Option<char> {
        if self.position < self.chars.len() {
            Some(self.chars[self.position])
        } else {
            None
        }
    }

    #[allow(dead_code)]
    fn peek_ahead(&self, n: usize) -> Option<char> {
        let pos = self.position + n;

        if pos < self.chars.len() {
            Some(self.chars[pos])
        } else {
            None
        }
    }

    fn next(&mut self) -> Option<char> {
        if self.position < self.chars.len() {
            let ch = self.chars[self.position];

            self.position += 1;

            if ch == '\n' {
                self.line += 1;
                self.column = 0;
            } else {
                self.column += 1;
            }

            Some(ch)
        } else {
            None
        }
    }

    fn create_span(&self, start: (usize, usize), end: (usize, usize)) -> Span {
        Span {
            start,
            end,
            file: self.file.clone(),
        }
    }

    fn push_token(&mut self, kind: TokenKind, span: Span) {
        self.tokens.push(Token { kind, span });
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

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        while let Some(ch) = self.peek() {
            match ch {
                ch if ch.is_whitespace() && ch != '\n' => {
                    self.next();

                    continue
                },

                ch if ch.is_digit(10) || ch == '.' => self.handle_number()?,

                ch if ch.is_alphabetic() || ch == '_' => self.handle_identifier()?,

                '\'' => self.handle_character()?,

                '"' => self.handle_string()?,

                '/' => {
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
                }

                ch if OperatorKind::is_operator(ch) => {
                    self.next();

                    let mut operator = Vec::new();
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
                }

                ch if PunctuationKind::is_punctuation(ch) => {
                    self.next();

                    let start = (self.line, self.column);
                    let end = (self.line, self.column);
                    let span = self.create_span(start, end);

                    self.push_token(TokenKind::Punctuation(PunctuationKind::from_char(&ch)), span);
                }

                _ => {
                    self.next();

                    let start = (self.line, self.column);
                    let end = (self.line, self.column);
                    let span = self.create_span(start, end);

                    self.push_token(TokenKind::Invalid(ch.to_string()), span);
                    return Err(LexError::InvalidChar);
                }
            }
        }

        let file_end = (self.line, self.column);
        let span = self.create_span(file_end, file_end);

        self.push_token(TokenKind::EOF, span);

        Ok(self.tokens.clone())
    }

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

    fn handle_number(&mut self) -> Result<(), LexError> {
        let ch = self.next().unwrap();
        let mut number = ch.to_string();
        let start = (self.line, self.column);

        while let Some(ch) = self.peek() {
            match ch {
                ch if ch.is_digit(10) || ch ==  '.' => {
                    let digit = self.next().unwrap();

                    number.push(digit);
                }
                '.' => {
                    let number_end = (self.line, self.column);
                    self.next();

                    if let Some(next_ch) = self.peek() {
                        if next_ch == '.' {
                            let dot_pos = (self.line, self.column);

                            let kind = Self::lex_number(&number)?;
                            self.tokens.push(Token {
                                kind,
                                span: self.create_span(start, number_end),
                            });

                            self.next();
                            let op_end = (self.line, self.column);
                            self.tokens.push(Token {
                                kind: TokenKind::Operator(OperatorKind::DotDot),
                                span: self.create_span(dot_pos, op_end),
                            });

                            return Ok(());
                        }
                    }

                    number.push('.');
                }
                '_' => {
                    self.next();
                }
                _ => break,
            }
        }

        let number_end = (self.line, self.column);
        let operator = OperatorKind::from_str(&*number);

        if operator != OperatorKind::Unknown {
            self.tokens.push(Token {
                kind: TokenKind::Operator(operator),
                span: self.create_span(start, number_end),
            });
        } else {
            let kind = Self::lex_number(&number)?;
            self.tokens.push(Token {
                kind,
                span: self.create_span(start, number_end),
            });
        }

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
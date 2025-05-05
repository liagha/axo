use {
    std::path::PathBuf,

    crate::{
        axo_lexer::{
            Token,
            number::NumberLexer,
            handler::Handler,
            literal::LiteralLexer,
            operator::OperatorLexer,
            punctuation::PunctuationLexer,
            symbol::SymbolLexer,
            error::ErrorKind,
            LexError, TokenKind,
        },

        axo_rune::{
            unicode::{
                is_alphabetic, is_numeric, is_white_space,
            },
        },

        axo_span::{
            Span,
            position::Position,
        },
    }
};

pub struct Lexer {
    pub input: Vec<char>,
    pub position: Position,
    pub output: Vec<Token>,
}

impl Lexer {
    pub fn new(input: String, file: PathBuf) -> Lexer {
        let chars: Vec<char> = input.chars().collect();

        Lexer {
            input: chars,
            position: Position::new(file),
            output: Vec::new(),
        }
    }

    pub fn peek(&self) -> Option<char> {
        if self.position.index < self.input.len() {
            Some(self.input[self.position.index])
        } else {
            None
        }
    }

    pub fn peek_ahead(&self, n: usize) -> Option<char> {
        let pos = self.position.index + n;

        if pos < self.input.len() {
            Some(self.input[pos])
        } else {
            None
        }
    }

    pub fn next(&mut self) -> Option<char> {
        if self.position.index < self.input.len() {
            let ch = self.input[self.position.index];

            self.position.index += 1;

            if ch == '\n' {
                self.position.line += 1;
                self.position.column = 0;
            } else {
                self.position.column += 1;
            }

            Some(ch)
        } else {
            None
        }
    }

    pub fn create_span(&self, start: (usize, usize), end: (usize, usize)) -> Span {
        Span {
            start,
            end,
            file: self.position.file.clone(),
        }
    }

    pub fn push_token(&mut self, kind: TokenKind, span: Span) {
        self.output.push(Token { kind, span });
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        while let Some(ch) = self.peek() {
            match ch {
                ch if is_white_space(ch) && ch != '\n' => {
                    self.next();

                    continue
                },

                ch if is_numeric(ch) => self.handle_number()?,

                ch if is_alphabetic(ch) || ch == '_' => self.handle_identifier()?,

                '.' => {
                    if let Some(ch) = self.peek_ahead(1) {
                        if ch.is_digit(10) {
                            self.handle_number()?;
                        } else {
                            self.handle_operator();
                        }
                    }
                },

                '\'' => self.handle_character()?,

                '"' | '`' => self.handle_string()?,

                '/' => self.handle_comment()?,

                ch if ch.is_operator() => self.handle_operator(),

                ch if ch.is_punctuation() => self.handle_punctuation(),

                _ => {
                    self.next();

                    let start = (self.position.line, self.position.column);
                    let end = (self.position.line, self.position.column);
                    let span = self.create_span(start, end);

                    return Err(LexError::new(ErrorKind::InvalidChar, span));
                }
            }
        }

        Ok(self.output.clone())
    }
}
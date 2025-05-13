use {
    crate::Path,

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

        axo_data::peekable::Peekable,
    }
};

#[derive(Clone)]
pub struct Lexer {
    pub input: Vec<char>,
    pub index: usize,
    pub position: Position,
    pub output: Vec<Token>,
    pub errors: Vec<LexError>,
}

impl Peekable<char> for Lexer {
    fn peek_ahead(&self, n: usize) -> Option<&char> {
        let current = self.index + n;

        if current < self.input.len() {
            Some(&self.input[current])
        } else {
            None
        }
    }
    
    fn peek_behind(&self, n: usize) -> Option<&char> {
        let mut current = self.index;

        if current < n {
            return None;
        }
        
        current -= n;

        if current < self.input.len() {
            Some(&self.input[current])
        } else {
            None
        }
    }

    fn next(&mut self) -> Option<char> {
        if self.index < self.input.len() {
            let ch = self.input[self.index];

            self.index += 1;

            if ch == '\n' {
                self.position.line += 1;
                self.position.column = 1;
            } else {
                self.position.column += 1;
            }

            Some(ch)
        } else {
            None
        }
    }

    fn position(&self) -> Position {
        self.position.clone()
    }

    fn set_index(&mut self, index: usize) {
        self.index = index
    }

    fn set_line(&mut self, line: usize) {
        self.position.line = line
    }

    fn set_column(&mut self, column: usize) {
        self.position.column = column
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }
}

impl Lexer {
    pub fn new(input: String, file: Path) -> Lexer {
        let chars: Vec<char> = input.chars().collect();

        Lexer {
            input: chars,
            index: 0,
            position: Position::new(file),
            output: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn create_span(&self, start: (usize, usize), end: (usize, usize)) -> Span {
        let file = self.position.file.clone();
        
        let start = Position {
            line: start.0,
            column: start.1,
            file: file.clone(),
        };

        let end = Position {
            line: end.0,
            column: end.1,
            file,
        };

        Span {
            start,
            end,
        }
    }

    pub fn push_token(&mut self, kind: TokenKind, span: Span) {
        self.output.push(Token { kind, span });
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        while let Some(ch) = self.peek() {
            match *ch {
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
use crate::axo_lexer::error::{Error, ErrorKind};
use crate::axo_lexer::{Lexer, OperatorKind, TokenKind};

pub trait Handler {
    fn handle_identifier(&mut self) -> Result<(), Error>;
    fn handle_comment(&mut self) -> Result<(), Error>;
}

impl Handler for Lexer {
    fn handle_identifier(&mut self) -> Result<(), Error> {
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

    fn handle_comment(&mut self) -> Result<(), Error> {
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
                        return Err(Error::new(ErrorKind::UnClosedComment, span));
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

        Ok(())
    }
}
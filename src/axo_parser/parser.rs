#![allow(dead_code)]

use {
    super::{error::ErrorKind, Element, ElementKind, ItemKind, ParseError},
    crate::{
        axo_cursor::{Peekable, Position, Span},
        axo_form::{form::FormKind, former::Former},
        axo_scanner::{OperatorKind, PunctuationKind, Token, TokenKind},
        compiler::{Context, Marked},
        Path,
    },
};

#[derive(Clone)]
pub struct Parser {
    pub context: Context,
    pub index: usize,
    pub position: Position,
    pub input: Vec<Token>,
    pub output: Vec<Element>,
    pub errors: Vec<ParseError>,
}

impl Peekable<Token> for Parser {
    #[inline]
    fn len(&self) -> usize {
        self.input.len()
    }

    fn peek_ahead(&self, forward: usize) -> Option<&Token> {
        let mut current = self.index;
        let mut found = 0;

        while let Some(token) = self.get(current) {
            if Self::is_skippable(&token.kind) {
                current += 1;
                continue;
            }

            if found == forward {
                return Some(token);
            }

            found += 1;
            current += 1;
        }

        None
    }

    fn peek_behind(&self, backward: usize) -> Option<&Token> {
        if self.index == 0 {
            return None;
        }

        let mut current = self.index - 1;
        let mut found = 0;

        loop {
            if current >= self.input.len() {
                if current == 0 {
                    break;
                }
                current -= 1;
                continue;
            }

            let token = &self.input[current];

            if Self::is_skippable(&token.kind) {
                if current == 0 {
                    break;
                }
                current -= 1;
                continue;
            }

            if found == backward {
                return Some(token);
            }

            found += 1;

            if current == 0 {
                break;
            }
            current -= 1;
        }

        None
    }

    fn restore(&mut self) {
        self.set_position(Position {
            line: 1,
            column: 1,
            path: self.position.path.clone(),
        })
    }

    fn next(&mut self, position: &mut Position) -> Option<Token> {
        while let Some(token) = self.get(self.index).cloned() {
            *position = token.span.end.clone();
            
            if Self::is_skippable(&token.kind) {
                self.index += 1;
                
                continue;
            }
            
            return Some(token);
        }

        None
    }

    fn input(&self) -> &[Token] {
        self.input.as_slice()
    }

    fn input_mut(&mut self) -> &mut [Token] {
        &mut self.input
    }

    fn position(&self) -> Position {
        self.position.clone()
    }

    fn position_mut(&mut self) -> &mut Position {
        &mut self.position
    }

    fn index(&self) -> usize {
        self.index
    }

    fn index_mut(&mut self) -> &mut usize {
        &mut self.index
    }
}

impl Parser {
    fn is_skippable(kind: &TokenKind) -> bool {
        match kind {
            TokenKind::Punctuation(PunctuationKind::Newline) => true,
            TokenKind::Punctuation(PunctuationKind::Space) => true,
            TokenKind::Punctuation(PunctuationKind::Indentation(_)) => true,
            TokenKind::Character(_) => true,
            _ => false,
        }
    }
}

impl Parser {
    pub fn new(context: Context, tokens: Vec<Token>, file: Path) -> Self {
        Parser {
            context,
            input: tokens,
            index: 0,
            position: Position::new(file),
            output: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> (Vec<Element>, Vec<ParseError>) {
        let mut elements = Vec::new();
        let mut errors = Vec::new();

        while self.peek().is_some() {
            let forms = self.form(Self::parser()).expand();

            for form in forms {
                match form.kind {
                    FormKind::Output(element) => {
                        elements.push(element);
                    }

                    FormKind::Failure(error) => {
                        errors.push(error);
                    }

                    FormKind::Multiple(_) | FormKind::Blank | FormKind::Input(_) => {}
                }
            }
        }

        (elements, errors)
    }
}

impl Marked for Parser {
    fn context(&self) -> &Context {
        &self.context
    }

    fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }
}

use {
    super::{error::ErrorKind, Element, ElementKind, SymbolKind, ParseError},
    crate::{
        format::Debug,
        hash::Hash,
        axo_cursor::{Peekable, Position, Span},
        axo_form::{
            pattern::Classifier,
            form::FormKind,
            former::Former,
        },
        axo_scanner::{OperatorKind, PunctuationKind, Token, TokenKind},
        compiler::{Context, Marked},
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

    fn peek_ahead(&self, n: usize) -> Option<&Token> {
        let current = self.index + n;

        self.get(current)
    }

    fn peek_behind(&self, n: usize) -> Option<&Token> {
        let current = self.index - n;

        self.get(current)
    }

    fn restore(&mut self) {
        self.set_position(Position {
            line: 1,
            column: 1,
            location: self.position.location,
        })
    }

    fn next(&self, index: &mut usize, position: &mut Position) -> Option<Token> {
        if let Some(token) = self.get(*index) {
            *position = token.span.end;

            *index += 1;

            return Some(token.clone());
        }

        None
    }

    fn input(&self) -> &Vec<Token> {
        &self.input
    }

    fn input_mut(&mut self) -> &mut Vec<Token> {
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
    pub fn new(context: Context, tokens: Vec<Token>, file: &'static str) -> Self {
        Parser {
            context,
            input: tokens,
            index: 0,
            position: Position::new(file),
            output: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn strainer() -> Classifier<Token, Element, ParseError> {
        Classifier::persistence(
            Classifier::predicate(|token: &Token| {
                !matches!(token.kind,
                    TokenKind::Punctuation(PunctuationKind::Newline)
                    | TokenKind::Punctuation(PunctuationKind::Tab)
                    | TokenKind::Punctuation(PunctuationKind::Space)
                    | TokenKind::Punctuation(PunctuationKind::Indentation(_))
                    | TokenKind::Punctuation(PunctuationKind::Semicolon)
                    | TokenKind::Comment(_)
                )
            }),
            0,
            None
        )
    }

    pub fn parse(&mut self) -> (Vec<Element>, Vec<ParseError>) {
        let mut elements = Vec::new();
        let mut errors = Vec::new();

        self.strain(Self::strainer());
        
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

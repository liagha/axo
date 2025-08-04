use {
    super::{
        Element, ParseError
    },
    crate::{
        axo_cursor::{
            Peekable, Position, Location
        },
        axo_form::{
            form::Form,
            former::Former,
            classifier::Classifier,
        },
        axo_internal::compiler::{
            Registry, Marked
        },
        axo_scanner::{
            PunctuationKind, Token, TokenKind
        },
        hash::Hash,
    },
};

pub struct Parser<'parser> {
    pub registry: &'parser mut Registry<'parser>,
    pub index: usize,
    pub position: Position<'parser>,
    pub input: Vec<Token<'parser>>,
    pub output: Vec<Element<'parser>>,
    pub errors: Vec<ParseError<'parser>>,
}

impl<'parser> Peekable<'parser, Token<'parser>> for Parser<'parser> {
    #[inline]
    fn length(&self) -> usize {
        self.input.len()
    }

    fn peek_ahead(&self, n: usize) -> Option<&Token<'parser>> {
        let current = self.index + n;

        self.get(current)
    }

    fn peek_behind(&self, n: usize) -> Option<&Token<'parser>> {
        let current = self.index - n;

        self.get(current)
    }

    fn next(&self, index: &mut usize, position: &mut Position<'parser>) -> Option<Token<'parser>> {
        if let Some(token) = self.get(*index) {
            *position = token.span.end;

            *index += 1;

            return Some(token.clone());
        }

        None
    }

    fn input(&self) -> &Vec<Token<'parser>> {
        &self.input
    }

    fn input_mut(&mut self) -> &mut Vec<Token<'parser>> {
        &mut self.input
    }

    fn position(&self) -> Position<'parser> {
        self.position.clone()
    }

    fn position_mut(&mut self) -> &mut Position<'parser> {
        &mut self.position
    }

    fn index(&self) -> usize {
        self.index
    }

    fn index_mut(&mut self) -> &mut usize {
        &mut self.index
    }
}

impl<'parser> Parser<'parser> {
    pub fn new(registry: &'parser mut Registry<'parser>, location: Location<'parser>) -> Self {
        Parser {
            registry,
            index: 0,
            position: Position::new(location),
            input: Vec::new(),
            output: Vec::new(),
            errors: Vec::new(),
        }
    }
}

impl Parser<'static> {
    pub fn with_input(self, input: Vec<Token<'static>>) -> Self {
        Self {
            input,
            ..self
        }
    }

    pub fn set_input(&mut self, input: Vec<Token<'static>>) {
        self.input = input;
    }

    pub fn strainer(length: usize) -> Classifier<'static, Token<'static>, Element<'static>, ParseError<'static>> {
        Classifier::repetition(
            Classifier::alternative([
                Classifier::predicate(|token: &Token| {
                    matches!(token.kind,
                    TokenKind::Punctuation(PunctuationKind::Newline)
                    | TokenKind::Punctuation(PunctuationKind::Tab)
                    | TokenKind::Punctuation(PunctuationKind::Space)
                    | TokenKind::Punctuation(PunctuationKind::Indentation(_))
                    | TokenKind::Punctuation(PunctuationKind::Semicolon)
                    | TokenKind::Comment(_)
                )
                }).with_ignore(),
                Classifier::predicate(|token: &Token| {
                    !matches!(token.kind,
                    TokenKind::Punctuation(PunctuationKind::Newline)
                    | TokenKind::Punctuation(PunctuationKind::Tab)
                    | TokenKind::Punctuation(PunctuationKind::Space)
                    | TokenKind::Punctuation(PunctuationKind::Indentation(_))
                    | TokenKind::Punctuation(PunctuationKind::Semicolon)
                    | TokenKind::Comment(_)
                )
                })
            ]),
            0,
            Some(length)
        )
    }

    pub fn parse(&mut self) {
        let strained = self.form(Self::strainer(self.length())).collect_inputs();
        self.input = strained;
        self.reset();

        while self.peek().is_some() {
            let forms = self.form(Self::parser()).flatten();

            for form in forms {
                match form {
                    Form::Output(output) => {
                        self.output.push(output);
                    }

                    Form::Failure(failure) => {
                        self.errors.push(failure);
                    }

                    Form::Multiple(_) | Form::Blank | Form::Input(_) => {}
                }
            }
        }
    }
}

impl<'parser> Marked<'parser> for Parser<'parser> {
    fn registry(&self) -> &Registry<'parser> {
        &self.registry
    }

    fn registry_mut(&mut self) -> &mut Registry<'parser> {
        &mut self.registry
    }
}

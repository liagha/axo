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
            pattern::Classifier,
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

#[derive(Clone)]
pub struct Parser {
    pub registry: Registry,
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
    pub fn new(registry: Registry, tokens: Vec<Token>, location: Location) -> Self {
        Parser {
            registry,
            input: tokens,
            index: 0,
            position: Position::new(location),
            output: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn strainer(length: usize) -> Classifier<Token, Element, ParseError> {
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
        let strained = self.form(Self::strainer(self.len())).collect_inputs();
        self.input = strained;
        self.restore();

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

impl Marked for Parser {
    fn registry(&self) -> &Registry {
        &self.registry
    }

    fn registry_mut(&mut self) -> &mut Registry {
        &mut self.registry
    }
}

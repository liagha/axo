use crate::format::Debug;
use crate::axo_cursor::{Location, Peekable, Position, Span};
use crate::axo_form::form::Form;
use crate::axo_form::former::Former;
use crate::axo_form::pattern::Classifier;
use crate::axo_initial::InitialError;
use crate::axo_parser::{Element, ParseError, Symbolic};
use crate::axo_scanner::{OperatorKind, PunctuationKind, Token, TokenKind};
use crate::compiler::{Registry, Marked};
use crate::hash::Hash;

pub struct Initializer {
    pub registry: Registry,
    pub index: usize,
    pub position: Position,
    pub input: Vec<Token>,
    pub output: Vec<Preference>,
    pub errors: Vec<InitialError>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Preference {
    Verbosity(bool),
    Path(String),
}

impl Symbolic for Preference {
    fn brand(&self) -> Option<Token> {
        let label = match self {
            Preference::Verbosity(_) => {
                "Verbosity".to_string()
            }
            Preference::Path(_) => {
                "Path".to_string()
            }
        };

        Some(Token::new(TokenKind::Identifier(label), Span::default()))
    }
}

impl Peekable<Token> for Initializer {
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

impl Initializer {
    pub fn new(registry: Registry, tokens: Vec<Token>, location: Location) -> Self {
        Initializer {
            registry,
            input: tokens,
            index: 0,
            position: Position::new(location),
            output: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn verbosity() -> Classifier<Token, Preference, InitialError> {
        Classifier::sequence([
            Classifier::predicate(|token: &Token| {
                if let TokenKind::Operator(operator) = &token.kind {
                    if *operator == OperatorKind::Minus {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }),
            Classifier::predicate(|token: &Token| {
                if let TokenKind::Identifier(identifier) = &token.kind {
                    if identifier == "v" {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
        ]).with_transform(|_, _form| {
            Ok(Form::output(Preference::Verbosity(true)))
        })
    }

    pub fn path() -> Classifier<Token, Preference, InitialError> {
        Classifier::with_transform(
            Classifier::sequence([
                // Match "-p"
                Classifier::predicate(|token: &Token| {
                    if let TokenKind::Operator(operator) = &token.kind {
                        *operator == OperatorKind::Minus
                    } else {
                        false
                    }
                }),
                Classifier::predicate(|token: &Token| {
                    if let TokenKind::Identifier(identifier) = &token.kind {
                        identifier == "p"
                    } else {
                        false
                    }
                }),
                // Match the path components
                Classifier::sequence([
                    // First path component (required)
                    Classifier::predicate(|token: &Token| {
                        matches!(token.kind, TokenKind::Identifier(_))
                    }),
                    // Repeated "/component" parts
                    Classifier::repetition(
                        Classifier::sequence([
                            Classifier::predicate(|token: &Token| {
                                matches!(token.kind, TokenKind::Operator(OperatorKind::Slash))
                            }),
                            Classifier::predicate(|token: &Token| {
                                matches!(token.kind, TokenKind::Identifier(_))
                            }),
                        ]),
                        0,
                        None
                    ),
                    // Optional file extension ".ext"
                    Classifier::sequence([
                        Classifier::predicate(|token: &Token| {
                            matches!(token.kind, TokenKind::Operator(OperatorKind::Dot))
                        }),
                        Classifier::predicate(|token: &Token| {
                            matches!(token.kind, TokenKind::Identifier(_))
                        }),
                    ]).as_optional()
                ])
            ]),
            |_, form| {
                let inputs = form.collect_inputs();
                let mut path = String::new();

                for input in inputs.iter().skip(2) {
                    match &input.kind {
                        TokenKind::Identifier(identifier) => {
                            path.push_str(identifier);
                        }
                        TokenKind::Operator(OperatorKind::Slash) => {
                            path.push('/');
                        }
                        TokenKind::Operator(OperatorKind::Dot) => {
                            path.push('.');
                        }
                        _ => {}
                    }
                }

                Ok(Form::output(Preference::Path(path)))
            }
        )
    }

    pub fn strainer(length: usize) -> Classifier<Token, Element, ParseError> {
        Classifier::continuous(
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
            Some(length)
        )
    }

    pub fn preference() -> Classifier<Token, Preference, InitialError> {
        Classifier::alternative([
            Self::path(),
            Self::verbosity(),
            Classifier::anything().with_ignore(),
        ])
    }

    pub fn classifier() -> Classifier<Token, Preference, InitialError> {
        Classifier::repetition(
            Self::preference(),
            0,
            None
        )
    }

    pub fn initialize(&mut self) {
        self.input = self.input.iter().cloned().filter(|token| {
            !matches!(token.kind,
                    TokenKind::Punctuation(PunctuationKind::Newline)
                    | TokenKind::Punctuation(PunctuationKind::Tab)
                    | TokenKind::Punctuation(PunctuationKind::Space)
                    | TokenKind::Punctuation(PunctuationKind::Indentation(_))
                    | TokenKind::Punctuation(PunctuationKind::Semicolon)
                    | TokenKind::Comment(_)
                )
        }).collect::<Vec<_>>();

        while self.peek().is_some() {
            let forms = self.form(Self::classifier()).flatten();

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

impl Marked for Initializer {
    fn registry(&self) -> &Registry {
        &self.registry
    }

    fn registry_mut(&mut self) -> &mut Registry {
        &mut self.registry
    }
}
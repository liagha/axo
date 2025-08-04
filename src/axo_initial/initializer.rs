use {
    super::{
        InitialError,
    },
    crate::{
        axo_cursor::{Location, Peekable, Position, Span, Spanned},
        axo_form::{
            form::Form,
            former::Former,
            classifier::Classifier,
        },
        axo_parser::{Element, ParseError, Symbolic, Symbol},
        axo_scanner::{OperatorKind, PunctuationKind, Token, TokenKind, Scanner},
        compiler::{Registry, Marked},
        format::Debug,
        hash::Hash,
    },
};

#[derive(Debug)]
pub struct Initializer<'initializer> {
    pub registry: &'initializer mut Registry<'initializer>,
    pub index: usize,
    pub position: Position<'initializer>,
    pub input: Vec<Token<'initializer>>,
    pub output: Vec<Preference>,
    pub errors: Vec<InitialError<'initializer>>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Preference {
    pub target: Token<'static>,
    pub value: Token<'static>,
    pub span: Span<'static>,
}

impl Spanned<'static> for Preference {
    fn borrow_span(&self) -> Span<'static> {
        self.span.clone()
    }

    fn span(self) -> Span<'static> {
        self.span
    }
}

impl Preference {
    pub fn new(target: Token<'static>, value: Token<'static>) -> Self {
        let span = Span::merge(&target.borrow_span(), &value.borrow_span());

        Self {
            target,
            value,
            span
        }
    }
}

impl Symbolic for Preference {
    fn brand(&self) -> Option<Token<'static>> {
        Some(self.target.clone())
    }
}

impl<'initializer> Peekable<'initializer, Token<'initializer>> for Initializer<'initializer> {
    #[inline]
    fn length(&self) -> usize {
        self.input.len()
    }

    fn peek_ahead(&self, n: usize) -> Option<&Token<'initializer>> {
        let current = self.index + n;
        self.get(current)
    }

    fn peek_behind(&self, n: usize) -> Option<&Token<'initializer>> {
        let current = self.index - n;
        self.get(current)
    }

    fn next(&self, index: &mut usize, position: &mut Position<'initializer>) -> Option<Token<'initializer>> {
        if let Some(token) = self.get(*index) {
            *position = token.span.end;
            *index += 1;
            return Some(token.clone());
        }
        None
    }

    fn input(&self) -> &Vec<Token<'initializer>> {
        &self.input
    }

    fn input_mut(&mut self) -> &mut Vec<Token<'initializer>> {
        &mut self.input
    }

    fn position(&self) -> Position<'initializer> {
        self.position.clone()
    }

    fn position_mut(&mut self) -> &mut Position<'initializer> {
        &mut self.position
    }

    fn index(&self) -> usize {
        self.index
    }

    fn index_mut(&mut self) -> &mut usize {
        &mut self.index
    }
}

impl<'initializer> Initializer<'initializer> {
    pub fn new(registry: &'initializer mut Registry<'initializer>, location: Location<'initializer>) -> Self {
        Initializer {
            registry,
            index: 0,
            position: Position::new(location),
            input: Vec::new(),
            output: Vec::new(),
            errors: Vec::new(),
        }
    }
}

impl Initializer<'static> {
    pub fn verbosity() -> Classifier<'static, Token<'static>, Preference, InitialError<'static>> {
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
            }).with_ignore(),
            Classifier::predicate(|token: &Token| {
                if let TokenKind::Identifier(identifier) = &token.kind {
                    identifier == "v" || identifier == "verbose"
                } else {
                    false
                }
            }).with_transform(|_, form| {
                let identifier = form.collect_inputs()[0].clone();

                Ok(Form::Input(Token::new(TokenKind::Identifier("Verbosity".to_string()), identifier.borrow_span())))
            })
        ]).with_transform(|_, form| {
            let identifier = form.collect_inputs()[0].clone();
            Ok(Form::output(Preference::new(identifier.clone(), Token::new(TokenKind::Boolean(true), identifier.borrow_span()))))
        })
    }

    pub fn path() -> Classifier<'static, Token<'static>, Preference, InitialError<'static>> {
        Classifier::with_transform(
            Classifier::sequence([
                Classifier::predicate(|token: &Token| {
                    if let TokenKind::Operator(operator) = &token.kind {
                        *operator == OperatorKind::Minus
                    } else {
                        false
                    }
                }).with_ignore(),
                Classifier::predicate(|token: &Token| {
                    if let TokenKind::Identifier(identifier) = &token.kind {
                        identifier == "p" || identifier == "path"
                    } else {
                        false
                    }
                }).with_transform(|_, form| {
                    let identifier = form.collect_inputs()[0].clone();

                    Ok(Form::Input(Token::new(TokenKind::Identifier("Path".to_string()), identifier.borrow_span())))
                }),
                Classifier::sequence([
                    Classifier::predicate(|token: &Token| {
                        matches!(token.kind, TokenKind::Identifier(_))
                    }),
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
                let identifier = inputs[0].clone();
                let mut path = String::new();

                for input in inputs.iter().skip(1) {
                    match &input.kind {
                        TokenKind::Identifier(identifier) => {
                            path.push_str(&identifier);
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

                Ok(Form::output(Preference::new(identifier.clone(), Token::new(TokenKind::Identifier(path), identifier.borrow_span()))))
            }
        )
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

    pub fn preference() -> Classifier<'static, Token<'static>, Preference, InitialError<'static>> {
        Classifier::alternative([
            Self::path(),
            Self::verbosity(),
            Classifier::anything().with_ignore(),
        ])
    }

    pub fn classifier() -> Classifier<'static, Token<'static>, Preference, InitialError<'static>> {
        Classifier::repetition(
            Self::preference(),
            0,
            None
        )
    }

    pub fn initialize(&mut self) {
        let location = Location::Flag;
        let input = location.get_value();

        let mut registry = Registry::new();
        let mut scanner = Scanner::new(&mut registry, Location::Flag).with_input(input);
        scanner.scan();

        self.input = scanner.output;
        self.reset();

        let strained = self.form(Self::strainer(self.length())).collect_inputs();
        self.input = strained;
        self.reset();

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

        let preferences = self.output.iter().map(|preference| {
            Symbol::new(preference.clone(), preference.borrow_span())
        }).collect::<Vec<Symbol>>();

        self.registry.resolver.extend(preferences);
    }
}

impl<'initializer> Marked<'initializer> for Initializer<'initializer> {
    fn registry(&self) -> &Registry<'initializer> {
        &self.registry
    }

    fn registry_mut(&mut self) -> &mut Registry<'initializer> {
        &mut self.registry
    }
}
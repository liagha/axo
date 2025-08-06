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
        axo_parser::{Element, ParseError, Symbol},
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
    pub output: Vec<Preference<'initializer>>,
    pub errors: Vec<InitialError<'initializer>>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Preference<'preference> {
    pub target: Token<'preference>,
    pub value: Token<'preference>,
    pub span: Span<'preference>,
}

impl<'preference> Spanned<'preference> for Preference<'preference> {
    fn borrow_span(&self) -> Span<'preference> {
        self.span.clone()
    }

    fn span(self) -> Span<'preference> {
        self.span
    }
}

impl<'preference> Preference<'preference> {
    pub fn new(target: Token<'preference>, value: Token<'preference>) -> Self {
        let span = Span::merge(&target.borrow_span(), &value.borrow_span());

        Self {
            target,
            value,
            span
        }
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

    pub fn verbosity() -> Classifier<'initializer, Token<'initializer>, Preference<'initializer>, InitialError<'initializer>> {
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
            }).with_transform(move |_, form: Form<'initializer, Token<'initializer>, Preference, InitialError<'initializer>>| {
                let identifier = form.collect_inputs()[0].clone();
                let span = identifier.span();

                Ok(Form::Input(Token::new(TokenKind::Identifier("Verbosity".to_string()), span)))
            })
        ]).with_transform(move |_, form: Form<'initializer, Token<'initializer>, Preference, InitialError<'initializer>>| {
            let identifier: Token<'initializer> = form.collect_inputs()[0].clone();
            let span: Span<'initializer> = identifier.clone().span();

            Ok(Form::output(Preference::new(identifier, Token::new(TokenKind::Boolean(true), span))))
        })
    }

    pub fn path() -> Classifier<'initializer, Token<'initializer>, Preference<'initializer>, InitialError<'initializer>> {
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
                }).with_transform(|_, form: Form<'initializer, Token<'initializer>, Preference, InitialError<'initializer>>| {
                    let identifier = form.collect_inputs()[0].clone();
                    let span = identifier.span();

                    Ok(Form::Input(Token::new(TokenKind::Identifier("Path".to_string()), span)))
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
            |_, form: Form<'initializer, Token<'initializer>, Preference, InitialError<'initializer>>| {
                let inputs = form.collect_inputs();
                let identifier = inputs[0].clone();
                let span = identifier.clone().span();
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

                Ok(Form::output(Preference::new(identifier.clone(), Token::new(TokenKind::Identifier(path), span))))
            }
        )
    }

    pub fn strainer(length: usize) -> Classifier<'initializer, Token<'initializer>, Element<'initializer>, ParseError<'initializer>> {
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

    pub fn preference() -> Classifier<'initializer, Token<'initializer>, Preference<'initializer>, InitialError<'initializer>> {
        Classifier::alternative([
            Self::path(),
            Self::verbosity(),
            Classifier::anything().with_ignore(),
        ])
    }

    pub fn classifier() -> Classifier<'initializer, Token<'initializer>, Preference<'initializer>, InitialError<'initializer>> {
        Classifier::repetition(
            Self::preference(),
            0,
            None
        )
    }

    pub fn initialize(&mut self) {
        let location = Location::Flag;
        let input = location.get_value();

        let tokens = {
            let registry_ptr = self.registry as *mut Registry<'initializer>;
            let mut scanner = unsafe {
                Scanner::new(&mut *registry_ptr, Location::Flag).with_input(input)
            };
            scanner.scan();
            scanner.output
        };

        self.input = tokens;
        self.reset();

        let strained = {
            let length = self.length();
            let classifier = Self::strainer(length);
            self.form(classifier).collect_inputs()
        };

        self.input = strained;
        self.reset();

        let mut preferences = Vec::new();
        while self.peek().is_some() {
            let classifier = Self::classifier();
            let forms = self.form(classifier).flatten();
            for form in forms {
                match form {
                    Form::Output(output) => preferences.push(output),
                    Form::Failure(failure) => self.errors.push(failure),
                    _ => {}
                }
            }
        }

        let symbols = preferences.into_iter().map(|preference| {
            let span = preference.borrow_span();
            Symbol::new(preference, span)
        }).collect::<Vec<Symbol>>();

        self.registry.resolver.extend(symbols);
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
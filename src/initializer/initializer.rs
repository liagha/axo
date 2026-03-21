use {
    crate::{
        initializer::InitializeError,
        parser::{Element, ElementKind, Symbol, SymbolKind, ParseError},
        data::{Binding, Offset, Scale, Str},
        formation::{Classifier, Form, Former},
        scanner::{PunctuationKind, Scanner, Token, TokenKind},
        tracker::{Location, Peekable, Position},
    },
};

pub struct Initializer<'initializer> {
    pub index: Offset,
    pub position: Position<'initializer>,
    pub input: Vec<Token<'initializer>>,
    pub output: Vec<Symbol<'initializer>>,
    pub errors: Vec<InitializeError<'initializer>>,
}

impl<'initializer> Peekable<'initializer, Token<'initializer>> for Initializer<'initializer> {
    #[inline]
    fn length(&self) -> Scale {
        self.input.len()
    }

    fn peek_ahead(&self, n: Offset) -> Option<&Token<'initializer>> {
        let current = self.index + n;
        self.get(current)
    }

    fn peek_behind(&self, n: Offset) -> Option<&Token<'initializer>> {
        self.index
            .checked_sub(n)
            .and_then(|current| self.get(current))
    }

    fn next(
        &self,
        index: &mut Offset,
        position: &mut Position<'initializer>,
    ) -> Option<Token<'initializer>> {
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

    fn index(&self) -> Offset {
        self.index
    }

    fn index_mut(&mut self) -> &mut Offset {
        &mut self.index
    }
}

impl<'initializer> Initializer<'initializer> {
    pub fn new(location: Location<'initializer>) -> Self {
        Initializer {
            index: 0,
            position: Position::new(location),
            input: Vec::new(),
            output: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn filter(
        length: Scale,
    ) -> Classifier<
        'initializer,
        Token<'initializer>,
        Element<'initializer>,
        ParseError<'initializer>,
    > {
        Classifier::repetition(
            Classifier::alternative([
                Classifier::predicate(|token: &Token| {
                    matches!(
                        token.kind,
                        TokenKind::Punctuation(PunctuationKind::Newline)
                            | TokenKind::Punctuation(PunctuationKind::Tab)
                            | TokenKind::Punctuation(PunctuationKind::Space)
                            | TokenKind::Punctuation(PunctuationKind::Indentation(_))
                            | TokenKind::Punctuation(PunctuationKind::Semicolon)
                            | TokenKind::Comment(_)
                    )
                })
                    .with_ignore(),
                Classifier::predicate(|token: &Token| {
                    !matches!(
                        token.kind,
                        TokenKind::Punctuation(PunctuationKind::Newline)
                            | TokenKind::Punctuation(PunctuationKind::Tab)
                            | TokenKind::Punctuation(PunctuationKind::Space)
                            | TokenKind::Punctuation(PunctuationKind::Indentation(_))
                            | TokenKind::Punctuation(PunctuationKind::Semicolon)
                            | TokenKind::Comment(_)
                    )
                }),
            ]),
            0,
            Some(length),
        )
    }

    pub fn configuration() -> Classifier<
        'initializer,
        Token<'initializer>,
        Symbol<'initializer>,
        InitializeError<'initializer>,
    > {
        Classifier::alternative([
            Self::verbosity(),
            Self::input(),
            Self::output(),
            Self::implicit_input(),
            Classifier::anything().with_ignore(),
        ])
    }

    pub fn classifier() -> Classifier<
        'initializer,
        Token<'initializer>,
        Symbol<'initializer>,
        InitializeError<'initializer>,
    > {
        Classifier::repetition(Self::configuration(), 0, None)
    }

    pub fn initialize(&mut self) -> Vec<Location<'initializer>> {
        let location = Location::Flag;

        let mut scanner = Scanner::new(location);

        scanner.set_location(location);

        scanner.prepare();
        scanner.scan();

        self.input = scanner.output;

        let strained = {
            let length = self.length();
            let classifier = Self::filter(length);
            let mut former = Former::new(self);
            former.form(classifier).collect_inputs()
        };

        self.input = strained;
        self.reset();
        let mut former = Former::new(self);

        let mut configurations = Vec::new();

        let classifier = Self::classifier();

        let forms = former.form(classifier).flatten();

        for form in forms {
            match form {
                Form::Output(output) => configurations.push(output),
                Form::Failure(failure) => self.errors.push(failure),
                Form::Multiple(multiple) => {
                    for form in multiple {
                        configurations.push(form.unwrap_output().clone());
                    }
                }
                _ => {}
            }
        }

        let targets = configurations
            .iter()
            .filter(|configuration| {
                if let SymbolKind::Binding(Binding { target, .. }) = &configuration.kind {
                    if let ElementKind::Literal(token) = &target.kind {
                        if let TokenKind::Identifier(identifier) = &token.kind {
                            return *identifier == Str::from("Input")
                                || identifier.starts_with("Input(");
                        }
                    }
                }
                false
            })
            .filter_map(|configuration| {
                if let SymbolKind::Binding(Binding { value, .. }) = &configuration.kind {
                    if let Some(element) = value.as_ref() {
                        if let ElementKind::Literal(token) = &element.kind {
                            if let TokenKind::Identifier(value) = &token.kind {
                                return Some(Location::Entry(*value));
                            }
                        }
                    }
                }
                None
            })
            .collect::<Vec<_>>();

        let input_indexes = configurations
            .iter()
            .enumerate()
            .filter(|(_, configuration)| {
                if let SymbolKind::Binding(Binding { target, .. }) = &configuration.kind {
                    if let ElementKind::Literal(token) = &target.kind {
                        if let TokenKind::Identifier(identifier) = &token.kind {
                            return *identifier == Str::from("Input");
                        }
                    }
                }
                false
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>();

        if input_indexes.len() > 1 {
            for (ordinal, pref_index) in input_indexes.iter().enumerate() {
                if let Some(configuration) = configurations.get_mut(*pref_index) {
                    if let SymbolKind::Binding(Binding { target, .. }) = &mut configuration.kind {
                        if let ElementKind::Literal(token) = &target.kind {
                            let span = token.span;
                            *target = Box::new(Element::new(
                                ElementKind::Literal(Token::new(
                                    TokenKind::Identifier(Str::from(format!("Input({})", ordinal))),
                                    span,
                                )),
                                span,
                            ));
                        }
                    }
                }
            }
        }

        for key in [
            "Output",
            "OutputCode",
            "OutputBinary",
            "OutputIR",
            "OutputExec",
        ] {
            let indexes = configurations
                .iter()
                .enumerate()
                .filter(|(_, configuration)| {
                    if let SymbolKind::Binding(Binding { target, .. }) = &configuration.kind {
                        if let ElementKind::Literal(token) = &target.kind {
                            if let TokenKind::Identifier(identifier) = &token.kind {
                                return *identifier == Str::from(key);
                            }
                        }
                    }
                    false
                })
                .map(|(index, _)| index)
                .collect::<Vec<_>>();

            if indexes.len() > 1 {
                for (ordinal, pref_index) in indexes.iter().enumerate() {
                    if let Some(configuration) = configurations.get_mut(*pref_index) {
                        if let SymbolKind::Binding(Binding { target, .. }) = &mut configuration.kind {
                            if let ElementKind::Literal(token) = &target.kind {
                                let span = token.span;
                                *target = Box::new(Element::new(
                                    ElementKind::Literal(Token::new(
                                        TokenKind::Identifier(Str::from(format!(
                                            "{}({})",
                                            key, ordinal
                                        ))),
                                        span,
                                    )),
                                    span,
                                ));
                            }
                        }
                    }
                }
            }
        }

        self.output = configurations;

        targets
    }
}

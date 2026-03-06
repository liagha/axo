use {
    super::{InitializeError, Preference},
    crate::{
        data::{Offset, Scale, Str},
        formation::{classifier::Classifier, form::Form, former::Former},
        parser::{Element, ParseError},
        scanner::{PunctuationKind, Scanner, Token, TokenKind},
        tracker::{Location, Peekable, Position},
    },
};

pub struct Initializer<'initializer> {
    pub index: Offset,
    pub position: Position<'initializer>,
    pub input: Vec<Token<'initializer>>,
    pub output: Vec<Preference<'initializer>>,
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

    pub fn preference() -> Classifier<
        'initializer,
        Token<'initializer>,
        Preference<'initializer>,
        InitializeError<'initializer>,
    > {
        Classifier::alternative([
            Self::code(),
            Self::binary(),
            Self::verbosity(),
            Self::run(),
            Self::bootstrap(),
            Self::input(),
            Self::output(),
            Self::implicit_input(),
            Classifier::anything().with_ignore(),
        ])
    }

    pub fn classifier() -> Classifier<
        'initializer,
        Token<'initializer>,
        Preference<'initializer>,
        InitializeError<'initializer>,
    > {
        Classifier::repetition(Self::preference(), 0, None)
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

        let mut preferences = Vec::new();

        let classifier = Self::classifier();

        let forms = former.form(classifier).flatten();

        for form in forms {
            match form {
                Form::Output(output) => preferences.push(output),
                Form::Failure(failure) => self.errors.push(failure),
                Form::Multiple(multiple) => {
                    for form in multiple {
                        preferences.push(form.unwrap_output().clone());
                    }
                }
                _ => {}
            }
        }

        let targets = preferences
            .iter()
            .filter(|preference| {
                if let TokenKind::Identifier(identifier) = preference.target.kind {
                    identifier == Str::from("Input") || identifier.starts_with("Input(")
                } else {
                    false
                }
            })
            .map(|preference| {
                let path = preference.clone().value.kind.unwrap_identifier();

                Location::Entry(path)
            })
            .collect::<Vec<_>>();

        let input_indexes = preferences
            .iter()
            .enumerate()
            .filter(|(_, preference)| {
                if let TokenKind::Identifier(identifier) = preference.target.kind {
                    identifier == Str::from("Input")
                } else {
                    false
                }
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>();

        if input_indexes.len() > 1 {
            for (ordinal, pref_index) in input_indexes.iter().enumerate() {
                if let Some(preference) = preferences.get_mut(*pref_index) {
                    let span = preference.target.span;
                    preference.target = Token::new(
                        TokenKind::Identifier(Str::from(format!("Input({})", ordinal))),
                        span,
                    );
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
            let indexes = preferences
                .iter()
                .enumerate()
                .filter(|(_, preference)| {
                    if let TokenKind::Identifier(identifier) = preference.target.kind {
                        identifier == Str::from(key)
                    } else {
                        false
                    }
                })
                .map(|(index, _)| index)
                .collect::<Vec<_>>();

            if indexes.len() > 1 {
                for (ordinal, pref_index) in indexes.iter().enumerate() {
                    if let Some(preference) = preferences.get_mut(*pref_index) {
                        let span = preference.target.span;
                        preference.target = Token::new(
                            TokenKind::Identifier(Str::from(format!(
                                "{}({})",
                                key, ordinal
                            ))),
                            span,
                        );
                    }
                }
            }
        }

        self.output = preferences;

        targets
    }
}

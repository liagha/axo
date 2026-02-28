use {
    super::{InitialError, Preference},
    crate::{
        data::{Offset, Scale, Str},
        formation::{classifier::Classifier, form::Form, former::Former},
        internal::platform::{self, current_dir, PathBuf},
        parser::{Element, ParseError},
        scanner::{PunctuationKind, Scanner, Token, TokenKind},
        tracker::{Location, Peekable, Position, Span},
    },
};

#[derive(Debug)]
pub struct Initializer<'initializer> {
    pub index: Offset,
    pub position: Position<'initializer>,
    pub input: Vec<Token<'initializer>>,
    pub output: Vec<Preference<'initializer>>,
    pub errors: Vec<InitialError<'initializer>>,
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
        self.index.checked_sub(n).and_then(|current| self.get(current))
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
        InitialError<'initializer>,
    > {
        Classifier::alternative([
            Self::code(),
            Self::binary(),
            Self::quiet(),
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
        InitialError<'initializer>,
    > {
        Classifier::repetition(Self::preference(), 0, None)
    }

    fn visit() -> Result<Vec<PathBuf>, platform::Error> {
        use walkdir::WalkDir;

        let files: Vec<PathBuf> = WalkDir::new(current_dir()?.as_os_str())
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .map_or(false, |extension| extension == "axo")
            })
            .map(|entry| entry.path().to_path_buf())
            .collect();

        Ok(files)
    }

    pub fn initialize(&mut self) -> Vec<Location<'initializer>> {
        let location = Location::Flag;
        
        match location.get_value() { 
            Ok(input) => {
                let tokens = {
                    let mut scanner = Scanner::new(location);

                    let characters =
                        Scanner::inspect(Position::new(location), input.chars().collect::<Vec<_>>());
                    scanner.set_input(characters);
                    scanner.scan();
                    scanner.output
                };

                self.input = tokens;
                self.reset();

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

                let has_verbosity = preferences.iter().any(|preference| {
                    if let TokenKind::Identifier(identifier) = &preference.target.kind {
                        identifier == "Verbosity"
                    } else {
                        false
                    }
                });

                if !has_verbosity {
                    let span = Span::void();
                    preferences.push(Preference::new(
                        Token::new(TokenKind::Identifier(Str::from("Verbosity")), span),
                        Token::new(TokenKind::Boolean(true), span),
                    ));
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

                        Location::File(path)
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
                                    TokenKind::Identifier(Str::from(format!("{}({})", key, ordinal))),
                                    span,
                                );
                            }
                        }
                    }
                }

                self.output = preferences;

                targets
            }
            
            Err(error) => {
                let kind = super::ErrorKind::Tracking(error.clone());
                let error = super::InitialError::new(kind, error.span);
                
                self.errors.push(error);
                
                Vec::new()
            }
        }
    }
}

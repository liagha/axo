use {
    crate::{
        data::{
            any::{Any, TypeId},
            memory,
            string::Str,
            Offset, Scale
        },
        format::Debug,
        formation::{
            classifier::Classifier,
            form::Form,
            former::Former,
        },
        internal::{
            environment::current_dir,
            platform::{self, read_dir, Path, PathBuf},
            compiler::{Marked, Registry},
            hash::{DefaultHasher, Hash, Hasher},
        },
        parser::{Element, ParseError, Symbol, Symbolic},
        scanner::{OperatorKind, PunctuationKind, Scanner, Token, TokenKind},
        tracker::{Location, Peekable, Position, Span, Spanned},
    },
    super::{Preference, InitialError},
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
        let current = self.index - n;
        self.get(current)
    }

    fn next(&self, index: &mut Offset, position: &mut Position<'initializer>) -> Option<Token<'initializer>> {
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

                Ok(Form::Input(Token::new(TokenKind::Identifier(Str::from("Verbosity")), span)))
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

                    Ok(Form::Input(Token::new(TokenKind::Identifier(Str::from("Path")), span)))
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

                Ok(Form::output(Preference::new(identifier.clone(), Token::new(TokenKind::Identifier(Str::from(path)), span))))
            }
        )
    }

    pub fn strainer(length: Scale) -> Classifier<'initializer, Token<'initializer>, Element<'initializer>, ParseError<'initializer>> {
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

    fn visit() -> Result<Vec<PathBuf>, platform::Error> {
        use walkdir::WalkDir;

        let files: Vec<PathBuf> = WalkDir::new(".")
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| {
                entry.path().extension()
                    .map_or(false, |extension| extension == "axo")
            })
            .map(|entry| entry.path().to_path_buf())
            .collect();

        Ok(files)
    }

    pub fn initialize(&mut self) {
        let location = Location::Flag;
        let input = location.get_value();

        let tokens = {
            let mut scanner = Scanner::new(Location::Flag).with_input(input);
            scanner.scan();
            scanner.output
        };

        self.input = tokens;
        self.reset();

        let strained = {
            let length = self.length();
            let classifier = Self::strainer(length);
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
                _ => {}
            }
        }

        self.output = preferences;

        for file in Self::visit().unwrap() {
            println!("{:?}", file);
        }
    }
}

impl<'initializer> Marked<'initializer> for Initializer<'initializer> {
    fn registry(&self) -> &Registry<'initializer> {
        todo!()
    }

    fn registry_mut(&mut self) -> &mut Registry<'initializer> {
        todo!()
    }
}
use {
    crate::{
        data::{Str, Offset, Scale, },
        formation::{
            classifier::Classifier,
            form::Form,
            former::Former,
        },
        internal::{
            platform::{self, current_dir, PathBuf},
        },
        parser::{Element, ParseError},
        scanner::{PunctuationKind, Scanner, Token, TokenKind},
        tracker::{Location, Peekable, Position},
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

        let files: Vec<PathBuf> = WalkDir::new(current_dir()?.as_os_str())
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

    pub fn initialize(&mut self) -> Vec<Location<'initializer>> {
        let location = Location::Flag;
        let input = location.get_value();

        let tokens = {
            let mut scanner = Scanner::new(location);
            let characters = Scanner::inspect(Position::new(location), input.chars().collect::<Vec<_>>());
            scanner.set_input(characters);
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
                Form::Multiple(multiple) => {
                    for form in multiple {
                        preferences.push(form.unwrap_output().clone());
                    }
                }
                _ => {}
            }
        }

        let targets = {
            preferences
                .iter()
                .filter(|preference| {
                    if let TokenKind::Identifier(identifier) = preference.target.kind {
                        identifier == Str::from("Path")
                    } else {
                        false
                    }
                })
                .map(|preference| {
                    let path = preference.clone().value.kind.unwrap_identifier();

                    Location::File(path)
                })
                .collect::<Vec<_>>()
        };

        self.output = preferences;

        //let files = Self::visit().unwrap().iter().map(|path| format!("{}\n", path.as_path().display())).collect::<String>();
        //xprintln!("\nAxolotls:\n{}\n" => Color::Pink, files.indent() => Color::Magenta);

        targets
    }
}
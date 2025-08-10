use broccli::{xprintln, Color};
use {
    crate::{
        data::{
            any::{Any, TypeId},
            memory,
            string::Str,
            Offset, Scale
        },
        format::{Display, Show},
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

        let files = Self::visit().unwrap().iter().map(|path| format!("{}\n", path.as_path().display())).collect::<String>();
        xprintln!("\nAxolotls:\n{}\n" => Color::Pink, files.indent() => Color::Magenta);
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
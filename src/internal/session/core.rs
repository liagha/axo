use {
    crate::{
        analyzer::{Analysis, AnalyzeError},
        data::*,
        format::{Display, Show, Verbosity},
        initializer::{InitializeError, Initializer},
        internal::{
            hash::{Map, DefaultHasher, Hash, Hasher},
            platform::{read_dir, PathBuf},
            timer::{DefaultTimer, Duration},
        },
        parser::{Element, ElementKind, ParseError, Symbol, SymbolKind, Visibility},
        reporter::Error,
        resolver::{ResolveError, Resolver},
        scanner::{ScanError, Token, TokenKind},
        tracker::{self, Location, Span, TrackError},
    },
    broccli::{xprintln, Color},
};

#[cfg(feature = "generator")]
use {
    crate::generator::{GenerateError, Generator},
    inkwell::context::{Context, ContextRef},
};

pub const RUNTIME: &[&str] = &[
    "./runtime/cast.axo",
    "./runtime/cast.c",
    "./runtime/file.axo",
    "./runtime/file.c",
    "./runtime/memory.axo",
    "./runtime/memory.c",
    "./runtime/print.axo",
    "./runtime/print.c",
    "./runtime/process.axo",
    "./runtime/process.c",
    "./runtime/string.axo",
    "./runtime/string.c",
    "./runtime/input.axo",
    "./runtime/input.c",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InputKind {
    Source,
    Schema,
    Object,
    C,
}

impl InputKind {
    pub fn from_path(string: &str) -> Option<Self> {
        if string.ends_with(".axo") {
            Some(InputKind::Source)
        } else if string.ends_with(".ll") {
            Some(InputKind::Schema)
        } else if string.ends_with(".o") {
            Some(InputKind::Object)
        } else if string.ends_with(".c") {
            Some(InputKind::C)
        } else {
            None
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            InputKind::Source => "axo",
            InputKind::Schema => "ll",
            InputKind::Object => "o",
            InputKind::C => "c",
        }
    }
}

pub enum CompileError<'error> {
    Initialize(InitializeError<'error>),
    Scan(ScanError<'error>),
    Parse(ParseError<'error>),
    Resolve(ResolveError<'error>),
    Analyze(AnalyzeError<'error>),
    #[cfg(feature = "generator")]
    Generate(GenerateError<'error>),
    Track(TrackError<'error>),
}

pub struct Record<'session> {
    pub kind: InputKind,
    pub location: Location<'session>,
    pub module: Option<Identity>,
    pub tokens: Option<Vec<Token<'session>>>,
    pub elements: Option<Vec<Element<'session>>>,
    pub analyses: Option<Vec<Analysis<'session>>>,
    pub output: Option<Location<'session>>,
    pub object: Option<Location<'session>>,
    pub hash: u64,
    pub dirty: bool,
}

impl<'session> Record<'session> {
    pub fn new(kind: InputKind, location: Location<'session>) -> Self {
        Self {
            kind,
            location,
            module: None,
            tokens: None,
            elements: None,
            analyses: None,
            output: None,
            object: None,
            hash: 0,
            dirty: true,
        }
    }
}

pub struct Session<'session> {
    pub timer: DefaultTimer,
    pub order: Vec<Identity>,
    pub records: Map<Identity, Record<'session>>,
    pub initializer: Initializer<'session>,
    pub resolver: Resolver<'session>,
    #[cfg(feature = "generator")]
    pub generator: Generator<'session>,
    #[cfg(feature = "generator")]
    pub context: Context,
    pub errors: Vec<CompileError<'session>>,
    pub target: Option<Location<'session>>,
    pub cache: Map<Location<'session>, u64>,
}

impl<'session> Session<'session> {
    pub fn traverse(
        target: &Location<'session>,
        records: &mut Map<Identity, Record<'session>>,
    ) -> bool {
        let Ok(path) = target.to_path() else {
            return false;
        };

        if !path.is_dir() {
            return false;
        }

        let mut stack = vec![path];

        while let Some(current) = stack.pop() {
            if let Ok(entries) = read_dir(current) {
                for entry in entries.flatten() {
                    let child = entry.path();
                    if child.is_dir() {
                        stack.push(child);
                    } else {
                        let string = child.to_string_lossy().into_owned();

                        if let Some(kind) = InputKind::from_path(&string) {
                            let location = Location::Entry(Str::from(string.clone()));
                            let mut hasher = DefaultHasher::new();
                            Hash::hash(&string, &mut hasher);

                            let identity = (hasher.finish() as Identity) | 0x40000000;
                            records.insert(identity, Record::new(kind, location));
                        }
                    }
                }
            }
        }

        true
    }

    pub fn start() -> Self {
        let mut timer = DefaultTimer::new_default();
        _ = timer.start();

        let mut initializer = Initializer::new(Location::Flag);
        let mut resolver = Resolver::new();

        let mut records = Map::new();
        let mut errors = Vec::new();
        let cache = Map::new();

        for path in RUNTIME {
            if let Some(kind) = InputKind::from_path(path) {
                let string = path.to_string();
                let location = Location::Entry(Str::from(string.clone()));

                let mut hasher = DefaultHasher::new();
                Hash::hash(&string, &mut hasher);

                let identity = (hasher.finish() as Identity) & 0x3FFFFFFF;
                records.insert(identity, Record::new(kind, location));
            }
        }

        initializer.initialize().iter().for_each(|(target, span)| {
            if !Self::traverse(target, &mut records) {
                let string = target.to_string();

                if let Some(kind) = InputKind::from_path(&string) {
                    let mut hasher = DefaultHasher::new();
                    Hash::hash(&string, &mut hasher);

                    let identity = (hasher.finish() as Identity) | 0x40000000;
                    records.insert(identity, Record::new(kind, target.clone()));
                } else {
                    errors.push(CompileError::Track(TrackError::new(
                        tracker::error::ErrorKind::UnSupportedInput(target.clone()),
                        span.clone(),
                    )));
                }
            }
        });

        errors.extend(
            initializer
                .errors
                .iter()
                .map(|error| CompileError::Initialize(error.clone())),
        );

        for symbol in initializer.output.clone() {
            resolver.registry.insert(symbol.identity, symbol);
        }

        let directive = Symbol::new(
            SymbolKind::Module(Module::new(Box::from(Element::new(
                ElementKind::Literal(Token::new(
                    TokenKind::Identifier(Str::from("directive")),
                    Span::void(),
                )),
                Span::void(),
            )))),
            Span::void(),
            Visibility::Public,
        )
            .with_members(initializer.output.clone());

        resolver.insert(directive);

        _ = timer.lap();

        #[cfg(feature = "generator")]
        let (context, generator) = {
            let context = Context::create();
            let reference = unsafe { ContextRef::new(context.raw()) };
            let generator = Generator::new(reference);

            (context, generator)
        };

        Session {
            timer,
            order: Vec::new(),
            records,
            initializer,
            resolver,
            #[cfg(feature = "generator")]
            generator,
            #[cfg(feature = "generator")]
            context,
            errors,
            target: None,
            cache,
        }
    }

    pub fn get_directive(&self, key: Str<'session>) -> Option<Token<'session>> {
        let directive = self
            .resolver
            .registry
            .values()
            .find(|symbol| symbol.target() == Some(Str::from("directive")))?
            .clone();

        let identifier = Element::new(
            ElementKind::Literal(Token::new(TokenKind::Identifier(key), Span::void())),
            Span::void(),
        );

        let scope = directive.scope;
        let result = scope.lookup(&identifier, &self.resolver).ok()?;

        if let SymbolKind::Binding(binding) = result.kind {
            if let Some(value) = binding.value {
                if let ElementKind::Literal(literal) = value.kind {
                    return Some(literal);
                }
            }
        }

        None
    }

    pub fn get_verbosity(&self) -> Verbosity {
        match self.get_directive(Str::from("Verbosity")) {
            Some(Token {
                     kind: TokenKind::Integer(value),
                     ..
                 }) => Verbosity::from(value as u8),
            _ => Verbosity::Off,
        }
    }

    pub fn is_active(&self) -> bool {
        self.get_verbosity() != Verbosity::Off
    }

    pub fn report_start(&self, stage: &str) {
        if self.is_active() {
            xprintln!(
                "Started {}." => Color::Blue,
                format!("`{}`", stage) => Color::White
            );
            xprintln!();
        }
    }

    pub fn report_finish(&self, stage: &str, duration: Duration, count: usize) {
        if self.is_active() {
            let suffix = if count > 0 {
                format!(" ({} errors)", count)
            } else {
                String::new()
            };

            xprintln!(
                "Finished {} {}s{}" => Color::Green,
                format!("`{}` in", stage) => Color::White,
                duration.as_secs_f64(),
                suffix => Color::Red
            );
            xprintln!();
        }
    }

    pub fn report_external(&self, stage: &str, duration: Duration) {
        if self.is_active() {
            xprintln!(
                "Finished {} {}s" => Color::Yellow,
                format!("`{}` in", stage) => Color::White,
                duration.as_secs_f64()
            );
            xprintln!();
        }
    }

    pub fn report_section(&self, head: &str, color: Color, body: String) {
        if self.is_active() && !body.is_empty() {
            xprintln!(
                "{}{}\n{}" => Color::White,
                head => color,
                ":" => Color::White,
                Str::from(body).indent(self.get_verbosity().into()) => Color::White
            );
            xprintln!();
        }
    }

    pub fn report_error<K, H>(&self, error: &Error<K, H>)
    where
        K: Clone + Display,
        H: Clone + Display,
    {
        let (message, details) = error.handle();
        xprintln!(
            "{}\n{}" => Color::Red,
            message => Color::White,
            details => Color::White
        );
        xprintln!();
    }

    pub fn base(&self) -> PathBuf {
        let paths: Vec<_> = self
            .records
            .iter()
            .filter(|(&id, _)| (id & 0x40000000) != 0)
            .filter_map(|(_, record)| record.location.to_path().ok())
            .collect();

        if paths.is_empty() {
            return PathBuf::from(".");
        }

        let mut base = paths[0].parent().unwrap_or(&paths[0]).to_path_buf();

        for path in &paths[1..] {
            let parent = path.parent().unwrap_or(path);
            let mut current = PathBuf::new();

            let mut left = base.components();
            let mut right = parent.components();

            while let (Some(first), Some(second)) = (left.next(), right.next()) {
                if first == second {
                    current.push(first);
                } else {
                    break;
                }
            }

            base = current;
        }

        if base.as_os_str().is_empty() {
            PathBuf::from(".")
        } else {
            base
        }
    }

    #[cfg(feature = "generator")]
    pub fn schema(base: &PathBuf, location: Location<'session>) -> Location<'session> {
        let target = base
            .join("build")
            .join("schema")
            .join(location.stem().unwrap())
            .with_extension("ll");
        Location::Entry(Str::from(target))
    }

    pub fn object(
        base: &PathBuf,
        location: Location<'session>,
        kind: &InputKind,
        custom: Option<Str<'session>>,
    ) -> Location<'session> {
        let target = if let Some(path) = custom {
            PathBuf::from(path.to_string())
        } else {
            base.join("build")
                .join("objects")
                .join(kind.extension())
                .join(location.stem().unwrap())
                .with_extension("o")
        };

        Location::Entry(Str::from(target))
    }

    pub fn executable(
        base: &PathBuf,
        location: Location<'session>,
        custom: Option<Str<'session>>,
    ) -> Location<'session> {
        let target = if let Some(path) = custom {
            PathBuf::from(path.to_string())
        } else {
            base.join("build")
                .join(location.stem().unwrap())
                .with_extension("")
        };

        Location::Entry(Str::from(target))
    }
}

use {
    crate::{
        analyzer::Analysis,
        data::*,
        format::{Display, Show, Stencil},
        identifier,
        initializer::Initializer,
        internal::{
            hash::Map,
            platform::PathBuf,
            time::{Instant, Duration},
            SessionError,
        },
        literal,
        parser::{Element, ElementKind, SymbolKind},
        reporter::Error,
        resolver::Resolver,
        scanner::{Token, TokenKind},
        tracker::{Location, Span},
    },
    broccli::{xprintln, Color, TextStyle},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordKind {
    Source,
    Schema,
    Object,
    C,
    Flag,
}

impl RecordKind {
    pub fn from_path(path: &str) -> Option<Self> {
        if path.ends_with(".axo") {
            Some(RecordKind::Source)
        } else if path.ends_with(".ll") {
            Some(RecordKind::Schema)
        } else if path.ends_with(".o") {
            Some(RecordKind::Object)
        } else if path.ends_with(".c") {
            Some(RecordKind::C)
        } else {
            None
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            RecordKind::Source => "axo",
            RecordKind::Schema => "ll",
            RecordKind::Object => "o",
            RecordKind::C => "c",
            RecordKind::Flag => "",
        }
    }
}

pub enum Artifact<'session> {
    Module(Identity),
    Tokens(Vec<Token<'session>>),
    Elements(Vec<Element<'session>>),
    Analyses(Vec<Analysis<'session>>),
    Output(Location<'session>),
    Object(Location<'session>),
}

pub struct Record<'session> {
    pub kind: RecordKind,
    pub location: Location<'session>,
    pub content: Option<Str<'session>>,
    pub rows: Option<Vec<Offset>>,
    pub hash: u64,
    pub dirty: bool,
    pub version: usize,
    pub artifacts: Map<u8, Artifact<'session>>,
}

impl<'session> Record<'session> {
    pub fn new(kind: RecordKind, location: Location<'session>) -> Self {
        Self {
            kind,
            location,
            content: None,
            rows: None,
            hash: 0,
            dirty: true,
            version: 0,
            artifacts: Map::default(),
        }
    }

    pub fn set_content(&mut self, content: Str<'session>) {
        self.rows = Some(Self::rows(&content));
        self.content = Some(content);
        self.version += 1;
    }

    pub fn store(&mut self, key: u8, artifact: Artifact<'session>) {
        self.artifacts.insert(key, artifact);
        self.version += 1;
    }

    pub fn fetch(&self, key: u8) -> Option<&Artifact<'session>> {
        self.artifacts.get(&key)
    }

    pub fn fetch_mut(&mut self, key: u8) -> Option<&mut Artifact<'session>> {
        self.version += 1;
        self.artifacts.get_mut(&key)
    }

    pub fn rows(content: &str) -> Vec<Offset> {
        let mut rows = vec![0];

        for (index, byte) in content.bytes().enumerate() {
            if byte == b'\n' {
                rows.push(index as Offset + 1);
            }
        }

        rows
    }

    pub fn sync_rows(&mut self) {
        if self.rows.is_none() {
            if let Some(content) = &self.content {
                self.rows = Some(Self::rows(content));
            }
        }
    }

    pub fn span(&self, identity: Identity) -> Span {
        let end = self.content.as_ref().map(|value| value.len() as Offset).unwrap_or(0);
        Span::range(identity, 0, end)
    }
}

pub struct Session<'session> {
    pub timer: Instant,
    pub laps: Vec<Duration>,
    pub records: Map<Identity, Record<'session>>,
    pub initializer: Initializer<'session>,
    pub resolver: Resolver<'session>,
    pub errors: Vec<SessionError<'session>>,
    pub target: Option<Location<'session>>,
    pub cache: Map<Location<'session>, u64>,
    pub buffers: Vec<Vec<u8>>,
}

impl<'session> Session<'session> {
    pub fn get_directive(&self, key: Str<'session>) -> Option<Token<'session>> {
        let directive = self
            .resolver
            .registry
            .values()
            .find(|symbol| symbol.target() == Some(Str::from("directive")))?
            .clone();

        let identifier = literal!(identifier!(key));

        let scope = directive.scope;
        let result = scope.lookup(&identifier, &self.resolver).ok()?;

        if let SymbolKind::Binding(binding) = result.kind {
            if let Some(value) = binding.value {
                if let ElementKind::Literal(literal) = value.kind {
                    return Some(*literal);
                }
            }
        }

        None
    }

    pub fn get_stencil(&self) -> Option<Stencil> {
        match self.get_directive(Str::from("Verbosity")) {
            Some(Token {
                     kind: TokenKind::Integer(_),
                     ..
                 }) => Some(Stencil::default()),
            _ => None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.get_stencil().is_some()
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

    pub fn report_execute(&self, executable: &str) {
        if self.is_active() {
            xprintln!(
                "Executing {}." => Color::Blue,
                format!("`{}`", executable) => Color::White
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
        if let Some(stencil) = self.get_stencil() {
            xprintln!(
                "{}{}\n{}" => Color::White,
                head => color,
                ":" => Color::White,
                Str::from(body).indent(stencil) => Color::White
            );
            xprintln!();
        }
    }

    pub fn report_error<K, H>(&self, error: &Error<K, H>)
    where
        K: Clone + Display,
        H: Clone + Display,
    {
        let (message, details) = error.handle_record(self.records.get(&error.span.identity));
        xprintln!(
            "{}{}\n{}",
            "error: ".colorize(Color::Crimson).bold(),
            message,
            details
        );
        xprintln!();
    }

    pub fn report_all(&self) {
        for error in &self.errors {
            match error {
                SessionError::Initialize(error) => self.report_error(error),
                SessionError::Scan(error) => self.report_error(error),
                SessionError::Parse(error) => self.report_error(error),
                SessionError::Resolve(error) => self.report_error(error),
                SessionError::Analyze(error) => self.report_error(error),
                #[cfg(feature = "interpreter")]
                SessionError::Interpret(error) => self.report_error(error),
                SessionError::Track(error) => self.report_error(error),
                #[cfg(feature = "generator")]
                SessionError::Generate(error) => self.report_error(error),
            }
        }
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

    pub fn manifest(&self) -> PathBuf {
        self.base().join("build").join("records").join("manifest")
    }

    pub fn schema(base: &PathBuf, location: Location<'session>) -> Location<'session> {
        let target = base
            .join("build")
            .join("schema")
            .join(location.stem().unwrap())
            .with_extension("ll");
        Location::from(target)
    }

    pub fn object(
        base: &PathBuf,
        location: Location<'session>,
        kind: &RecordKind,
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

        Location::from(target)
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

        Location::from(target)
    }
}

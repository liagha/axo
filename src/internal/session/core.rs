use crate::format::Show;
use {
    crate::{
        analyzer::Analysis,
        data::*,
        format::{Display, Stencil},
        identifier,
        internal::{
            hash::{DefaultHasher, Hash, Hasher, Map},
            platform::{args, PathBuf, ARCH, OS},
            time::{Duration, Instant},
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

pub const BASE: &[(&str, &str)] = &[
    ("./base/cast.axo", include_str!("../../../base/cast.axo")),
    ("./base/cast.c", include_str!("../../../base/cast.c")),
    ("./base/file.axo", include_str!("../../../base/file.axo")),
    ("./base/file.c", include_str!("../../../base/file.c")),
    (
        "./base/memory.axo",
        include_str!("../../../base/memory.axo"),
    ),
    ("./base/memory.c", include_str!("../../../base/memory.c")),
    ("./base/print.axo", include_str!("../../../base/print.axo")),
    ("./base/print.c", include_str!("../../../base/print.c")),
    (
        "./base/process.axo",
        include_str!("../../../base/process.axo"),
    ),
    ("./base/process.c", include_str!("../../../base/process.c")),
    (
        "./base/string.axo",
        include_str!("../../../base/string.axo"),
    ),
    ("./base/string.c", include_str!("../../../base/string.c")),
    ("./base/input.axo", include_str!("../../../base/input.axo")),
    ("./base/input.c", include_str!("../../../base/input.c")),
    (
        "./base/vector.axo",
        include_str!("../../../base/vector.axo"),
    ),
    ("./base/vector.c", include_str!("../../../base/vector.c")),
];

pub const EXECUTABLE_ID: Identity = 0x7FFFFFFF;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordKind {
    Source,
    Schema,
    Object,
    C,
    Flag,
    Executable,
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
            RecordKind::Executable => "",
        }
    }
}

pub enum Artifact<'session> {
    Module(Identity),
    Tokens(Vec<Token<'session>>),
    Elements(Vec<Element<'session>>),
    Analyses(Vec<Analysis<'session>>),
    Schema(Location<'session>),
    Object(Location<'session>),
    Content(Str<'session>),
}

pub struct Record<'session> {
    pub kind: RecordKind,
    pub location: Location<'session>,
    pub artifacts: Map<u8, Artifact<'session>>,
}

impl<'session> Record<'session> {
    pub fn new(kind: RecordKind, location: Location<'session>) -> Self {
        Self {
            kind,
            location,
            artifacts: Map::default(),
        }
    }

    pub fn content(&self) -> Option<&Str<'session>> {
        self.artifacts.get(&6).and_then(|artifact| match artifact {
            Artifact::Content(content) => Some(content),
            _ => None,
        })
    }

    pub fn set_content(&mut self, content: Str<'session>) {
        self.artifacts.insert(6, Artifact::Content(content));
    }

    pub fn fetch(&self, key: u8) -> Option<&Artifact<'session>> {
        self.artifacts.get(&key)
    }

    pub fn fetch_mut(&mut self, key: u8) -> Option<&mut Artifact<'session>> {
        self.artifacts.get_mut(&key)
    }

    pub fn offset_to_line_column(&self, offset: Offset) -> Option<(usize, usize)> {
        let text = self.content()?;
        let mut line = 0;
        let mut col = 0;
        for (i, byte) in text.bytes().enumerate() {
            if i == offset as usize {
                return Some((line, col));
            }
            if *byte == b'\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
        if offset as usize == text.len() {
            Some((line, col))
        } else {
            None
        }
    }

    pub fn span(&self, identity: Identity) -> Span {
        let end = self
            .content()
            .map(|value| value.len() as Offset)
            .unwrap_or(0);
        Span::range(identity, 0, end)
    }
}

pub struct Session<'session> {
    pub timer: Instant,
    pub laps: Vec<Duration>,
    pub records: Map<Identity, Record<'session>>,
    pub resolver: Resolver<'session>,
    pub errors: Vec<SessionError<'session>>,
}

impl<'session> Session<'session> {
    pub fn new() -> Self {
        let timer = Instant::now();
        let mut records = Map::new();

        for &(path, content) in BASE {
            if let Some(kind) = RecordKind::from_path(path) {
                let string = path.to_string();
                let location = Location::from(string.clone());
                let mut hasher = DefaultHasher::new();
                Hash::hash(&string, &mut hasher);
                let identity = (hasher.finish() as Identity) & 0x3FFFFFFF;
                let mut base = Record::new(kind, location);
                base.set_content(Str::from(content));
                records.insert(identity, base);
            }
        }

        Self {
            timer,
            laps: vec![timer.elapsed()],
            records,
            resolver: Resolver::new(),
            errors: Vec::new(),
        }
    }

    pub fn has_input(&self) -> bool {
        self.records
            .iter()
            .any(|(&id, record)| record.kind == RecordKind::Source && (id & 0x40000000) != 0)
    }

    pub fn arguments() -> Str<'static> {
        args()
            .skip(1)
            .map(|arg| {
                if arg.contains(' ') || arg.contains('\t') {
                    format!("\"{}\"", arg.replace('\\', "\\\\").replace('"', "\\\""))
                } else {
                    arg
                }
            })
            .collect::<Vec<String>>()
            .join(" ")
            .into()
    }

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

    pub fn get_host() -> &'static str {
        match (ARCH, OS) {
            ("x86_64", "windows") => "x86_64-pc-windows-msvc",
            ("aarch64", "windows") => "aarch64-pc-windows-msvc",
            ("x86_64", "macos") => "x86_64-apple-darwin",
            ("aarch64", "macos") => "aarch64-apple-darwin",
            ("x86_64", "linux") => "x86_64-unknown-linux-gnu",
            ("aarch64", "linux") => "aarch64-unknown-linux-gnu",
            _ => "unknown",
        }
    }

    pub fn get_target(&self) -> Option<Str<'session>> {
        match self.get_directive(Str::from("Target")) {
            Some(Token {
                kind: TokenKind::Identifier(value),
                ..
            }) => Some(*value),
            Some(Token {
                kind: TokenKind::String(value),
                ..
            }) => Some(*value),
            _ => None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.get_stencil().is_some()
    }

    pub fn get_executable(&self) -> Option<Location<'session>> {
        self.records
            .get(&EXECUTABLE_ID)
            .map(|record| record.location)
    }

    pub fn set_executable(&mut self, location: Location<'session>) {
        let record = Record::new(RecordKind::Executable, location);
        self.records.insert(EXECUTABLE_ID, record);
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

    pub fn report_error<K>(&self, error: &Error<K>)
    where
        K: Clone + Display,
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
                SessionError::Track(error) => self.report_error(error),
                #[cfg(any(feature = "llvm", feature = "interpreter"))]
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

    pub fn executable_path(
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

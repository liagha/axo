use {
    crate::{
        analyzer::Analysis,
        data::*,
        format::{Display, Show, Stencil},
        initializer::Initializer,
        internal::{
            hash::Map,
            platform::PathBuf,
            time::{DefaultTimer, Duration},
            CompileError,
        },
        parser::{Element, ElementKind, SymbolKind},
        reporter::Error,
        resolver::Resolver,
        scanner::{Token, TokenKind},
        tracker::{Location, Span},
    },
    broccli::{xprintln, Color},
};

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

pub struct Record<'session> {
    pub kind: InputKind,
    pub location: Location<'session>,
    pub module: Option<Identity>,
    pub content: Option<String>,
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
            content: None,
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
    pub records: Map<Identity, Record<'session>>,
    pub initializer: Initializer<'session>,
    pub resolver: Resolver<'session>,
    pub errors: Vec<CompileError<'session>>,
    pub target: Option<Location<'session>>,
    pub cache: Map<Location<'session>, u64>,
}

impl<'session> Session<'session> {
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

    pub fn get_stencil(&self) -> Option<Stencil> {
        match self.get_directive(Str::from("Verbosity")) {
            Some(Token {
                     kind: TokenKind::Integer(_),
                     ..
                 }) => Some(Stencil::default()),
            _ => Some(Stencil::default()),
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
        xprintln!("{}", error);
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

    pub fn manifest(&self) -> PathBuf {
        let base = self.base();
        base.join("build").join("records").join("manifest")
    }

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

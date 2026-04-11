#[cfg(feature = "interpreter")]
mod dialog;

use axo::{
    data::{Identity, Module, Str},
    initializer::Initializer,
    internal::{
        hash::{DefaultHasher, Hash, Hasher, Map},
        platform::read_dir,
        time::DefaultTimer,
        SessionError, RecordKind, Record, Session,
    },
    parser::{Element, ElementKind, Symbol, SymbolKind, Visibility},
    resolver::{Resolver},
    scanner::{Token, TokenKind},
    tracker::{self, Location, Span, TrackError},
};

#[cfg(feature = "interpreter")]
use axo::interpreter::{Interpreter, interpret};

pub const BASE: &[(&str, &str)] = &[
    ("./base/cast.axo", include_str!("../base/cast.axo")),
    ("./base/cast.c", include_str!("../base/cast.c")),
    ("./base/file.axo", include_str!("../base/file.axo")),
    ("./base/file.c", include_str!("../base/file.c")),
    ("./base/memory.axo", include_str!("../base/memory.axo")),
    ("./base/memory.c", include_str!("../base/memory.c")),
    ("./base/print.axo", include_str!("../base/print.axo")),
    ("./base/print.c", include_str!("../base/print.c")),
    ("./base/process.axo", include_str!("../base/process.axo")),
    ("./base/process.c", include_str!("../base/process.c")),
    ("./base/string.axo", include_str!("../base/string.axo")),
    ("./base/string.c", include_str!("../base/string.c")),
    ("./base/input.axo", include_str!("../base/input.axo")),
    ("./base/input.c", include_str!("../base/input.c")),
];

fn main() {
    println!("SymbolKind: {}", size_of::<SymbolKind>());
    println!("ElementKind: {}", size_of::<ElementKind>());
    println!("Type: {}", size_of::<axo::resolver::Type>());
    println!("Token: {}", size_of::<Token>());
    println!("Scope: {}", size_of::<axo::resolver::scope::Scope>());
    println!("Span: {}", size_of::<Span>());
    let mut initializer = Initializer::new(Location::Flag);
    let targets = initializer.initialize();

    let bare = initializer.output.iter().any(|symbol| {
        if let SymbolKind::Binding(binding) = &symbol.kind {
            if let ElementKind::Literal(token) = &binding.target.kind {
                if let TokenKind::Identifier(name) = &token.kind {
                    return **name == "Bare";
                }
            }
        }
        false
    });

    let failures: Vec<SessionError> = initializer
        .errors
        .into_iter()
        .map(SessionError::Initialize)
        .collect();

    if targets.is_empty() {
        #[cfg(feature = "interpreter")]
        dialog::start(bare, initializer.output);
    } else {
        build(targets, bare, initializer.output, failures);
    }
}

#[cfg(feature = "interpreter")]
pub fn run<'a>(
    session: &mut Session<'a>,
    core: &mut Interpreter<'a>,
    keys: &[Identity],
) {
    use axo::{
        internal::prepare,
        scanner::scan,
        parser::parse,
        resolver::resolve,
        analyzer::analyze,
    };

    session.errors.clear();

    if !prepare(session) {
        session.report_all();
        return;
    }

    scan(session, keys);
    parse(session, keys);
    resolve(session, keys);
    analyze(session, keys);
    interpret(session, core, keys);

    session.report_all();
}

fn build(
    targets: Vec<(Location<'static>, Span<'static>)>,
    bare: bool,
    directives: Vec<Symbol>,
    failures: Vec<SessionError<'static>>,
) {
    let mut session = create(bare, directives, failures);

    targets.iter().for_each(|(target, span)| {
        if !traverse(target, &mut session.records) {
            let string = target.to_string();

            if let Some(kind) = RecordKind::from_path(&string) {
                let mut hasher = DefaultHasher::new();
                Hash::hash(&string, &mut hasher);

                let identity = (hasher.finish() as Identity) | 0x40000000;
                session.records.insert(identity, Record::new(kind, target.clone()));
            } else {
                session.errors.push(SessionError::Track(TrackError::new(
                    tracker::error::ErrorKind::UnSupportedInput(target.clone()),
                    span.clone(),
                )));
            }
        }
    });

    let _session = session.compile();
}

pub fn create<'a>(
    bare: bool,
    directives: Vec<Symbol<'a>>,
    failures: Vec<SessionError<'a>>,
) -> Session<'a> {
    let mut timer = DefaultTimer::new_default();
    _ = timer.start();

    let mut resolver = Resolver::new();
    let mut records = Map::new();
    let cache = Map::new();

    if !bare {
        for &(path, content) in BASE {
            if let Some(kind) = RecordKind::from_path(path) {
                let string = path.to_string();
                let location = Location::Entry(Str::from(string.clone()));

                let mut hasher = DefaultHasher::new();
                Hash::hash(&string, &mut hasher);

                let identity = (hasher.finish() as Identity) & 0x3FFFFFFF;
                let mut record = Record::new(kind, location);

                record.content = Some(content.to_string());
                records.insert(identity, record);
            }
        }
    }

    for symbol in directives.clone() {
        resolver.registry.insert(symbol.identity, symbol);
    }

    let directive = Symbol::new(
        SymbolKind::module(Module::new(Box::from(Element::new(
            ElementKind::literal(Token::new(
                TokenKind::identifier(Str::from("directive")),
                Span::void(),
            )),
            Span::void(),
        )))),
        Span::void(),
        Visibility::Public,
    )
        .with_members(directives);

    resolver.insert(directive);

    _ = timer.lap();

    Session {
        timer,
        records,
        initializer: Initializer::new(Location::Flag),
        resolver,
        errors: failures,
        target: None,
        cache,
    }
}

pub fn traverse<'a>(target: &Location<'a>, records: &mut Map<Identity, Record<'a>>) -> bool {
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

                    if let Some(kind) = RecordKind::from_path(&string) {
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

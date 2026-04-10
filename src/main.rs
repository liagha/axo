mod dialog;

use axo::{
    analyzer,
    data::{Identity, Module, Str},
    initializer::Initializer,
    internal::{
        hash::{DefaultHasher, Hash, Hasher, Map},
        platform::read_dir,
        prepare,
        time::DefaultTimer,
        CompileError, InputKind, Record, Session,
    },
    interpreter,
    parser,
    parser::{Element, ElementKind, Symbol, SymbolKind, Visibility},
    resolver::Resolver,
    scanner,
    scanner::{Token, TokenKind},
    tracker::{self, Location, Span, TrackError},
};

pub const BASE: &[&str] = &[
    "./base/cast.axo",
    "./base/cast.c",
    "./base/file.axo",
    "./base/file.c",
    "./base/memory.axo",
    "./base/memory.c",
    "./base/print.axo",
    "./base/print.c",
    "./base/process.axo",
    "./base/process.c",
    "./base/string.axo",
    "./base/string.c",
    "./base/input.axo",
    "./base/input.c",
];

fn main() {
    let mut initializer = Initializer::new(Location::Flag);
    let targets = initializer.initialize();

    let bare = initializer.output.iter().any(|symbol| {
        if let SymbolKind::Binding(binding) = &symbol.kind {
            if let ElementKind::Literal(token) = &binding.target.kind {
                if let TokenKind::Identifier(name) = &token.kind {
                    return name == "Bare";
                }
            }
        }
        false
    });

    let failures: Vec<CompileError> = initializer
        .errors
        .into_iter()
        .map(CompileError::Initialize)
        .collect();

    if targets.is_empty() {
        dialog::start(bare, initializer.output);
    } else {
        build(targets, bare, initializer.output, failures);
    }
}

pub(crate) fn run<'a>(
    session: &mut Session<'a>,
    core: &mut interpreter::Interpreter<'a>,
    keys: &[Identity],
) {
    session.errors.clear();

    if !prepare(session) {
        show(session);
        return;
    }

    scanner::scan(session, keys);
    parser::parse(session, keys);
    axo::resolver::resolve(session, keys);
    analyzer::analyze(session, keys);
    interpreter::interpret(session, core, keys);

    show(session);
}

fn show(session: &Session) {
    for error in &session.errors {
        match error {
            CompileError::Initialize(error) => session.report_error(error),
            CompileError::Scan(error) => session.report_error(error),
            CompileError::Parse(error) => session.report_error(error),
            CompileError::Resolve(error) => session.report_error(error),
            CompileError::Analyze(error) => session.report_error(error),
            CompileError::Interpret(error) => session.report_error(error),
            CompileError::Track(error) => session.report_error(error),
            #[cfg(feature = "generator")]
            CompileError::Generate(error) => session.report_error(error),
        }
    }
}

fn build(
    targets: Vec<(Location<'static>, Span<'static>)>,
    bare: bool,
    directives: Vec<Symbol>,
    failures: Vec<CompileError<'static>>,
) {
    let mut session = create(bare, directives, failures);

    targets.iter().for_each(|(target, span)| {
        if !traverse(target, &mut session.records) {
            let string = target.to_string();

            if let Some(kind) = InputKind::from_path(&string) {
                let mut hasher = DefaultHasher::new();
                Hash::hash(&string, &mut hasher);

                let identity = (hasher.finish() as Identity) | 0x40000000;
                session.records.insert(identity, Record::new(kind, target.clone()));
            } else {
                session.errors.push(CompileError::Track(TrackError::new(
                    tracker::error::ErrorKind::UnSupportedInput(target.clone()),
                    span.clone(),
                )));
            }
        }
    });

    let _session = session.compile();
}

pub(crate) fn create<'a>(
    bare: bool,
    directives: Vec<Symbol<'a>>,
    failures: Vec<CompileError<'a>>,
) -> Session<'a> {
    let mut timer = DefaultTimer::new_default();
    _ = timer.start();

    let mut resolver = Resolver::new();
    let mut records = Map::new();
    let cache = Map::new();

    if !bare {
        for path in BASE {
            if let Some(kind) = InputKind::from_path(path) {
                let string = path.to_string();
                let location = Location::Entry(Str::from(string.clone()));

                let mut hasher = DefaultHasher::new();
                Hash::hash(&string, &mut hasher);

                let identity = (hasher.finish() as Identity) & 0x3FFFFFFF;
                records.insert(identity, Record::new(kind, location));
            }
        }
    }

    for symbol in directives.clone() {
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

use axo::{
    data::{Identity, Module, Str},
    initializer::Initializer,
    internal::{
        hash::{DefaultHasher, Hash, Hasher, Map},
        platform::read_dir,
        time::DefaultTimer,
        CompileError, InputKind, Record, Session,
    },
    parser::{Element, ElementKind, Symbol, SymbolKind, Visibility},
    resolver::Resolver,
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
    let mut timer = DefaultTimer::new_default();
    _ = timer.start();

    let mut initializer = Initializer::new(Location::Flag);
    let mut resolver = Resolver::new();

    let mut records = Map::new();
    let mut errors = Vec::new();
    let cache = Map::new();

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

    targets.iter().for_each(|(target, span)| {
        if !traverse(target, &mut records) {
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

    let compiler = Session { timer, records, initializer, resolver, errors, target: None, cache };

    compiler.compile();
}

pub fn traverse<'a>(
    target: &Location<'a>,
    records: &mut Map<Identity, Record<'a>>,
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
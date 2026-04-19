use axo::{
    dialog,
    internal::{Session, SessionError},
    parser::{ElementKind, Symbol, SymbolKind},
    scanner::TokenKind,
    tracker::{Location, Span, TrackError, ErrorKind},
    data::Str,
};

fn main() {
    let flag = Session::arguments();
    let mut initializer = axo::initializer::Initializer::new(flag.clone());
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
        dialog::start(bare, initializer.output, flag);
    } else {
        build(targets, bare, initializer.output, failures, flag);
    }
}

fn build(
    targets: Vec<(Location<'static>, Span)>,
    bare: bool,
    directives: Vec<Symbol>,
    failures: Vec<SessionError<'static>>,
    flag: Str<'static>,
) {
    let mut session = Session::create(bare, directives, failures, flag);

    targets.iter().for_each(|(target, span)| {
        if !Session::traverse(target, &mut session.records) {
            let string = target.to_string();

            if let Some(kind) = axo::internal::RecordKind::from_path(&string) {
                let mut hasher = axo::internal::hash::DefaultHasher::new();
                axo::internal::hash::Hash::hash(&string, &mut hasher);

                let identity = (axo::internal::hash::Hasher::finish(&hasher) as axo::data::Identity) | 0x40000000;
                session.records.insert(identity, axo::internal::Record::new(kind, target.clone()));
            } else {
                session.errors.push(SessionError::Track(TrackError::new(
                    ErrorKind::UnSupportedInput(target.clone()),
                    span.clone(),
                )));
            }
        }
    });

    let _session = session.compile();
}

use axo::{
    internal::{Session, SessionError},
    tracker::{ErrorKind, TrackError},
};

fn main() {
    let flag = Session::arguments();
    let mut initializer = axo::initializer::Initializer::new(flag.clone());
    let targets = initializer.initialize();

    let failures: Vec<SessionError> = initializer
        .errors
        .into_iter()
        .map(SessionError::Initialize)
        .collect();

    if targets.is_empty() {
        #[cfg(feature = "dialog")]
        axo::dialog::Dialog::start(initializer.output, flag);
    } else {
        let mut session = Session::create(initializer.output, failures, flag);

        targets.iter().for_each(|(target, span)| {
            let string = target.to_string();

            if let Some(kind) = axo::internal::RecordKind::from_path(&string) {
                let mut hasher = axo::internal::hash::DefaultHasher::new();
                axo::internal::hash::Hash::hash(&string, &mut hasher);

                let identity = (axo::internal::hash::Hasher::finish(&hasher)
                    as axo::data::Identity)
                    | 0x40000000;
                session
                    .records
                    .insert(identity, axo::internal::Record::new(kind, target.clone()));
            } else {
                session.errors.push(SessionError::Track(TrackError::new(
                    ErrorKind::UnSupportedInput(target.clone()),
                    span.clone(),
                )));
            }
        });

        let _session = session.compile();
    }
}

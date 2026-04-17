mod backend;
mod generator;
mod inkwell;

use {
    crate::{
        internal::{
            platform::{
                create_dir_all,
                Lock,
                Command,
            },
            time::Duration,
            SessionError, RecordKind, Session,
            Artifact,
        },
        data::{
            Str,
            memory::Arc,
        },
        combinator::{Action, Operation, Operator},
        reporter::Error,
        tracker::{Span, error::ErrorKind as TrackErrorKind, TrackError},
    },
};

pub use {backend::Backend, inkwell::*};

pub type GenerateError<'error> = Error<'error, ErrorKind<'error>>;

pub struct GenerateAction;

impl<'source>
Action<
    'static,
    Operator<Arc<Lock<Session<'source>>>>,
    Operation<'source, Arc<Lock<Session<'source>>>>,
> for GenerateAction
{
    fn action(
        &self,
        operator: &mut Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) -> () {
        let mut guard = operator.store.write().unwrap();
        let session = &mut *guard;

        let context = Context::create();
        let reference = unsafe { ContextRef::new(context.raw()) };
        let mut generator = Generator::new(reference);

        let triple = TargetMachine::get_default_triple();
        let base = session.base();

        let initial = session.errors.len();

        session.report_start("generating");

        let mut keys: Vec<_> = session
            .records
            .iter()
            .filter_map(|(&key, record)| {
                if record.kind == RecordKind::Source && record.fetch(0).is_some() {
                    Some(key)
                } else {
                    None
                }
            })
            .collect();
        keys.sort();

        let discard = session.get_directive(Str::from("Discard")).is_some();

        for &key in &keys {
            let record = session.records.get_mut(&key).unwrap();
            let location = record.location;
            let schema = Session::schema(&base, location);

            if !record.dirty && schema.to_path().map(|p| p.exists()).unwrap_or(false) {
                record.store(4, Artifact::Output(schema));
                continue;
            }

            let stem = Str::from(location.stem().unwrap().to_string());

            if let Some(Artifact::Analyses(analysis_ref)) = record.fetch(3) {
                let analysis = analysis_ref.clone();
                let module = generator.context.create_module(stem.as_str().unwrap());

                module.set_triple(&triple);

                generator.modules.insert(stem, module);
                generator.current_module = stem;

                generator.generate(analysis);

                if discard {
                    continue;
                }

                match schema.as_path() {
                    Ok(path) => {
                        let parent = path.parent().unwrap();
                        _ = create_dir_all(parent);

                        match crate::internal::platform::File::create(&path) {
                            Ok(mut file) => {
                                use crate::internal::platform::Write;
                                let string = generator
                                    .current_module()
                                    .print_to_string()
                                    .to_string();
                                if let Err(error) = file.write_all(string.as_bytes()) {
                                    let kind = TrackErrorKind::from_io(error, schema);
                                    let track = TrackError::new(kind, Span::void());
                                    session.errors.push(SessionError::Track(track));
                                    operation.set_reject();
                                    return ();
                                }
                                record.store(4, Artifact::Output(schema));
                            }
                            Err(error) => {
                                let kind = TrackErrorKind::from_io(error, schema);
                                let track = TrackError::new(kind, Span::void());
                                session.errors.push(SessionError::Track(track));
                            }
                        }
                    }
                    Err(error) => session.errors.push(SessionError::Track(error)),
                }
            }
        }

        let duration = Duration::from_nanos(session.timer.lap().unwrap());
        session.report_finish("generating", duration, session.errors.len() - initial);

        session.errors.extend(generator
                                  .errors
                                  .iter()
                                  .map(|error| SessionError::Generate(error.clone())),
        );

        if session.errors.is_empty() {
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }
        ()
    }
}

pub struct EmitAction;

impl<'source>
Action<
    'static,
    Operator<Arc<Lock<Session<'source>>>>,
    Operation<'source, Arc<Lock<Session<'source>>>>,
> for EmitAction
{
    fn action(
        &self,
        operator: &mut Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) -> () {
        let mut session = operator.store.write().unwrap();
        if session.get_directive(Str::from("Discard")).is_some() {
            if session.errors.is_empty() {
                operation.set_resolve(Vec::new());
            } else {
                operation.set_reject();
            }
            return ();
        }

        session.report_start("emitting");

        let base = session.base();
        let mut direct = Vec::new();

        let mut keys: Vec<_> = session.records.keys().copied().collect();
        keys.sort();

        for &key in &keys {
            let record = session.records.get_mut(&key).unwrap();

            let target = match record.kind {
                RecordKind::Source => {
                    if let Some(Artifact::Output(loc)) = record.fetch(4) {
                        Some(loc.to_string())
                    } else {
                        None
                    }
                },
                RecordKind::Schema => Some(record.location.to_string()),
                RecordKind::C => {
                    if let Some(content) = &record.content {
                        let path = record.location.to_path().unwrap();
                        let name = path.file_name().unwrap();
                        let build = base.join("build").join("base");

                        _ = create_dir_all(&build);
                        let build_path = build.join(name);

                        if !build_path.exists() {
                            if let Ok(mut file) = crate::internal::platform::File::create(&build_path) {
                                use crate::internal::platform::Write;
                                _ = file.write_all(content.as_bytes());
                            }
                        }
                        Some(build_path.to_string_lossy().into_owned())
                    } else {
                        Some(record.location.to_string())
                    }
                }
                RecordKind::Object => {
                    direct.push(record.location);
                    None
                }
                RecordKind::Flag => {
                    None
                }
            };

            if let Some(path) = target {
                let object = Session::object(&base, record.location, &record.kind, None);
                let parent = object.to_path().unwrap().parent().unwrap().to_path_buf();
                _ = create_dir_all(&parent);

                record.store(5, Artifact::Object(object));

                if !record.dirty && object.to_path().map(|p| p.exists()).unwrap_or(false) {
                    continue;
                }

                let mut command = Command::new("clang");

                command
                    .arg("-c")
                    .arg(path.clone())
                    .arg("-o")
                    .arg(object.to_string());

                let status = command.status().expect("failed");

                if !status.success() {
                    panic!("failed compiling: {}", path);
                }
            }
        }

        let mut link = Command::new("cc");

        for &key in &keys {
            if let Some(Artifact::Object(object)) = session.records.get(&key).unwrap().fetch(5) {
                link.arg(object.to_string());
            }
        }

        for object in direct {
            link.arg(object.to_string());
        }

        let key = *keys.last().expect("missing");

        let record = session.records.get(&key).unwrap();

        let location = if let Some(Artifact::Output(loc)) = record.fetch(4) {
            *loc
        } else {
            record.location
        };

        let executable = Session::executable(&base, location, None);
        link.arg("-o").arg(executable.to_string());

        let status = link.status().expect("failed");

        if !status.success() {
            panic!("emitter failed: {}", status);
        }

        session.target = Some(executable);

        let duration = Duration::from_nanos(session.timer.lap().unwrap());
        session.report_external("emitting", duration);

        if session.errors.is_empty() {
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }
        ()
    }
}

pub struct RunAction;

impl<'source>
Action<
    'static,
    Operator<Arc<Lock<Session<'source>>>>,
    Operation<'source, Arc<Lock<Session<'source>>>>,
> for RunAction
{
    fn action(
        &self,
        operator: &mut Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) -> () {
        let mut session = operator.store.write().unwrap();
        if session.get_directive(Str::from("Discard")).is_some() {
            if session.errors.is_empty() {
                operation.set_resolve(Vec::new());
            } else {
                operation.set_reject();
            }
            return ();
        }

        session.report_start("running");

        let executable = session.target.unwrap();

        session.report_execute(&executable.to_string());

        let status = Command::new(executable.to_string())
            .status()
            .expect("failed");

        if !status.success() {
            panic!("{}", status);
        }

        let duration = Duration::from_nanos(session.timer.lap().unwrap());
        session.report_external("running", duration);

        if session.errors.is_empty() {
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }
        ()
    }
}

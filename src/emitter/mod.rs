// src/emitter/mod.rs

mod error;
mod inkwell;
mod interpreter;

use {
    crate::{
        combinator::{Combinator, Operation, Operator},
        data::{memory::Arc, Str},
        internal::{
            platform::{create_dir_all, Command, Lock},
            Artifact, RecordKind, Session, SessionError,
        },
        reporter::Error,
        tracker::{Span, TrackError},
    },
    std::sync::atomic::{AtomicBool, Ordering},
};

pub use {
    error::*,
    inkwell::{Context, ContextRef, Generator, TargetMachine},
    interpreter::{Engine, InterpretError, Value},
};

pub static CRANELIFT: AtomicBool = AtomicBool::new(false);

pub type GenerateError<'source> = Error<'source, ErrorKind<'source>>;

pub struct GenerateCombinator;

fn use_cranelift(session: &Session) -> bool {
    CRANELIFT.load(Ordering::Relaxed) || session.get_directive(Str::from("Cranelift")).is_some()
}

impl<'source>
Combinator<
'static,
Operator<Arc<Lock<Session<'source>>>>,
Operation<'source, Arc<Lock<Session<'source>>>>,
> for GenerateCombinator
{
fn combinator(
    &self,
    operator: &mut Operator<Arc<Lock<Session<'source>>>>,
    operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
) {
    let guard = operator.store.read().unwrap();
    let has_input = guard.has_input();
    let cranelift = use_cranelift(&guard);
    drop(guard);

    if !has_input {
        operation.set_resolve(Vec::new());
        return;
    }

    if cranelift {
    } else {
        generate_inkwell(operator, operation);
    }
}
}

fn generate_inkwell<'source>(
    operator: &mut Operator<Arc<Lock<Session<'source>>>>,
    operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
) {
    let mut guard = operator.store.write().unwrap();
    let session = &mut *guard;

    let context = Context::create();
    let reference = unsafe { ContextRef::new(context.raw()) };
    let mut emitter = Generator::new(reference);

    let triple = TargetMachine::get_default_triple();
    let base = session.base();

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

        let stem = Str::from(location.stem().unwrap().to_string());

        if let Some(Artifact::Analyses(analysis_ref)) = record.fetch(3) {
            let analysis = analysis_ref.clone();
            let module = emitter.context.create_module(stem.as_str().unwrap());

            module.set_triple(&triple);

            emitter.modules.insert(stem, module);
            emitter.current_module = stem;

            emitter.generate(analysis);

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
                            let string = emitter.current_module().print_to_string().to_string();
                            if let Err(error) = file.write_all(string.as_bytes()) {
                                let kind = crate::tracker::ErrorKind::from_io(error, schema);
                                let track = TrackError::new(kind, Span::void());
                                session.errors.push(SessionError::Track(track));
                                operation.set_reject();
                                return;
                            }
                            session.records.get_mut(&key).unwrap().artifacts.insert(4, Artifact::Schema(schema));
                        }
                        Err(error) => {
                            let kind = crate::tracker::ErrorKind::from_io(error, schema);
                            let track = TrackError::new(kind, Span::void());
                            session.errors.push(SessionError::Track(track));
                        }
                    }
                }
                Err(error) => session.errors.push(SessionError::Track(error)),
            }
        }
    }

    session.errors.extend(
        emitter
            .errors
            .iter()
            .map(|error| SessionError::Generate(error.clone())),
    );

    if session.errors.is_empty() {
        operation.set_resolve(Vec::new());
    } else {
        operation.set_reject();
    }
}

pub struct EmitCombinator;

impl<'source>
Combinator<
'static,
Operator<Arc<Lock<Session<'source>>>>,
Operation<'source, Arc<Lock<Session<'source>>>>,
> for EmitCombinator
{
fn combinator(
    &self,
    operator: &mut Operator<Arc<Lock<Session<'source>>>>,
    operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
) {
    let mut session = operator.store.write().unwrap();
    if !session.has_input() || session.get_directive(Str::from("Discard")).is_some() {
        if session.errors.is_empty() {
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }
        return;
    }

    let base = session.base();
    let mut direct = Vec::new();

    let mut keys: Vec<_> = session.records.keys().copied().collect();
    keys.sort();

    let target = session
        .get_target()
        .map(|t| t.as_str().unwrap().to_string());
    let msvc = target
        .as_ref()
        .map(|t| t.contains("msvc"))
        .unwrap_or_else(|| cfg!(target_env = "msvc"));

    for &key in &keys {
        let record = session.records.get_mut(&key).unwrap();

        let path = match record.kind {
            RecordKind::Source => {
                if record.fetch(5).is_some() {
                    None
                } else if let Some(Artifact::Schema(location)) = record.fetch(4) {
                    Some(location.to_string())
                } else {
                    None
                }
            }
            RecordKind::Schema => Some(record.location.to_string()),
            RecordKind::C => {
                if let Some(content) = record.content() {
                    let path = record.location.to_path().unwrap();
                    let name = path.file_name().unwrap();
                    let build = base.join("build").join("base");

                    _ = create_dir_all(&build);
                    let output = build.join(name);

                    if !output.exists() {
                        if let Ok(mut file) = crate::internal::platform::File::create(&output) {
                            use crate::internal::platform::Write;
                            _ = file.write_all(content.as_bytes());
                        }
                    }
                    Some(output.to_string_lossy().into_owned())
                } else {
                    Some(record.location.to_string())
                }
            }
            RecordKind::Object => {
                direct.push(record.location);
                None
            }
            RecordKind::Flag => None,
            RecordKind::Executable => None,
        };

        if let Some(path) = path {
            let object = Session::object(&base, record.location, &record.kind, None);
            let parent = object.to_path().unwrap().parent().unwrap().to_path_buf();
            _ = create_dir_all(&parent);

            session.records.get_mut(&key).unwrap().artifacts.insert(5, Artifact::Object(object));

            let mut command = Command::new("clang");
            if let Some(t) = &target {
                command.arg("-target").arg(t);
            }

            if msvc {
                command
                    .arg("/nologo")
                    .arg("/c")
                    .arg(path.clone())
                    .arg(format!("/Fo{}", object));
            } else {
                command
                    .arg("-w")
                    .arg("-Wno-override-module")
                    .arg("-c")
                    .arg(path.clone())
                    .arg("-o")
                    .arg(object.to_string());
            }

            let status = command
                .status()
                .expect("failed compiling: clang not found or execution failed");

            if !status.success() {
                panic!("failed compiling: {}", path);
            }
        }
    }

    let mut link = Command::new("clang");
    if let Some(t) = &target {
        link.arg("-target").arg(t);
    }

    if msvc {
        link.arg("/nologo");
    } else {
        link.arg("-w");
    }

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

    let location = if let Some(Artifact::Schema(location)) = record.fetch(4) {
        *location
    } else {
        record.location
    };

    let executable = Session::executable_path(&base, location, None);

    if msvc {
        link.arg(format!("/Fe{}", executable));
    } else {
        link.arg("-w").arg("-o").arg(executable.to_string());
    }

    let status = link
        .status()
        .expect("failed linking: clang not found or execution failed");

    if !status.success() {
        panic!("emitter failed: {}", status);
    }

    session.set_executable(executable);

    if session.errors.is_empty() {
        operation.set_resolve(Vec::new());
    } else {
        operation.set_reject();
    }
}
}

pub struct RunCombinator;

impl<'source>
Combinator<
'static,
Operator<Arc<Lock<Session<'source>>>>,
Operation<'source, Arc<Lock<Session<'source>>>>,
> for RunCombinator
{
fn combinator(
    &self,
    operator: &mut Operator<Arc<Lock<Session<'source>>>>,
    operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
) {
    let session = operator.store.write().unwrap();
    if !session.has_input() || session.get_directive(Str::from("Discard")).is_some() {
        if session.errors.is_empty() {
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }
        return;
    }

    let executable = session.get_executable().unwrap();

    session.report_execute(&executable.to_string());

    let status = Command::new(executable.to_string())
        .status()
        .expect("failed");

    if !status.success() {
        panic!("{}", status);
    }

    if session.errors.is_empty() {
        operation.set_resolve(Vec::new());
    } else {
        operation.set_reject();
    }
}
}
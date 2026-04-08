mod core;

pub use core::*;

use crate::{
    analyzer::Analyzer,
    combinator::{Action, Operation, Operator},
    data::{memory::Arc, Str},
    internal::{
        cache::{Decode, Encode},
        platform::{create_dir_all, read, write, Lock},
        time::Duration,
        CompileError,
    },
    parser::Parser,
    resolver::Resolver,
    scanner::Scanner,
};

#[cfg(not(feature = "generator"))]
use crate::interpreter::{InterpretAction, Interpreter};

#[cfg(feature = "generator")]
use crate::generator::{EmitAction, GenerateAction, RunAction};

pub struct PrepareAction;

impl<'source>
Action<
    'static,
    Operator<Arc<Lock<Session<'source>>>>,
    Operation<'source, Arc<Lock<Session<'source>>>>,
> for PrepareAction
{
    fn action(
        &self,
        operator: &mut Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) -> () {
        let mut guard = operator.store.write().unwrap();
        let session = &mut *guard;

        use crate::{
            internal::{
                hash::{DefaultHasher, Hash, Hasher, Map},
                platform::read_to_string,
            },
            tracker::Location,
        };

        let manifest = session.manifest();
        if session.cache.is_empty() && session.get_directive(Str::from("Discard")).is_none() {
            if let Ok(data) = read(&manifest) {
                if let Some(cache) = Session::decode::<Option<Map<Location<'source>, u64>>>(data).flatten() {
                    session.cache = cache;
                }
            }
        }

        let mut keys: Vec<_> = session.records.keys().copied().collect();
        keys.sort();

        for key in &keys {
            let record = session.records.get_mut(key).unwrap();

            if record.kind == InputKind::Source {
                let location = record.location;
                let path = location.to_string();

                let mut hash_content = None;

                if let Some(content) = &record.content {
                    hash_content = Some(content.clone());
                } else if let Ok(content) = read_to_string(&path) {
                    record.content = Some(content.clone());
                    hash_content = Some(content);
                }

                if let Some(content) = hash_content {
                    let mut hasher = DefaultHasher::new();
                    content.hash(&mut hasher);
                    let hash = hasher.finish();

                    record.hash = hash;

                    if let Some(&prior) = session.cache.get(&location) {
                        record.dirty = prior != hash;
                    } else {
                        record.dirty = true;
                    }

                    session.cache.insert(location, hash);
                }
            }
        }

        #[cfg(not(feature = "generator"))]
        {
            use std::ffi::CString;
            use std::process::Command;

            let mut sources = Vec::new();
            for key in &keys {
                let record = session.records.get(key).unwrap();
                if record.kind == InputKind::C {
                    if let Ok(path) = record.location.to_path() {
                        sources.push(path);
                    }
                }
            }

            if !sources.is_empty() {
                let base = session.base();
                let build = base.join("build");
                _ = create_dir_all(&build);

                let library = build.join("lib_axo.so");
                let mut command = Command::new("cc");

                command.arg("-shared").arg("-fPIC").arg("-o").arg(&library);
                for source in sources {
                    command.arg(source);
                }

                if command.status().unwrap().success() {
                    let string = library.to_str().unwrap();
                    let path = CString::new(string).unwrap();
                    unsafe {
                        if libc::dlopen(path.as_ptr(), 258).is_null() {
                            panic!();
                        }
                    }
                } else {
                    panic!();
                }
            }
        }

        if session.get_directive(Str::from("Discard")).is_none() {
            if let Some(parent) = manifest.parent() {
                _ = create_dir_all(parent);
            }
            let mut buffer = Vec::new();
            Some(session.cache.clone()).encode(&mut buffer);
            _ = write(manifest, buffer);
        }

        if session.errors.is_empty() {
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }

        ()
    }
}

impl<'session> Session<'session> {
    fn decode<T: Decode<'session>>(bytes: Vec<u8>) -> Option<T> {
        let bytes: &'static [u8] = Box::leak(bytes.into_boxed_slice());
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut cursor = 0;
            T::decode(bytes, &mut cursor)
        }))
            .ok()
    }

    pub fn cache<T: Decode<'session> + Encode + Clone>(
        &self,
        name: &str,
        hash: u64,
        data: Option<T>,
    ) -> Option<T> {
        if self.get_directive(Str::from("Discard")).is_some() {
            return data;
        }

        let base = self.base();
        let cache = base.join("build").join("records").join(name);
        _ = create_dir_all(&cache);
        let path = cache.join(format!("{:016x}", hash));

        if let Some(value) = data {
            let mut buffer = Vec::new();
            Some(value.clone()).encode(&mut buffer);
            _ = write(path, buffer);
            Some(value)
        } else if let Ok(bytes) = read(&path) {
            Self::decode(bytes).flatten()
        } else {
            None
        }
    }

    pub fn add_path(&mut self, path: &'session str) {
        use crate::tracker::Location;
        let location = Location::Entry(Str::from(path));
        let kind = InputKind::from_path(path).unwrap_or(InputKind::Source);
        let record = Record::new(kind, location);
        let id = self.records.len() | 0x40000000;
        self.records.insert(id, record);
    }

    pub fn add_string(&mut self, name: &'session str, content: String) {
        use crate::tracker::Location;
        let location = Location::Entry(Str::from(name));
        let mut record = Record::new(InputKind::Source, location);
        record.content = Some(content);
        let id = self.records.len() | 0x40000000;
        self.records.insert(id, record);
    }

    pub fn run(
        mut self,
        mut pipeline: Operation<'session, Arc<Lock<Session<'session>>>>,
    ) -> Self {
        // Restart the timer so that subsequent loops in the REPL don't fail
        _ = self.timer.start();

        if !self.errors.is_empty() {
            for error in &self.errors {
                match error {
                    CompileError::Initialize(error) => self.report_error(error),
                    CompileError::Scan(error) => self.report_error(error),
                    CompileError::Parse(error) => self.report_error(error),
                    CompileError::Resolve(error) => self.report_error(error),
                    CompileError::Analyze(error) => self.report_error(error),
                    CompileError::Interpret(error) => self.report_error(error),
                    #[cfg(feature = "generator")]
                    CompileError::Generate(error) => self.report_error(error),
                    CompileError::Track(error) => self.report_error(error),
                }
            }
            return self;
        }

        let store = Arc::new(Lock::new(self));
        let mut operator = Operator::new(store.clone());

        operator.execute(&mut pipeline);

        let mut session = store.write().unwrap();

        _ = session.timer.lap();
        let sum = session.timer.laps().iter().copied().sum::<u64>();
        let internal = Duration::from_nanos(sum);

        session.report_finish("pipeline", internal, session.errors.len());

        let total = Duration::from_nanos(session.timer.stop().unwrap());
        session.report_finish("compilation", total, session.errors.len());

        for error in &session.errors {
            match error {
                CompileError::Initialize(error) => session.report_error(error),
                CompileError::Scan(error) => session.report_error(error),
                CompileError::Parse(error) => session.report_error(error),
                CompileError::Resolve(error) => session.report_error(error),
                CompileError::Analyze(error) => session.report_error(error),
                CompileError::Interpret(error) => session.report_error(error),
                #[cfg(feature = "generator")]
                CompileError::Generate(error) => session.report_error(error),
                CompileError::Track(error) => session.report_error(error),
            }
        }

        drop(session);
        drop(operator);

        Arc::try_unwrap(store)
            .unwrap_or_else(|_| panic!())
            .into_inner()
            .unwrap()
    }

    pub fn compile(self) -> Self {
        #[cfg(not(feature = "generator"))]
        let engine = Arc::new(Lock::new(Interpreter::new(1024)));

        self.run(Operation::sequence([
            Operation::new(Arc::new(PrepareAction)),
            Operation::new(Arc::new(Scanner::default())),
            Operation::new(Arc::new(Parser::default())),
            Operation::new(Arc::new(Resolver::default())),
            Operation::new(Arc::new(Analyzer::default())),
            #[cfg(not(feature = "generator"))]
            Operation::new(Arc::new(InterpretAction::new(engine))),
            #[cfg(feature = "generator")]
            Operation::new(Arc::new(GenerateAction)),
            #[cfg(feature = "generator")]
            Operation::new(Arc::new(EmitAction)),
            #[cfg(feature = "generator")]
            Operation::new(Arc::new(RunAction)),
        ]))
    }
}
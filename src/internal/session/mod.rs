mod core;

pub use core::*;

use {
    orbyte::{Serialize, Deserialize},
    crate::{
        analyzer::Analyzer,
        combinator::{Action, Operation, Operator},
        data::{memory::{Arc, transmute}, Str},
        internal::{
            platform::{
                temp_dir,
                create_dir_all,
                read, write,
                catch_unwind,
                DLL_EXTENSION,
                AssertUnwindSafe,
                Lock,
            },
            time::Duration,
        },
        parser::Parser,
        resolver::Resolver,
        scanner::Scanner,
    }
};

#[cfg(feature = "interpreter")]
use crate::interpreter::{InterpretAction, Interpreter};

#[cfg(feature = "generator")]
use crate::generator::{EmitAction, GenerateAction, RunAction};

pub struct PrepareAction;

pub fn prepare<'source>(session: &mut Session<'source>) -> bool {
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
            session.buffers.push(data);
            let raw = session.buffers.last().unwrap().as_slice();
            let extended: &'source [u8] = unsafe { transmute(raw) };

            if let Some(cache) = Session::decode::<Option<Map<Location<'source>, u64>>>(extended).flatten() {
                session.cache = cache;
            }
        }
    }

    let mut keys: Vec<_> = session.records.keys().copied().collect();
    keys.sort();

    for key in &keys {
        let record = session.records.get_mut(key).unwrap();

        if record.kind == RecordKind::Source || record.kind == RecordKind::C {
            let location = record.location;
            let path = location.to_string();
            let mut content = None;

            record.sync_rows();
            if let Some(value) = &record.content {
                content = Some(value.clone());
            } else if let Ok(value) = read_to_string(&path) {
                record.set_content(value.clone());
                content = Some(value);
            }

            if let Some(value) = content {
                let mut hasher = DefaultHasher::new();
                value.hash(&mut hasher);
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

    #[cfg(feature = "interpreter")]
    {
        use crate::{data::CString, internal::platform::Command};

        let mut sources = Vec::new();
        let discard = session.get_directive(Str::from("Discard")).is_some();

        let build = if discard {
            temp_dir().join("axo").join("build")
        } else {
            session.base().join("build")
        };

        let mut dirty = false;

        for key in &keys {
            let record = session.records.get(key).unwrap();
            if record.kind == RecordKind::C {
                if let Ok(path) = record.location.to_path() {
                    if let Some(content) = &record.content {
                        _ = create_dir_all(&build);
                        if let Some(name) = path.file_name() {
                            let build_path = build.join(name);
                            if !build_path.exists() || record.dirty {
                                _ = write(build_path.clone(), content.as_bytes().to_vec());
                            }
                            sources.push(build_path);
                        }
                    } else {
                        sources.push(path);
                    }

                    if record.dirty {
                        dirty = true;
                    }
                }
            }
        }

        if !sources.is_empty() {
            let library = build.join(format!("lib_axo.{}", DLL_EXTENSION));
            let recompile = dirty || !library.exists();

            if recompile {
                let mut command = Command::new("cc");
                command.arg("-shared").arg("-fPIC").arg("-o").arg(&library);

                for source in sources {
                    command.arg(source);
                }

                if !command.status().expect("cc not found").success() {
                    panic!("failed to compile dynamic library");
                }
            }

            if library.exists() {
                let string = library.to_str().unwrap();
                let path = CString::new(string).unwrap();
                unsafe {
                    if libc::dlopen(path.as_ptr(), libc::RTLD_NOW | libc::RTLD_GLOBAL).is_null() {
                        panic!("dlopen failed: {}", string);
                    }
                }
            }
        }
    }

    if session.get_directive(Str::from("Discard")).is_none() {
        if let Some(parent) = manifest.parent() {
            _ = create_dir_all(parent);
        }
        let buffer = Some(session.cache.clone()).serialize();
        _ = write(manifest, buffer);
    }

    session.errors.is_empty()
}

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
        if prepare(session) {
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }
    }
}

impl<'session> Session<'session> {
    fn decode<T: Deserialize>(bytes: &'session [u8]) -> Option<T> {
        catch_unwind(AssertUnwindSafe(|| T::deserialize(bytes).ok()))
            .ok()
            .flatten()
    }

    pub fn cache<T: Deserialize + Serialize + Clone>(
        &mut self,
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
            let buffer = Some(value.clone()).serialize();
            _ = write(path, buffer);
            Some(value)
        } else if let Ok(bytes) = read(&path) {
            self.buffers.push(bytes);
            let raw = self.buffers.last().unwrap().as_slice();
            let extended: &'session [u8] = unsafe { transmute(raw) };
            Self::decode(extended).flatten()
        } else {
            None
        }
    }

    pub fn add_path(&mut self, path: &'session str) {
        use crate::tracker::Location;
        let location = Location::from(path);
        let kind = RecordKind::from_path(path).unwrap_or(RecordKind::Source);
        let record = Record::new(kind, location);
        let id = self.records.len() | 0x40000000;
        self.records.insert(id, record);
    }

    pub fn add_string(&mut self, name: &'session str, content: String) {
        use crate::tracker::Location;
        let location = Location::from(name);
        let mut record = Record::new(RecordKind::Source, location);
        record.set_content(content);
        let id = self.records.len() | 0x40000000;
        self.records.insert(id, record);
    }

    pub fn run(
        mut self,
        mut pipeline: Operation<'session, Arc<Lock<Session<'session>>>>,
    ) -> Self {
        _ = self.timer.start();

        if !self.errors.is_empty() {
            self.report_all();
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

        session.report_all();

        drop(session);
        drop(operator);

        Arc::try_unwrap(store)
            .unwrap_or_else(|_| panic!())
            .into_inner()
            .unwrap()
    }

    pub fn compile(self) -> Self {
        #[cfg(feature = "interpreter")]
        let engine = Arc::new(Lock::new(Interpreter::new(1024)));

        self.run(Operation::sequence([
            Operation::new(Arc::new(PrepareAction)),
            Operation::new(Arc::new(Scanner::default())),
            Operation::new(Arc::new(Parser::default())),
            Operation::new(Arc::new(Resolver::default())),
            Operation::new(Arc::new(Analyzer::default())),
            #[cfg(feature = "interpreter")]
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

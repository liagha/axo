mod core;

pub use core::*;

use {
    crate::{
        analyzer::Analyzer,
        combinator::{Action, Operation, Operator},
        data::{
            memory::{transmute, Arc},
            Str,
        },
        internal::{
            platform::{catch_unwind, create_dir_all, read, write, AssertUnwindSafe, Lock},
            time::{UNIX_EPOCH, Instant, Duration},
        },
        parser::Parser,
        resolver::Resolver,
        scanner::Scanner,
    },
    orbyte::{Deserialize, Serialize},
};

#[cfg(feature = "interpreter")]
use crate::{
    internal::platform::{temp_dir, DLL_EXTENSION},
    interpreter::{InterpretAction, Interpreter},
};

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
    ) {
        let mut guard = operator.store.write().unwrap();
        let session = &mut *guard;
        if session.prepare() {
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
        } else if let Ok(mut bytes) = read(&path) {
            bytes.shrink_to_fit();
            self.buffers.push(bytes);
            let raw = self.buffers.last().unwrap().as_slice();
            let extended: &'session [u8] = unsafe { transmute(raw) };

            let result = Self::decode(extended).flatten();
            if result.is_none() {
                self.buffers.pop();
            }
            result
        } else {
            None
        }
    }

    pub fn prepare(&mut self) -> bool {
        use crate::{
            internal::hash::{DefaultHasher, Hash, Hasher, Map},
            tracker::Location,
        };

        let manifest = self.manifest();
        if self.cache.is_empty() && self.get_directive(Str::from("Discard")).is_none() {
            if let Ok(mut data) = read(&manifest) {
                data.shrink_to_fit();
                self.buffers.push(data);
                let raw = self.buffers.last().unwrap().as_slice();
                let extended: &'session [u8] = unsafe { transmute(raw) };

                if let Some(cache) =
                    Session::decode::<Option<Map<Location<'session>, u64>>>(extended).flatten()
                {
                    self.cache = cache;
                } else {
                    self.buffers.pop();
                }
            }
        }

        let mut keys: Vec<_> = self.records.keys().copied().collect();
        keys.sort();

        for key in &keys {
            let record = self.records.get_mut(key).unwrap();

            if record.kind == RecordKind::Source || record.kind == RecordKind::C {
                let location = record.location;
                let mut hash = None;

                if let Some(value) = &record.content {
                    let mut hasher = DefaultHasher::new();
                    value.hash(&mut hasher);
                    hash = Some(hasher.finish());
                } else if let Ok(path) = location.to_path() {
                    if let Ok(metadata) = path.metadata() {
                        let mut hasher = DefaultHasher::new();
                        if let Ok(modified) = metadata.modified() {
                            if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                                duration.as_secs().hash(&mut hasher);
                                duration.subsec_nanos().hash(&mut hasher);
                            }
                        }
                        metadata.len().hash(&mut hasher);
                        hash = Some(hasher.finish());
                    }
                }

                if let Some(value) = hash {
                    record.hash = value;

                    if let Some(&prior) = self.cache.get(&location) {
                        record.dirty = prior != value;
                    } else {
                        record.dirty = true;
                    }

                    self.cache.insert(location, value);

                    if record.dirty {
                        if record.content.is_none() {
                            if let Ok(text) = location.get_value() {
                                record.set_content(Str::from(text));
                            }
                        }
                        record.sync_rows();
                    }
                }
            }
        }

        #[cfg(feature = "interpreter")]
        {
            let mut sources = Vec::new();
            let discard = self.get_directive(Str::from("Discard")).is_some();

            let build = if discard {
                temp_dir().join("axo").join("build")
            } else {
                self.base().join("build")
            };

            let mut dirty = false;

            for key in &keys {
                let record = self.records.get(key).unwrap();
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
                    let mut build = cc::Build::new();
                    build.compiler("clang");
                    build.opt_level(0);
                    build.host(Session::get_host());
                    build.warnings(false);
                    build.cargo_metadata(false);

                    if let Some(target) = self.get_target() {
                        build.target(target.as_str().unwrap());
                    }

                    let compiler = build.get_compiler();
                    let mut command = compiler.to_command();

                    if compiler.is_like_msvc() {
                        command.arg("/nologo").arg("/LD").arg(format!("/Fe{}", library.display()));
                    } else {
                        command.arg("-w").arg("-shared").arg("-fPIC").arg("-o").arg(&library);
                    }

                    for source in sources {
                        command.arg(source);
                    }

                    if !command.status().expect("cc not found").success() {
                        panic!("failed to compile dynamic library");
                    }
                }

                if library.exists() {
                    use crate::data::memory::forget;
                    
                    let loading = unsafe { libloading::Library::new(&library) };
                    match loading {
                        Ok(instance) => forget(instance),
                        Err(error) => panic!("failed to open library: {} - {}", library.display(), error),
                    }
                }
            }
        }

        if self.get_directive(Str::from("Discard")).is_none() {
            if let Some(parent) = manifest.parent() {
                _ = create_dir_all(parent);
            }
            let buffer = Some(self.cache.clone()).serialize();
            _ = write(manifest, buffer);
        }

        self.errors.is_empty()
    }

    pub fn add_path(&mut self, path: &'session str) {
        use crate::tracker::Location;
        let location = Location::from(path);
        let kind = RecordKind::from_path(path).unwrap_or(RecordKind::Source);
        let record = Record::new(kind, location);
        let id = self.records.len() | 0x40000000;
        self.records.insert(id, record);
    }

    pub fn add_string(&mut self, name: &'session str, content: Str<'session>) {
        use crate::tracker::Location;
        let location = Location::from(name);
        let mut record = Record::new(RecordKind::Source, location);
        record.set_content(content);
        let id = self.records.len() | 0x40000000;
        self.records.insert(id, record);
    }

    pub fn run(mut self, mut pipeline: Operation<'session, Arc<Lock<Session<'session>>>>) -> Self {
        self.timer = Instant::now();
        self.laps.clear();

        if !self.errors.is_empty() {
            self.report_all();
            return self;
        }

        let store = Arc::new(Lock::new(self));
        let mut operator = Operator::new(store.clone());
        operator.execute(&mut pipeline);

        let mut session = store.write().unwrap();

        let elapsed = session.timer.elapsed();
        session.laps.push(elapsed);

        let internal = session.laps.iter().copied().sum::<Duration>();

        session.report_finish("pipeline", internal, session.errors.len());
        let total = session.timer.elapsed();
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

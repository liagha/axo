mod core;

pub use core::*;

use {
    crate::{
        analyzer::Analyzer,
        combinator::{Combinator, Operation, Operator},
        data::{
            memory::{Arc, PhantomData},
            Identity, Module, Str,
        },
        identifier,
        internal::{
            hash::{DefaultHasher, Hash, Hasher},
            platform::{Lock, Command, var},
            time::{Duration, Instant, UNIX_EPOCH},
        },
        literal, module,
        parser::Parser,
        resolver::Resolver,
        scanner::Scanner,
    },
};

#[cfg(feature = "interpreter")]
use crate::{
    internal::platform::{temp_dir, DLL_EXTENSION},
    interpreter::Interpreter,
};

#[cfg(feature = "generator")]
use crate::generator::{EmitCombinator, GenerateCombinator, RunCombinator};

pub struct Prepare;
pub struct Bootstrap;
pub struct Report<'source> {
    pub keys: Vec<Identity>,
    pub slot: u8,
    pub head: &'static str,
    pub color: broccli::Color,
    pub phantom: PhantomData<&'source ()>,
}
pub struct Scan<'source> {
    pub keys: Vec<Identity>,
    pub phantom: PhantomData<&'source ()>,
}
pub struct Parse<'source> {
    pub keys: Vec<Identity>,
    pub phantom: PhantomData<&'source ()>,
}
pub struct Resolve<'source> {
    pub phantom: PhantomData<&'source ()>,
}
pub struct Analyze<'source> {
    pub phantom: PhantomData<&'source ()>,
}
#[cfg(feature = "interpreter")]
pub struct Interpret<'source> {
    #[cfg(feature = "interpreter")]
    pub engine: Option<Arc<Lock<Interpreter<'source>>>>,
    pub phantom: PhantomData<&'source ()>,
}

impl<'source>
Combinator<
    'static,
    Operator<Arc<Lock<Session<'source>>>>,
    Operation<'source, Arc<Lock<Session<'source>>>>,
> for Bootstrap
{
    fn combinator(
        &self,
        operator: &mut Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) {
        Session::trace("bootstrap:start");
        let mut guard = operator.store.write().unwrap();
        let session = &mut *guard;
        session.bootstrap();
        Session::trace("bootstrap:end");
        operation.set_resolve(Vec::new());
    }
}

impl<'source>
Combinator<
    'static,
    Operator<Arc<Lock<Session<'source>>>>,
    Operation<'source, Arc<Lock<Session<'source>>>>,
> for Prepare
{
    fn combinator(
        &self,
        operator: &mut Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) {
        Session::trace("prepare:start");
        let mut guard = operator.store.write().unwrap();
        let session = &mut *guard;
        if session.prepare() {
            Session::trace("prepare:end");
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }
    }
}

impl<'source>
Combinator<
    'static,
    Operator<Arc<Lock<Session<'source>>>>,
    Operation<'source, Arc<Lock<Session<'source>>>>,
> for Report<'source>
{
    fn combinator(
        &self,
        operator: &mut Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) {
        let session = operator.store.read().unwrap();
        match self.slot {
            1 => session.report_tokens(&self.keys),
            2 => session.report_elements(&self.keys),
            3 => session.report_analyses(&self.keys),
            _ => {}
        }
        let _ = self.head;
        let _ = self.color;
        operation.set_resolve(Vec::new());
    }
}

impl<'source>
Combinator<
    'static,
    Operator<Arc<Lock<Session<'source>>>>,
    Operation<'source, Arc<Lock<Session<'source>>>>,
> for Scan<'source>
{
    fn combinator(
        &self,
        operator: &mut Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) {
        Session::trace("scan:start");
        let mut session = operator.store.write().unwrap();
        let mut changed = false;
        for key in session.source_keys(&self.keys) {
            let signature = session.scan_signature(key);
            if session.stage_value(SCAN_STAGE, key) == signature
                && session.records.get(&key).unwrap().fetch(1).is_some()
            {
                continue;
            }
            let before = session.records.get(&key).map(|record| record.artifact_version(1)).unwrap_or(0);
            Scanner::execute(&mut session, &[key]);
            let after = session.records.get(&key).map(|record| record.artifact_version(1)).unwrap_or(0);
            session.set_stage(SCAN_STAGE, key, signature);
            changed |= before != after;
        }
        if session.errors.is_empty() {
            Session::trace("scan:end");
            operation.set_resolve(if changed { vec![1] } else { Vec::new() });
        } else {
            operation.set_reject();
        }
    }
}

impl<'source>
Combinator<
    'static,
    Operator<Arc<Lock<Session<'source>>>>,
    Operation<'source, Arc<Lock<Session<'source>>>>,
> for Parse<'source>
{
    fn combinator(
        &self,
        operator: &mut Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) {
        Session::trace("parse:start");
        let mut session = operator.store.write().unwrap();
        let mut changed = false;
        for key in session.source_keys(&self.keys) {
            let signature = session.parse_signature(key);
            if session.stage_value(PARSE_STAGE, key) == signature
                && session.records.get(&key).unwrap().fetch(2).is_some()
            {
                continue;
            }
            let before = session.records.get(&key).map(|record| record.artifact_version(2)).unwrap_or(0);
            Parser::execute(&mut session, &[key]);
            let after = session.records.get(&key).map(|record| record.artifact_version(2)).unwrap_or(0);
            session.set_stage(PARSE_STAGE, key, signature);
            changed |= before != after;
        }
        if session.errors.is_empty() {
            Session::trace("parse:end");
            operation.set_resolve(if changed { vec![1] } else { Vec::new() });
        } else {
            operation.set_reject();
        }
    }
}

impl<'source>
Combinator<
    'static,
    Operator<Arc<Lock<Session<'source>>>>,
    Operation<'source, Arc<Lock<Session<'source>>>>,
> for Resolve<'source>
{
    fn combinator(
        &self,
        operator: &mut Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) {
        Session::trace("resolve:start");
        let mut session = operator.store.write().unwrap();
        let targets = session.all_source_keys();
        let signature = session.resolve_signature(&targets);
        if session.stage_value(RESOLVE_STAGE, 0) == signature {
            operation.set_resolve(Vec::new());
            return;
        }
        let before = session.resolver.registry.len();
        Resolver::execute(&mut session, &targets);
        let after = session.resolver.registry.len();
        session.set_stage(RESOLVE_STAGE, 0, signature);
        if session.errors.is_empty() {
            Session::trace("resolve:end");
            operation.set_resolve(if before != after { vec![1] } else { Vec::new() });
        } else {
            operation.set_reject();
        }
    }
}

impl<'source>
Combinator<
    'static,
    Operator<Arc<Lock<Session<'source>>>>,
    Operation<'source, Arc<Lock<Session<'source>>>>,
> for Analyze<'source>
{
    fn combinator(
        &self,
        operator: &mut Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) {
        Session::trace("analyze:start");
        let mut session = operator.store.write().unwrap();
        let targets = session.all_source_keys();
        let signature = session.analyze_signature(&targets);
        if session.stage_value(ANALYZE_STAGE, 0) == signature
            && targets.iter().all(|key| session.records.get(key).unwrap().fetch(3).is_some())
        {
            operation.set_resolve(Vec::new());
            return;
        }
        let before = targets
            .iter()
            .map(|key| session.records.get(key).map(|record| record.artifact_version(3)).unwrap_or(0))
            .sum::<usize>();
        Analyzer::execute(&mut session, &targets);
        let after = targets
            .iter()
            .map(|key| session.records.get(key).map(|record| record.artifact_version(3)).unwrap_or(0))
            .sum::<usize>();
        session.set_stage(ANALYZE_STAGE, 0, signature);
        if session.errors.is_empty() {
            Session::trace("analyze:end");
            operation.set_resolve(if before != after { vec![1] } else { Vec::new() });
        } else {
            operation.set_reject();
        }
    }
}

#[cfg(feature = "interpreter")]
impl<'source>
Combinator<
    'static,
    Operator<Arc<Lock<Session<'source>>>>,
    Operation<'source, Arc<Lock<Session<'source>>>>,
> for Interpret<'source>
{
    fn combinator(
        &self,
        operator: &mut Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) {
        let mut session = operator.store.write().unwrap();
        let signature = session.interpret_signature();
        if session.stage_value(INTERPRET_STAGE, 0) == signature {
            operation.set_resolve(Vec::new());
            return;
        }
        let mut changed = false;
        if let Some(engine) = &self.engine {
            let mut core = engine.write().unwrap();
            core.reset();
            let targets = session.all_source_keys();
            Interpreter::process(&mut session, &mut core, &targets);
            session.set_stage(INTERPRET_STAGE, 0, signature);
            changed = true;
        }
        if session.errors.is_empty() {
            operation.set_resolve(if changed { vec![1] } else { Vec::new() });
        } else {
            operation.set_reject();
        }
    }
}

pub const DIRECTIVE_STAGE: u8 = 1;
pub const SCAN_STAGE: u8 = 2;
pub const PARSE_STAGE: u8 = 3;
pub const RESOLVE_STAGE: u8 = 4;
pub const ANALYZE_STAGE: u8 = 5;
pub const INTERPRET_STAGE: u8 = 6;
pub const CACHE_REV: u64 = 1;

impl<'session> Session<'session> {
    pub(crate) fn trace(stage: &str) {
        if var("AXO_TRACE").is_ok() {
            eprintln!("AXO_TRACE {}", stage);
        }
    }

    pub fn stage_key(stage: u8, key: Identity) -> Identity {
        ((stage as Identity) << 56) ^ key
    }

    pub fn source_keys(&self, keys: &[Identity]) -> Vec<Identity> {
        let mut items = keys
            .iter()
            .copied()
            .filter(|key| {
                self.records
                    .get(key)
                    .map(|record| record.kind == RecordKind::Source)
                    .unwrap_or(false)
            })
            .collect::<Vec<_>>();
        items.sort();
        items
    }

    pub fn all_source_keys(&self) -> Vec<Identity> {
        let mut keys = self
            .records
            .iter()
            .filter_map(|(&key, record)| (record.kind == RecordKind::Source).then_some(key))
            .collect::<Vec<_>>();
        keys.sort();
        keys
    }

    pub fn stage_value(&self, stage: u8, key: Identity) -> usize {
        self.pipeline.get(&Self::stage_key(stage, key)).copied().unwrap_or(0)
    }

    pub fn set_stage(&mut self, stage: u8, key: Identity, value: usize) {
        self.pipeline.insert(Self::stage_key(stage, key), value);
    }

    pub fn scan_signature(&self, key: Identity) -> usize {
        let mut hasher = DefaultHasher::new();

        if let Some(record) = self.records.get(&key) {
            record.hash.hash(&mut hasher);
            record.source_version.hash(&mut hasher);
        }

        hasher.finish() as usize
    }

    pub fn parse_signature(&self, key: Identity) -> usize {
        let mut hasher = DefaultHasher::new();

        if let Some(record) = self.records.get(&key) {
            record.hash.hash(&mut hasher);
            record.artifact_version(1).hash(&mut hasher);
        }

        hasher.finish() as usize
    }

    fn combine_signature(&self, keys: &[Identity], artifact: u8) -> usize {
        let mut hasher = DefaultHasher::new();

        for key in self.source_keys(keys) {
            key.hash(&mut hasher);
            if let Some(record) = self.records.get(&key) {
                record.hash.hash(&mut hasher);
                record.artifact_version(artifact).hash(&mut hasher);
            }
        }

        hasher.finish() as usize
    }

    pub fn resolve_signature(&self, keys: &[Identity]) -> usize {
        self.combine_signature(keys, 2)
    }

    pub fn analyze_signature(&self, keys: &[Identity]) -> usize {
        let mut hasher = DefaultHasher::new();
        self.combine_signature(keys, 2).hash(&mut hasher);
        self.stage_value(RESOLVE_STAGE, 0).hash(&mut hasher);
        hasher.finish() as usize
    }

    #[cfg(feature = "interpreter")]
    pub fn interpret_signature(&self) -> usize {
        self.combine_signature(&self.all_source_keys(), 3)
    }

    pub fn bootstrap(&mut self) {
        if self.stage_value(DIRECTIVE_STAGE, 0) != 0 {
            return;
        }

        let directive = module!(Module::new(literal!(identifier!("directive"))))
            .with_members(self.directives.clone());

        for symbol in self.directives.clone() {
            self.resolver.registry.insert(symbol.identity, symbol);
        }

        self.resolver.insert(directive);
        self.set_stage(DIRECTIVE_STAGE, 0, 1);
    }

    pub fn report_tokens(&self, keys: &[Identity]) {
        let Some(stencil) = self.get_stencil() else {
            return;
        };

        use crate::format::Show;
        use broccli::Color;

        for key in self.source_keys(keys) {
            let Some(record) = self.records.get(&key) else {
                continue;
            };

            if let Some(Artifact::Tokens(tokens)) = record.fetch(1) {
                self.report_section("Tokens", Color::Cyan, tokens.format(stencil.clone()).to_string());
            }
        }
    }

    pub fn report_elements(&self, keys: &[Identity]) {
        let Some(stencil) = self.get_stencil() else {
            return;
        };

        use crate::format::Show;
        use broccli::Color;

        for key in self.source_keys(keys) {
            let Some(record) = self.records.get(&key) else {
                continue;
            };

            if let Some(Artifact::Elements(elements)) = record.fetch(2) {
                self.report_section("Elements", Color::Cyan, elements.format(stencil.clone()).to_string());
            }
        }
    }

    pub fn report_analyses(&self, keys: &[Identity]) {
        let Some(stencil) = self.get_stencil() else {
            return;
        };

        use crate::format::Show;
        use broccli::Color;

        for key in self.source_keys(keys) {
            let Some(record) = self.records.get(&key) else {
                continue;
            };

            if let Some(Artifact::Analyses(analyses)) = record.fetch(3) {
                self.report_section("Analysis", Color::Blue, analyses.format(stencil.clone()).to_string());
            }
        }
    }

    pub fn prepare(&mut self) -> bool {
        use crate::{
            internal::hash::{DefaultHasher, Hash, Hasher},
        };

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
            use crate::{
                internal::platform::{create_dir_all, write},
            };

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
                                let mut file_dirty = record.dirty;

                                if build_path.exists() {
                                    if let Ok(existing_content) = std::fs::read(&build_path) {
                                        file_dirty = existing_content != content.as_bytes();
                                    }
                                } else {
                                    file_dirty = true;
                                }

                                if file_dirty {
                                    _ = write(build_path.clone(), content.as_bytes().to_vec());
                                    dirty = true;
                                }
                                sources.push(build_path);
                            }
                        } else {
                            sources.push(path);
                            if record.dirty {
                                dirty = true;
                            }
                        }
                    }
                }
            }

            if !sources.is_empty() {
                let library = build.join(format!("lib_axo.{}", DLL_EXTENSION));
                let recompile = dirty || !library.exists();

                if recompile {
                    let mut command = Command::new("clang");
                    let mut is_msvc = cfg!(target_env = "msvc");

                    if let Some(target) = self.get_target() {
                        let target_str = target.as_str().unwrap();
                        command.arg("-target").arg(target_str);
                        is_msvc = target_str.contains("msvc");
                    }

                    if is_msvc {
                        command.arg("/nologo").arg("/LD").arg(format!("/Fe{}", library.display()));
                    } else {
                        command.arg("-w").arg("-shared").arg("-fPIC").arg("-o").arg(&library);
                    }

                    for source in sources {
                        command.arg(source);
                    }

                    if !command.status().expect("clang not found").success() {
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

    pub fn pipeline(
        keys: Vec<Identity>,
        #[cfg(feature = "interpreter")]
        engine: Option<Arc<Lock<Interpreter<'session>>>>,
    ) -> Operation<'session, Arc<Lock<Session<'session>>>> {
        #[cfg(feature = "generator")]
        let mut states = vec![
            Operation::new(Arc::new(Bootstrap)),
            Operation::new(Arc::new(Prepare)),
            Operation::cycle(Operation::plan(vec![
                Operation::new(Arc::new(Scan {
                    keys: keys.clone(),
                    phantom: PhantomData,
                })),
                Operation::new(Arc::new(Parse {
                    keys: keys.clone(),
                    phantom: PhantomData,
                })),
                Operation::new(Arc::new(Resolve {
                    phantom: PhantomData,
                })),
                Operation::new(Arc::new(Analyze {
                    phantom: PhantomData,
                })),
                #[cfg(feature = "interpreter")]
                Operation::new(Arc::new(Interpret {
                    engine: engine.clone(),
                    phantom: PhantomData,
                })),
            ])),
            Operation::plan(vec![
                Operation::new(Arc::new(Report {
                    keys: keys.clone(),
                    slot: 1,
                    head: "Tokens",
                    color: broccli::Color::Cyan,
                    phantom: PhantomData,
                })),
                Operation::new(Arc::new(Report {
                    keys: keys.clone(),
                    slot: 2,
                    head: "Elements",
                    color: broccli::Color::Cyan,
                    phantom: PhantomData,
                })),
                Operation::new(Arc::new(Report {
                    keys: keys.clone(),
                    slot: 3,
                    head: "Analysis",
                    color: broccli::Color::Blue,
                    phantom: PhantomData,
                })),
            ]),
        ];

        #[cfg(not(feature = "generator"))]
        let states = vec![
            Operation::new(Arc::new(Bootstrap)),
            Operation::new(Arc::new(Prepare)),
            Operation::cycle(Operation::plan(vec![
                Operation::new(Arc::new(Scan {
                    keys: keys.clone(),
                    phantom: PhantomData,
                })),
                Operation::new(Arc::new(Parse {
                    keys: keys.clone(),
                    phantom: PhantomData,
                })),
                Operation::new(Arc::new(Resolve {
                    phantom: PhantomData,
                })),
                Operation::new(Arc::new(Analyze {
                    phantom: PhantomData,
                })),
                #[cfg(feature = "interpreter")]
                Operation::new(Arc::new(Interpret {
                    engine: engine.clone(),
                    phantom: PhantomData,
                })),
            ])),
            Operation::plan(vec![
                Operation::new(Arc::new(Report {
                    keys: keys.clone(),
                    slot: 1,
                    head: "Tokens",
                    color: broccli::Color::Cyan,
                    phantom: PhantomData,
                })),
                Operation::new(Arc::new(Report {
                    keys: keys.clone(),
                    slot: 2,
                    head: "Elements",
                    color: broccli::Color::Cyan,
                    phantom: PhantomData,
                })),
                Operation::new(Arc::new(Report {
                    keys: keys.clone(),
                    slot: 3,
                    head: "Analysis",
                    color: broccli::Color::Blue,
                    phantom: PhantomData,
                })),
            ]),
        ];

        #[cfg(feature = "generator")]
        {
            states.push(Operation::new(Arc::new(GenerateCombinator)));
            states.push(Operation::new(Arc::new(EmitCombinator)));
            states.push(Operation::new(Arc::new(RunCombinator)));
        }

        Operation::plan(states)
    }

    pub fn compile(self) -> Self {
        #[cfg(feature = "interpreter")]
        let engine = Arc::new(Lock::new(Interpreter::new(1024)));
        let mut keys: Vec<_> = self.records.keys().copied().collect();
        keys.sort();

        self.run(Self::pipeline(
            keys,
            #[cfg(feature = "interpreter")]
            Some(engine),
        ))
    }
}

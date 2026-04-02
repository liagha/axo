mod core;

pub use core::*;

use crate::{
    combinator::{Action, Operation, Operator},
    data::{
        memory::Arc,
        Str,
    },
    internal::{
        cache::{Decode, Encode},
        platform::{create_dir_all, read, write, Lock},
        time::Duration,
        CompileError,
    },
    scanner::{
        Token,
        ScanAction,
    },
    parser::{
        Element,
        ParseAction
    },
    resolver::ResolveAction,
    analyzer::AnalyzeAction,
    tracker::Span,
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
                let data: &'static [u8] = Box::leak(data.into_boxed_slice());
                let mut cursor = 0;

                if let Some(cache) =
                    Option::<Map<Location<'source>, u64>>::decode(data, &mut cursor)
                {
                    session.cache = cache;
                }
            }
        }

        let mut keys: Vec<_> = session.records.keys().copied().collect();
        keys.sort();

        for key in keys {
            let record = session.records.get_mut(&key).unwrap();

            if record.kind == InputKind::Source {
                let location = record.location;
                let path = location.to_string();

                if let Ok(content) = read_to_string(&path) {
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

pub struct PopulateAction;
impl<'source>
Action<
    'static,
    Operator<Arc<Lock<Session<'source>>>>,
    Operation<'source, Arc<Lock<Session<'source>>>>,
> for PopulateAction
{
    fn action(
        &self,
        operator: &mut Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) -> () {
        let mut session = operator.store.write().unwrap();
        use crate::{
            data::Module,
            parser::{ElementKind, Symbol, SymbolKind, Visibility},
            scanner::TokenKind,
        };

        let mut keys: Vec<_> = session.records.keys().copied().collect();
        keys.sort();

        let modules: Vec<_> = keys
            .into_iter()
            .filter_map(|identity| {
                let record = session.records.get_mut(&identity).unwrap();

                if record.kind != InputKind::Source {
                    return None;
                }

                let stem = Str::from(record.location.stem().unwrap().to_string());
                let span = Span::file(Str::from(record.location.to_string())).unwrap();

                let head = Element::new(
                    ElementKind::Literal(Token::new(TokenKind::Identifier(stem), span)),
                    span,
                )
                    .into();

                let mut symbol = Symbol::new(
                    SymbolKind::Module(Module::new(head)),
                    span,
                    Visibility::Public,
                );

                symbol.identity = identity;

                record.module = Some(symbol.identity);
                Some(symbol)
            })
            .collect();

        for module in modules {
            session.resolver.insert(module);
        }

        if session.errors.is_empty() {
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }
        ()
    }
}

#[allow(unused)]
pub struct InterpretAction;
impl<'source>
Action<
    'static,
    Operator<Arc<Lock<Session<'source>>>>,
    Operation<'source, Arc<Lock<Session<'source>>>>,
> for InterpretAction
{
    fn action(
        &self,
        operator: &mut Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) -> () {
        let mut session = operator.store.write().unwrap();
        use crate::interpreter::{Machine, Translator};

        let initial = session.errors.len();

        session.report_start("interpreting");

        let mut keys: Vec<_> = session
            .records
            .iter()
            .filter_map(|(&key, record)| {
                if record.kind == InputKind::Source && record.module.is_some() {
                    Some(key)
                } else {
                    None
                }
            })
            .collect();
        keys.sort();

        let mut translator = Translator::new();

        for &key in &keys {
            if let Some(analyses) = session.records.get(&key).unwrap().analyses.clone() {
                for analysis in analyses {
                    translator.walk(analysis);
                }
            }
        }

        let mut machine = Machine::new(translator.code, 1024, vec![]);

        if let Err(error) = machine.run() {
            session.errors.push(CompileError::Interpret(error.clone()));
        }

        let duration = Duration::from_nanos(session.timer.lap().unwrap());
        session.report_finish("interpreting", duration, session.errors.len() - initial);

        if session.errors.is_empty() {
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }
        ()
    }
}

impl<'session> Session<'session> {
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
            let bytes: &'static [u8] = Box::leak(bytes.into_boxed_slice());
            let mut cursor = 0;
            Option::<T>::decode(bytes, &mut cursor)
        } else {
            None
        }
    }

    pub fn compile(self) {
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
            return;
        }

        let store = Arc::new(Lock::new(self));
        let mut operator = Operator::new(store.clone());

        let mut pipeline = Operation::sequence([
            Operation::new(Arc::new(PrepareAction)),
            Operation::new(Arc::new(ScanAction)),
            Operation::new(Arc::new(ParseAction)),
            Operation::new(Arc::new(PopulateAction)),
            Operation::new(Arc::new(ResolveAction)),
            Operation::new(Arc::new(AnalyzeAction)),
            #[cfg(feature = "generator")]
            Operation::new(Arc::new(GenerateAction)),
            #[cfg(feature = "generator")]
            Operation::new(Arc::new(EmitAction)),
            #[cfg(feature = "generator")]
            Operation::new(Arc::new(RunAction)),
        ]);

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
    }
}

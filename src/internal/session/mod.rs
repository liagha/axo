// src/internal/session/mod.rs

mod core;

pub use core::*;

use crate::{
    analyzer::Analyzer,
    combinator::{Combinator, Operation, Operator},
    data::{
        memory::Arc,
        Identity, Module, Str,
    },
    identifier,
    initializer::Initializer,
    internal::{
        platform::Lock,
        time::{Duration, Instant},
        SessionError,
    },
    literal, module,
    parser::Parser,
    resolver::Resolver,
    scanner::Scanner,
    tracker::{TrackError, ErrorKind as TrackErrorKind},
};

#[cfg(feature = "emitter")]
use crate::emitter::{EmitCombinator, Engine, GenerateCombinator, RunCombinator, Value};

pub struct Initialize {
    pub flag: Str<'static>,
}

impl<'source>
Combinator<
'static,
Operator<Arc<Lock<Session<'source>>>>,
Operation<'source, Arc<Lock<Session<'source>>>>,
> for Initialize
{
fn combinator(
    &self,
    operator: &mut Operator<Arc<Lock<Session<'source>>>>,
    operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
) {
    let mut guard = operator.store.write().unwrap();
    let session = &mut *guard;

    let mut initializer = Initializer::new(self.flag);
    let targets = initializer.initialize();

    for (target, span) in targets {
        let string = target.to_string();
        if let Some(kind) = RecordKind::from_path(&string) {
            let mut hasher = crate::internal::hash::DefaultHasher::new();
            crate::internal::hash::Hash::hash(&string, &mut hasher);
            let identity = (crate::internal::hash::Hasher::finish(&hasher) as Identity) | 0x40000000;
            session.records.insert(identity, Record::new(kind, target));
        } else {
            session.errors.push(SessionError::Track(TrackError::new(
                TrackErrorKind::UnSupportedInput(target),
                span,
            )));
        }
    }

    let directive = module!(Module::new(literal!(identifier!("directive"))))
        .with_members(initializer.output.clone());
    for symbol in initializer.output {
        session.resolver.registry.insert(symbol.identity, symbol);
    }
    session.resolver.insert(directive);

    for error in initializer.errors {
        session.errors.push(SessionError::Initialize(error));
    }

    if session.errors.is_empty() {
        operation.set_resolve(Vec::new());
    } else {
        operation.set_reject();
    }
}
}

pub struct Prepare;

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
    let mut guard = operator.store.write().unwrap();
    let session = &mut *guard;
    if session.prepare() {
        operation.set_resolve(Vec::new());
    } else {
        operation.set_reject();
    }
}
}

pub struct Report;

impl<'source>
Combinator<
'static,
Operator<Arc<Lock<Session<'source>>>>,
Operation<'source, Arc<Lock<Session<'source>>>>,
> for Report
{
fn combinator(
    &self,
    operator: &mut Operator<Arc<Lock<Session<'source>>>>,
    operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
) {
    let session = operator.store.read().unwrap();
    let keys = session.all_source_keys();
    session.report_tokens(&keys);
    session.report_elements(&keys);
    session.report_analyses(&keys);
    operation.set_resolve(Vec::new());
}
}

impl<'session> Session<'session> {
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
                self.report_section(
                    "Tokens",
                    Color::Cyan,
                    tokens.format(stencil.clone()).to_string(),
                );
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
                self.report_section(
                    "Elements",
                    Color::Cyan,
                    elements.format(stencil.clone()).to_string(),
                );
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
                self.report_section(
                    "Analysis",
                    Color::Blue,
                    analyses.format(stencil.clone()).to_string(),
                );
            }
        }
    }

    pub fn prepare(&mut self) -> bool {
        let mut keys: Vec<_> = self.records.keys().copied().collect();
        keys.sort();

        for key in &keys {
            let record = self.records.get_mut(key).unwrap();

            if record.kind == RecordKind::Source || record.kind == RecordKind::C {
                if record.content().is_none() {
                    if let Ok(text) = record.location.get_value() {
                        record.set_content(Str::from(text));
                    }
                }
            }
        }

        self.errors.is_empty()
    }

    #[cfg(feature = "emitter")]
    pub fn execute_line(
        &self,
        engine: &mut Engine<'session>,
        key: Identity,
    ) -> Result<Option<Value<'session>>, crate::emitter::InterpretError<'session>> {
        let record = match self.records.get(&key) {
            Some(r) => r,
            None => return Ok(None),
        };
        let analyses = if let Some(Artifact::Analyses(analyses)) = record.fetch(3) {
            analyses.clone()
        } else {
            return Ok(None);
        };
        let result = engine.process(analyses)?;
        Ok(Some(result))
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

    pub fn pipeline() -> Operation<'session, Arc<Lock<Session<'session>>>> {
        let states = vec![
            Operation::new(Arc::new(Initialize { flag: Session::arguments() })),
            Operation::new(Arc::new(Prepare)),
            Operation::new(Arc::new(Scanner::default())),
            Operation::new(Arc::new(Parser::default())),
            Operation::new(Arc::new(Resolver::default())),
            Operation::new(Arc::new(Analyzer::default())),
            Operation::new(Arc::new(Report)),
        ];

        #[cfg(feature = "emitter")]
        let states = {
            let mut states = states;
            states.push(Operation::new(Arc::new(GenerateCombinator)));
            states.push(Operation::new(Arc::new(EmitCombinator)));
            states.push(Operation::new(Arc::new(RunCombinator)));
            states
        };

        Operation::plan(states)
    }
    
    pub fn compile(self) -> Self {
        self.run(Self::pipeline())
    }
}
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
    internal::{
        platform::{var, Lock},
        time::{Duration, Instant},
    },
    literal, module,
    parser::Parser,
    resolver::Resolver,
    scanner::Scanner,
};

#[cfg(feature = "dialog")]
use crate::{emitter::CraneliftEngine, internal::SessionError};

#[cfg(feature = "emitter")]
use crate::emitter::{EmitCombinator, GenerateCombinator, RunCombinator};

pub struct Prepare;
pub struct Bootstrap;
pub struct Report;
pub struct Scan;
pub struct Parse;
pub struct Resolve;
pub struct Analyze;
#[cfg(feature = "dialog")]
pub struct Interpret<'a> {
    #[cfg(feature = "dialog")]
    pub engine: Option<Arc<Lock<CraneliftEngine<'a>>>>,
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

impl<'source>
Combinator<
    'static,
    Operator<Arc<Lock<Session<'source>>>>,
    Operation<'source, Arc<Lock<Session<'source>>>>,
> for Scan
{
    fn combinator(
        &self,
        operator: &mut Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) {
        Session::trace("scan:start");
        let mut session = operator.store.write().unwrap();
        let keys = session.all_source_keys();
        Scanner::execute(&mut session, &keys);
        if session.errors.is_empty() {
            Session::trace("scan:end");
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
> for Parse
{
    fn combinator(
        &self,
        operator: &mut Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) {
        Session::trace("parse:start");
        let mut session = operator.store.write().unwrap();
        let keys = session.all_source_keys();
        Parser::execute(&mut session, &keys);
        if session.errors.is_empty() {
            Session::trace("parse:end");
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
> for Resolve
{
    fn combinator(
        &self,
        operator: &mut Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) {
        Session::trace("resolve:start");
        let mut session = operator.store.write().unwrap();
        let keys = session.all_source_keys();
        Resolver::execute(&mut session, &keys);
        if session.errors.is_empty() {
            Session::trace("resolve:end");
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
> for Analyze
{
    fn combinator(
        &self,
        operator: &mut Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) {
        Session::trace("analyze:start");
        let mut session = operator.store.write().unwrap();
        let keys = session.all_source_keys();
        Analyzer::execute(&mut session, &keys);
        if session.errors.is_empty() {
            Session::trace("analyze:end");
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }
    }
}

#[cfg(feature = "dialog")]
impl<'source>
Combinator<
    'static,
    Operator<Arc<Lock<Session<'source>>>>,
    Operation<'source, Arc<Lock<Session<'source>>>>,
> for Interpret<'_>
{
    fn combinator(
        &self,
        operator: &mut Operator<Arc<Lock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<Lock<Session<'source>>>>,
    ) {
        Session::trace("interpret:start");
        let mut session = operator.store.write().unwrap();
        if let Some(engine) = &self.engine {
            let mut core = engine.write().unwrap();
            core.reset();
            let keys = session.all_source_keys();
            if let Err(error) = core.process(&session, &keys) {
                session.errors.push(SessionError::Generate(error));
            }
        }
        if session.errors.is_empty() {
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }
    }
}

pub const DIRECTIVE_STAGE: u8 = 1;

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
        self.pipeline
            .get(&Self::stage_key(stage, key))
            .copied()
            .unwrap_or(0)
    }

    pub fn set_stage(&mut self, stage: u8, key: Identity, value: usize) {
        self.pipeline.insert(Self::stage_key(stage, key), value);
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
                if record.content.is_none() {
                    if let Ok(text) = record.location.get_value() {
                        record.set_content(Str::from(text));
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
        #[cfg(feature = "dialog")] engine: Option<Arc<Lock<CraneliftEngine<'session>>>>,
    ) -> Operation<'session, Arc<Lock<Session<'session>>>> {
        let mut states = vec![
            Operation::new(Arc::new(Bootstrap)),
            Operation::new(Arc::new(Prepare)),
            Operation::new(Arc::new(Scan)),
            Operation::new(Arc::new(Parse)),
            Operation::new(Arc::new(Resolve)),
            Operation::new(Arc::new(Analyze)),
        ];

        states.push(Operation::new(Arc::new(Report)));

        #[cfg(feature = "dialog")]
        states.push(Operation::new(Arc::new(Interpret {
            engine,
        })));

        #[cfg(feature = "emitter")]
        {
            states.push(Operation::new(Arc::new(GenerateCombinator)));
            states.push(Operation::new(Arc::new(EmitCombinator)));
            states.push(Operation::new(Arc::new(RunCombinator)));
        }

        Operation::plan(states)
    }

    pub fn compile(self) -> Self {
        #[cfg(feature = "dialog")]
        let engine: Option<Arc<Lock<CraneliftEngine<'session>>>> = None;

        self.run(Self::pipeline(
            #[cfg(feature = "dialog")]
            engine,
        ))
    }
}
mod core;

pub use core::*;

use {
    crate::{
        combinator::{Action, Operation, Operator},
        data::{
            memory::Arc,
            Str,
        },
        format::Show,
        internal::{
            cache::{Decode, Encode},
            platform::{create_dir_all, read, write},
            time::Duration,
        },
        parser::Element,
        scanner::Token,
        tracker::{Peekable, Span},
    },
    broccli::Color,
    std::sync::RwLock,
};

#[cfg(feature = "generator")]
use crate::internal::platform::Command;

pub struct PrepareAction;

impl<'source>
Action<
    'static,
    Operator<Arc<RwLock<Session<'source>>>>,
    Operation<'source, Arc<RwLock<Session<'source>>>>,
> for PrepareAction
{
    fn action(
        &self,
        operator: &mut Operator<Arc<RwLock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<RwLock<Session<'source>>>>,
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
    Operator<Arc<RwLock<Session<'source>>>>,
    Operation<'source, Arc<RwLock<Session<'source>>>>,
> for PopulateAction
{
    fn action(
        &self,
        operator: &mut Operator<Arc<RwLock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<RwLock<Session<'source>>>>,
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

pub struct ScanAction;
impl<'source>
Action<
    'static,
    Operator<Arc<RwLock<Session<'source>>>>,
    Operation<'source, Arc<RwLock<Session<'source>>>>,
> for ScanAction
{
    fn action(
        &self,
        operator: &mut Operator<Arc<RwLock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<RwLock<Session<'source>>>>,
    ) -> () {
        let mut session = operator.store.write().unwrap();
        use crate::scanner::Scanner;

        let initial = session.errors.len();
        session.report_start("scanning");

        let mut keys: Vec<_> = session.records.keys().copied().collect();
        keys.sort();

        for key in keys {
            let (kind, hash, dirty, location) = {
                let record = session.records.get(&key).unwrap();
                (
                    record.kind.clone(),
                    record.hash,
                    record.dirty,
                    record.location,
                )
            };

            if kind != InputKind::Source {
                continue;
            }

            if !dirty {
                if let Some(tokens) = session.cache::<Vec<Token>>("tokens", hash, None) {
                    session.records.get_mut(&key).unwrap().tokens = Some(tokens);
                    continue;
                }
            }

            let mut scanner = Scanner::new(location);
            scanner.prepare();
            scanner.scan();

            if let Some(stencil) = session.get_stencil() {
                session.report_section(
                    "Tokens",
                    Color::Cyan,
                    scanner.output.format(stencil).to_string(),
                );
            }

            session.errors.extend(
                scanner
                    .errors
                    .iter()
                    .map(|error| CompileError::Scan(error.clone())),
            );

            session.records.get_mut(&key).unwrap().tokens =
                session.cache("tokens", hash, Some(scanner.output));
        }

        let duration = Duration::from_nanos(session.timer.lap().unwrap());
        session.report_finish("scanning", duration, session.errors.len() - initial);

        if session.errors.is_empty() {
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }
        ()
    }
}

pub struct ParseAction;
impl<'source>
Action<
    'static,
    Operator<Arc<RwLock<Session<'source>>>>,
    Operation<'source, Arc<RwLock<Session<'source>>>>,
> for ParseAction
{
    fn action(
        &self,
        operator: &mut Operator<Arc<RwLock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<RwLock<Session<'source>>>>,
    ) -> () {
        let mut session = operator.store.write().unwrap();
        use crate::parser::Parser;

        let initial = session.errors.len();
        session.report_start("parsing");

        let mut keys: Vec<_> = session.records.keys().copied().collect();
        keys.sort();

        for key in keys {
            let (kind, hash, dirty, location, tokens) = {
                let record = session.records.get(&key).unwrap();
                (
                    record.kind.clone(),
                    record.hash,
                    record.dirty,
                    record.location,
                    record.tokens.clone(),
                )
            };

            if kind != InputKind::Source {
                continue;
            }

            if !dirty {
                if let Some(elements) = session.cache::<Vec<Element>>("elements", hash, None) {
                    session.records.get_mut(&key).unwrap().elements = Some(elements);
                    continue;
                }
            }

            let mut parser = Parser::new(location);
            parser.set_input(tokens.unwrap());
            parser.parse();

            if let Some(stencil) = session.get_stencil() {
                session.report_section(
                    "Elements",
                    Color::Cyan,
                    parser.output.format(stencil).to_string(),
                );
            }

            session.errors.extend(
                parser
                    .errors
                    .iter()
                    .map(|error| CompileError::Parse(error.clone())),
            );

            session.records.get_mut(&key).unwrap().elements =
                session.cache("elements", hash, Some(parser.output));
        }

        let duration = Duration::from_nanos(session.timer.lap().unwrap());
        session.report_finish("parsing", duration, session.errors.len() - initial);

        if session.errors.is_empty() {
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }
        ()
    }
}

pub struct ResolveAction;
impl<'source>
Action<
    'static,
    Operator<Arc<RwLock<Session<'source>>>>,
    Operation<'source, Arc<RwLock<Session<'source>>>>,
> for ResolveAction
{
    fn action(
        &self,
        operator: &mut Operator<Arc<RwLock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<RwLock<Session<'source>>>>,
    ) -> () {
        let mut guard = operator.store.write().unwrap();
        let session = &mut *guard;

        use crate::{
            data::memory::replace,
            resolver::{Resolvable, Scope},
        };

        let initial = session.errors.len();
        session.report_start("resolving");

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

        for &key in &keys {
            let target = session.records.get(&key).unwrap().module.unwrap();
            let mut module = session.resolver.get_symbol(target).unwrap().clone();
            let scope = replace(&mut module.scope, Scope::new(None));

            session.resolver.enter_scope(scope);

            let elements = session
                .records
                .get_mut(&key)
                .unwrap()
                .elements
                .as_mut()
                .unwrap();

            for element in elements.iter_mut() {
                element.declare(&mut session.resolver);
            }

            let active = session.resolver.active;
            session.resolver.exit();

            module.scope = session.resolver.scopes.remove(&active).unwrap();
            session.resolver.insert(module);
        }

        if let Some(stencil) = session.get_stencil() {
            session.report_section(
                "Symbols",
                Color::Blue,
                session
                    .resolver
                    .collect()
                    .iter()
                    .map(|symbol| {
                        let children = symbol
                            .scope
                            .symbols
                            .iter()
                            .filter_map(|identity| session.resolver.get_symbol(*identity))
                            .collect::<Vec<_>>()
                            .format(stencil.clone())
                            .to_string();

                        format!(
                            "{}\n{}\n",
                            symbol.format(stencil.clone()),
                            children.indent(stencil.clone())
                        )
                    })
                    .collect::<Vec<String>>()
                    .join("\n"),
            )
        }

        for &key in &keys {
            let target = session.records.get(&key).unwrap().module.unwrap();
            let mut module = session.resolver.get_symbol(target).unwrap().clone();
            let scope = replace(&mut module.scope, Scope::new(None));

            session.resolver.enter_scope(scope);

            let elements = session
                .records
                .get_mut(&key)
                .unwrap()
                .elements
                .as_mut()
                .unwrap();

            for element in elements.iter_mut() {
                element.resolve(&mut session.resolver);
            }

            let active = session.resolver.active;
            session.resolver.exit();

            module.scope = session.resolver.scopes.remove(&active).unwrap();
            session.resolver.insert(module);
        }

        for &key in &keys {
            let target = session.records.get(&key).unwrap().module.unwrap();
            let mut module = session.resolver.get_symbol(target).unwrap().clone();
            let scope = replace(&mut module.scope, Scope::new(None));

            session.resolver.enter_scope(scope);

            let elements = session
                .records
                .get_mut(&key)
                .unwrap()
                .elements
                .as_mut()
                .unwrap();

            for element in elements.iter_mut() {
                element.reify(&mut session.resolver);
            }

            let active = session.resolver.active;
            session.resolver.exit();

            module.scope = session.resolver.scopes.remove(&active).unwrap();
            session.resolver.insert(module);
        }

        session
            .errors
            .extend(session.resolver.errors.drain(..).map(CompileError::Resolve));

        let duration = Duration::from_nanos(session.timer.lap().unwrap());
        session.report_finish("resolving", duration, session.errors.len() - initial);

        if session.errors.is_empty() {
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }
        ()
    }
}

pub struct AnalyzeAction;
impl<'source>
Action<
    'static,
    Operator<Arc<RwLock<Session<'source>>>>,
    Operation<'source, Arc<RwLock<Session<'source>>>>,
> for AnalyzeAction
{
    fn action(
        &self,
        operator: &mut Operator<Arc<RwLock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<RwLock<Session<'source>>>>,
    ) -> () {
        let mut session = operator.store.write().unwrap();
        use crate::analyzer::{Analysis, Analyzer};

        let initial = session.errors.len();

        session.report_start("analyzing");

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

        for &key in &keys {
            let (hash, dirty, elements) = {
                let record = session.records.get(&key).unwrap();
                (record.hash, record.dirty, record.elements.clone())
            };

            if !dirty {
                if let Some(analyses) = session.cache::<Vec<Analysis>>("analyses", hash, None) {
                    session.records.get_mut(&key).unwrap().analyses = Some(analyses);
                    continue;
                }
            }

            let mut analyzer = Analyzer::new(elements.unwrap());
            analyzer.analyze(&mut session.resolver);

            if let Some(stencil) = session.get_stencil() {
                session.report_section(
                    "Analysis",
                    Color::Blue,
                    analyzer.output.format(stencil).to_string(),
                );
            }

            session.errors.extend(
                analyzer
                    .errors
                    .iter()
                    .map(|error| CompileError::Analyze(error.clone())),
            );

            session.records.get_mut(&key).unwrap().analyses =
                session.cache("analyses", hash, Some(analyzer.output));
        }

        let duration = Duration::from_nanos(session.timer.lap().unwrap());
        session.report_finish("analyzing", duration, session.errors.len() - initial);

        if session.errors.is_empty() {
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }
        ()
    }
}

pub struct InterpretAction;
impl<'source>
Action<
    'static,
    Operator<Arc<RwLock<Session<'source>>>>,
    Operation<'source, Arc<RwLock<Session<'source>>>>,
> for InterpretAction
{
    fn action(
        &self,
        operator: &mut Operator<Arc<RwLock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<RwLock<Session<'source>>>>,
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

#[cfg(feature = "generator")]
pub struct GenerateAction;

#[cfg(feature = "generator")]
impl<'source>
Action<
    'static,
    Operator<Arc<RwLock<Session<'source>>>>,
    Operation<'source, Arc<RwLock<Session<'source>>>>,
> for GenerateAction
{
    fn action(
        &self,
        operator: &mut Operator<Arc<RwLock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<RwLock<Session<'source>>>>,
    ) -> () {
        let mut guard = operator.store.write().unwrap();
        let session = &mut *guard;

        use {
            crate::{
                generator::{Backend, Generator},
                tracker::{error::ErrorKind as TrackErrorKind, TrackError},
            },
            inkwell::{
                context::{Context, ContextRef},
                targets::TargetMachine,
            },
        };

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
                if record.kind == InputKind::Source && record.module.is_some() {
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
                record.output = Some(schema);
                continue;
            }

            let stem = Str::from(location.stem().unwrap().to_string());

            if let Some(analysis) = record.analyses.clone() {
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
                                    session.errors.push(CompileError::Track(track));
                                    operation.set_reject();
                                    return ();
                                }
                                record.output = Some(schema);
                            }
                            Err(error) => {
                                let kind = TrackErrorKind::from_io(error, schema);
                                let track = TrackError::new(kind, Span::void());
                                session.errors.push(CompileError::Track(track));
                            }
                        }
                    }
                    Err(error) => session.errors.push(CompileError::Track(error)),
                }
            }
        }

        let duration = Duration::from_nanos(session.timer.lap().unwrap());
        session.report_finish("generating", duration, session.errors.len() - initial);

        session.errors.extend(generator
                                  .errors
                                  .iter()
                                  .map(|error| CompileError::Generate(error.clone())),
        );

        if session.errors.is_empty() {
            operation.set_resolve(Vec::new());
        } else {
            operation.set_reject();
        }
        ()
    }
}

#[cfg(feature = "generator")]
pub struct EmitAction;

#[cfg(feature = "generator")]
impl<'source>
Action<
    'static,
    Operator<Arc<RwLock<Session<'source>>>>,
    Operation<'source, Arc<RwLock<Session<'source>>>>,
> for EmitAction
{
    fn action(
        &self,
        operator: &mut Operator<Arc<RwLock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<RwLock<Session<'source>>>>,
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
                InputKind::Source => record.output,
                InputKind::Schema | InputKind::C => Some(record.location),
                InputKind::Object => {
                    direct.push(record.location);
                    None
                }
            };

            if let Some(path) = target {
                let object = Session::object(&base, record.location, &record.kind, None);
                let parent = object.to_path().unwrap().parent().unwrap().to_path_buf();
                _ = create_dir_all(&parent);

                record.object = Some(object);

                if !record.dirty && object.to_path().map(|p| p.exists()).unwrap_or(false) {
                    continue;
                }

                let mut command = Command::new("clang");

                command
                    .arg("-c")
                    .arg(path.to_string())
                    .arg("-o")
                    .arg(object.to_string());

                let status = command.status().expect("failed");

                if !status.success() {
                    panic!("failed {}", path);
                }
            }
        }

        let mut link = Command::new("clang");

        for &key in &keys {
            if let Some(object) = session.records.get(&key).unwrap().object {
                link.arg(object.to_string());
            }
        }

        for object in direct {
            link.arg(object.to_string());
        }

        let key = *keys.last().expect("missing");

        let record = session.records.get(&key).unwrap();
        let location = record.output.unwrap_or(record.location);

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

#[cfg(feature = "generator")]
pub struct RunAction;

#[cfg(feature = "generator")]
impl<'source>
Action<
    'static,
    Operator<Arc<RwLock<Session<'source>>>>,
    Operation<'source, Arc<RwLock<Session<'source>>>>,
> for RunAction
{
    fn action(
        &self,
        operator: &mut Operator<Arc<RwLock<Session<'source>>>>,
        operation: &mut Operation<'source, Arc<RwLock<Session<'source>>>>,
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

        let store = Arc::new(RwLock::new(self));
        let mut operator = Operator::new(store.clone());

        #[cfg(not(feature = "generator"))]
        let mut pipeline = Operation::sequence([
            Operation::new(Arc::new(PrepareAction)),
            Operation::new(Arc::new(ScanAction)),
            Operation::new(Arc::new(ParseAction)),
            Operation::new(Arc::new(PopulateAction)),
            Operation::new(Arc::new(ResolveAction)),
            Operation::new(Arc::new(AnalyzeAction)),
        ]);

        #[cfg(feature = "generator")]
        let mut pipeline = Operation::sequence([
            Operation::new(Arc::new(PrepareAction)),
            Operation::new(Arc::new(ScanAction)),
            Operation::new(Arc::new(ParseAction)),
            Operation::new(Arc::new(PopulateAction)),
            Operation::new(Arc::new(ResolveAction)),
            Operation::new(Arc::new(AnalyzeAction)),
            Operation::new(Arc::new(GenerateAction)),
            Operation::new(Arc::new(EmitAction)),
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

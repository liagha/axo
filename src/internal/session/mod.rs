mod core;

pub use core::*;

use {
    broccli::{Color},
    crate::{
        analyzer::{Analysis, Analyzer},
        data::{memory::replace, Module, Str},
        format::Show,
        internal::{
            cache::{Decode, Encode},
            hash::{DefaultHasher, Hash, Hasher, Map},
            platform::{read, read_to_string, write},
            timer::Duration,
        },
        parser::{Element, ElementKind, Parser, Symbol, SymbolKind, Visibility},
        resolver::{Resolvable, Scope},
        scanner::{Scanner, Token, TokenKind},
        tracker::{
            Peekable, Span,
            Location,
        },
    },
};

#[cfg(feature = "generator")]
use {
    crate::{
        generator::Backend,
        internal::platform::{create_dir_all, Command},
        tracker::{
            TrackError,
            error::ErrorKind as TrackErrorKind
        },
    },
    inkwell::targets::TargetMachine
};

impl<'session> Session<'session> {
    const PIPELINE: [fn(&mut Session<'session>); 6] = [
        Self::prepare,
        Self::scan,
        Self::parse,
        Self::populate,
        Self::resolve,
        Self::analyze,
    ];

    pub fn compile(&mut self) {
        'pipeline: {
            for stage in Self::PIPELINE {
                stage(self);

                if !self.errors.is_empty() {
                    break 'pipeline;
                }
            }

            #[cfg(feature = "generator")]
            self.generate();
            if !self.errors.is_empty() {
                break 'pipeline;
            }

            _ = self.timer.lap();

            let sum = self.timer.laps().iter().copied().sum::<u64>();
            let internal = Duration::from_nanos(sum);

            self.report_finish("pipeline", internal, self.errors.len());

            #[cfg(feature = "generator")]
            self.emit();
            if !self.errors.is_empty() {
                break 'pipeline;
            }

            #[cfg(feature = "generator")]
            self.run();
        }

        let total = Duration::from_nanos(self.timer.stop().unwrap());
        self.report_finish("compilation", total, self.errors.len());

        for error in &self.errors {
            match error {
                CompileError::Initialize(error) => self.report_error(error),
                CompileError::Scan(error) => self.report_error(error),
                CompileError::Parse(error) => self.report_error(error),
                CompileError::Resolve(error) => self.report_error(error),
                CompileError::Analyze(error) => self.report_error(error),
                #[cfg(feature = "generator")]
                CompileError::Generate(error) => self.report_error(error),
                CompileError::Track(error) => self.report_error(error),
            }
        }
    }

    pub fn prepare(&mut self) {
        let manifest_file = self.manifest();
        if self.cache.is_empty() {
            if let Ok(data) = read(&manifest_file) {
                let data: &'static [u8] = Box::leak(data.into_boxed_slice());
                let mut cursor = 0;

                if let Some(loaded_cache) = Option::<Map<Location<'session>, u64>>::decode(data, &mut cursor) {
                    self.cache = loaded_cache;
                }
            }
        }

        let mut keys: Vec<_> = self.records.keys().copied().collect();
        keys.sort();

        for key in keys {
            let record = self.records.get_mut(&key).unwrap();

            if record.kind == InputKind::Source {
                let location = record.location;
                let path = location.to_string();

                if let Ok(content) = read_to_string(&path) {
                    let mut hasher = DefaultHasher::new();
                    content.hash(&mut hasher);
                    let hash = hasher.finish();

                    record.hash = hash;

                    if let Some(&prior) = self.cache.get(&location) {
                        record.dirty = prior != hash;
                    } else {
                        record.dirty = true;
                    }

                    self.cache.insert(location, hash);
                }
            }
        }

        let mut buffer = Vec::new();
        Some(self.cache.clone()).encode(&mut buffer);
        _ = write(manifest_file, buffer);
    }

    pub fn populate(&mut self) {
        let mut keys: Vec<_> = self.records.keys().copied().collect();
        keys.sort();

        let modules: Vec<_> = keys
            .into_iter()
            .filter_map(|identity| {
                let record = self.records.get_mut(&identity).unwrap();

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
            self.resolver.insert(module);
        }
    }

    pub fn scan(&mut self) {
        let initial = self.errors.len();
        self.report_start("scanning");

        let mut keys: Vec<_> = self.records.keys().copied().collect();
        keys.sort();

        for key in keys {
            let (kind, hash, dirty, location) = {
                let record = self.records.get(&key).unwrap();
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

            let file = self.cache("tokens", hash);

            if !dirty {
                if let Ok(data) = read(&file) {
                    let data: &'static [u8] = Box::leak(data.into_boxed_slice());
                    let mut cursor = 0;
                    let tokens = Option::<Vec<Token>>::decode(data, &mut cursor);
                    self.records.get_mut(&key).unwrap().tokens = tokens;
                    continue;
                }
            }

            let mut scanner = Scanner::new(location);
            scanner.prepare();
            scanner.scan();

            if let Some(stencil) = self.get_stencil() {
                self.report_section(
                    "Tokens",
                    Color::Cyan,
                    scanner
                        .output
                        .format(stencil).to_string()
                );
            }

            self.errors.extend(
                scanner
                    .errors
                    .iter()
                    .map(|error| CompileError::Scan(error.clone())),
            );

            let mut buffer = Vec::new();
            Some(scanner.output.clone()).encode(&mut buffer);
            _ = write(file, buffer);

            self.records.get_mut(&key).unwrap().tokens = Some(scanner.output);
        }

        let duration = Duration::from_nanos(self.timer.lap().unwrap());
        self.report_finish("scanning", duration, self.errors.len() - initial);
    }

    pub fn parse(&mut self) {
        let initial = self.errors.len();
        self.report_start("parsing");

        let mut keys: Vec<_> = self.records.keys().copied().collect();
        keys.sort();

        for key in keys {
            let (kind, hash, dirty, location, tokens) = {
                let record = self.records.get(&key).unwrap();
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

            let file = self.cache("elements", hash);

            if !dirty {
                if let Ok(data) = read(&file) {
                    let data: &'static [u8] = Box::leak(data.into_boxed_slice());
                    let mut cursor = 0;
                    let elements = Option::<Vec<Element>>::decode(data, &mut cursor);
                    self.records.get_mut(&key).unwrap().elements = elements;
                    continue;
                }
            }

            let mut parser = Parser::new(location);
            parser.set_input(tokens.unwrap());
            parser.parse();

            if let Some(stencil) = self.get_stencil() {
                self.report_section(
                    "Elements",
                    Color::Cyan,
                    parser
                        .output
                        .format(stencil).to_string()
                );
            }

            self.errors.extend(
                parser
                    .errors
                    .iter()
                    .map(|error| CompileError::Parse(error.clone())),
            );

            let mut buffer = Vec::new();
            Some(parser.output.clone()).encode(&mut buffer);
            _ = write(file, buffer);

            self.records.get_mut(&key).unwrap().elements = Some(parser.output);
        }

        let duration = Duration::from_nanos(self.timer.lap().unwrap());
        self.report_finish("parsing", duration, self.errors.len() - initial);
    }

    pub fn resolve(&mut self) {
        let initial = self.errors.len();
        self.report_start("resolving");

        let mut keys: Vec<_> = self.records
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
            let target = self.records.get(&key).unwrap().module.unwrap();
            let mut module = self.resolver.get_symbol(target).unwrap().clone();
            let scope = replace(&mut module.scope, Scope::new(None));

            self.resolver.enter_scope(scope);

            let elements = self.records.get_mut(&key).unwrap().elements.as_mut().unwrap();

            for element in elements.iter_mut() {
                element.declare(&mut self.resolver);
            }

            let active = self.resolver.active;
            self.resolver.exit();

            module.scope = self.resolver.scopes.remove(&active).unwrap();
            self.resolver.insert(module);
        }

        if let Some(stencil) = self.get_stencil() {
            self.report_section(
                "Symbols",
                Color::Blue,
                self.resolver
                    .collect()
                    .iter()
                    .map(|symbol| {
                        let children = symbol
                            .scope
                            .symbols
                            .iter()
                            .filter_map(|identity| self.resolver.get_symbol(*identity))
                            .collect::<Vec<_>>()
                            .format(stencil.clone()).to_string();

                        format!("{}\n{}\n", symbol.format(stencil.clone()), children.indent(stencil.clone()))
                    })
                    .collect::<Vec<String>>()
                    .join("\n"),
            )
        }

        for &key in &keys {
            let target = self.records.get(&key).unwrap().module.unwrap();
            let mut module = self.resolver.get_symbol(target).unwrap().clone();
            let scope = replace(&mut module.scope, Scope::new(None));

            self.resolver.enter_scope(scope);

            let elements = self.records.get_mut(&key).unwrap().elements.as_mut().unwrap();

            for element in elements.iter_mut() {
                element.resolve(&mut self.resolver);
            }

            let active = self.resolver.active;
            self.resolver.exit();

            module.scope = self.resolver.scopes.remove(&active).unwrap();
            self.resolver.insert(module);
        }

        for &key in &keys {
            let target = self.records.get(&key).unwrap().module.unwrap();
            let mut module = self.resolver.get_symbol(target).unwrap().clone();
            let scope = replace(&mut module.scope, Scope::new(None));

            self.resolver.enter_scope(scope);

            let elements = self.records.get_mut(&key).unwrap().elements.as_mut().unwrap();

            for element in elements.iter_mut() {
                element.reify(&mut self.resolver);
            }

            let active = self.resolver.active;
            self.resolver.exit();

            module.scope = self.resolver.scopes.remove(&active).unwrap();
            self.resolver.insert(module);
        }

        self.errors
            .extend(self.resolver.errors.drain(..).map(CompileError::Resolve));

        let duration = Duration::from_nanos(self.timer.lap().unwrap());
        self.report_finish("resolving", duration, self.errors.len() - initial);
    }

    pub fn analyze(&mut self) {
        let initial = self.errors.len();

        self.report_start("analyzing");

        let mut keys: Vec<_> = self.records
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
                let record = self.records.get(&key).unwrap();
                (record.hash, record.dirty, record.elements.clone())
            };

            let file = self.cache("analyses", hash);

            if !dirty {
                if let Ok(data) = read(&file) {
                    let data: &'static [u8] = Box::leak(data.into_boxed_slice());
                    let mut cursor = 0;
                    let analyses = Option::<Vec<Analysis>>::decode(data, &mut cursor);
                    self.records.get_mut(&key).unwrap().analyses = analyses;
                    continue;
                }
            }

            let mut analyzer = Analyzer::new(elements.unwrap());
            analyzer.analyze(&mut self.resolver);

            if let Some(stencil) = self.get_stencil() {
                self.report_section(
                    "Analysis",
                    Color::Blue,
                    analyzer
                        .output
                        .format(stencil).to_string()
                );
            }

            self.errors.extend(
                analyzer
                    .errors
                    .iter()
                    .map(|error| CompileError::Analyze(error.clone())),
            );

            let mut buffer = Vec::new();
            Some(analyzer.output.clone()).encode(&mut buffer);
            _ = write(file, buffer);

            self.records.get_mut(&key).unwrap().analyses = Some(analyzer.output);
        }

        let duration = Duration::from_nanos(self.timer.lap().unwrap());
        self.report_finish("analyzing", duration, self.errors.len() - initial);
    }

    #[cfg(feature = "generator")]
    pub fn generate(&mut self) {
        let triple = TargetMachine::get_default_triple();
        let base = self.base();

        let initial = self.errors.len();

        self.report_start("generating");

        let mut keys: Vec<_> = self.records
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
            let record = self.records.get_mut(&key).unwrap();
            let location = record.location;
            let schema = Self::schema(&base, location);

            if !record.dirty && schema.to_path().map(|p| p.exists()).unwrap_or(false) {
                record.output = Some(schema);
                continue;
            }

            let stem = Str::from(location.stem().unwrap().to_string());

            if let Some(analysis) = record.analyses.clone() {
                let module = self.generator.context.create_module(stem.as_str().unwrap());

                module.set_triple(&triple);

                self.generator.modules.insert(stem, module);
                self.generator.current_module = stem;

                self.generator.generate(analysis);

                match schema.as_path() {
                    Ok(path) => {
                        let parent = path.parent().unwrap();
                        _ = create_dir_all(parent);

                        match crate::internal::platform::File::create(&path) {
                            Ok(mut file) => {
                                use crate::internal::platform::Write;
                                let string = self
                                    .generator
                                    .current_module()
                                    .print_to_string()
                                    .to_string();
                                if let Err(error) = file.write_all(string.as_bytes()) {
                                    let kind = TrackErrorKind::from_io(error, schema);
                                    let track = TrackError::new(kind, Span::void());
                                    self.errors.push(CompileError::Track(track));
                                    return;
                                }
                                record.output = Some(schema);
                            }
                            Err(error) => {
                                let kind = TrackErrorKind::from_io(error, schema);
                                let track = TrackError::new(kind, Span::void());
                                self.errors.push(CompileError::Track(track));
                            }
                        }
                    }
                    Err(error) => self.errors.push(CompileError::Track(error)),
                }
            }
        }

        let duration = Duration::from_nanos(self.timer.lap().unwrap());
        self.report_finish("generating", duration, self.errors.len() - initial);

        self.errors.extend(
            self.generator
                .errors
                .iter()
                .map(|error| CompileError::Generate(error.clone())),
        );
    }

    #[cfg(feature = "generator")]
    pub fn emit(&mut self) {
        self.report_start("emitting");

        let base = self.base();
        let mut direct = Vec::new();

        let mut keys: Vec<_> = self.records.keys().copied().collect();
        keys.sort();

        for &key in &keys {
            let record = self.records.get_mut(&key).unwrap();

            let target = match record.kind {
                InputKind::Source => record.output,
                InputKind::Schema | InputKind::C => Some(record.location),
                InputKind::Object => {
                    direct.push(record.location);
                    None
                }
            };

            if let Some(path) = target {
                let object = Self::object(&base, record.location, &record.kind, None);
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
            if let Some(object) = self.records.get(&key).unwrap().object {
                link.arg(object.to_string());
            }
        }

        for object in direct {
            link.arg(object.to_string());
        }

        let key = *keys.last().expect("missing");

        let record = self.records.get(&key).unwrap();
        let location = record.output.unwrap_or(record.location);

        let executable = Self::executable(&base, location, None);
        link.arg("-o").arg(executable.to_string());

        let status = link.status().expect("failed");

        if !status.success() {
            panic!("emitter failed: {}", status);
        }

        self.target = Some(executable);

        let duration = Duration::from_nanos(self.timer.lap().unwrap());
        self.report_external("emitting", duration);
    }

    #[cfg(feature = "generator")]
    pub fn run(&mut self) {
        self.report_start("running");

        let executable = self.target.unwrap();

        self.report_execute(&executable.to_string());

        let status = Command::new(executable.to_string())
            .status()
            .expect("failed");

        if !status.success() {
            panic!("{}", status);
        }

        let duration = Duration::from_nanos(self.timer.lap().unwrap());
        self.report_external("running", duration);
    }
}

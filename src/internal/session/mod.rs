// src/internal/session/mod.rs

mod core;
pub use core::*;

use {
    broccli::{xprintln, Color},
    crate::{
        analyzer::Analyzer,
        data::{memory::replace, Identity, Module, Str},
        format::Show,
        generator::Backend,
        internal::{
            cache::{Decode, Encode},
            hash::{Map, Set},
            platform::{create_dir_all, Command},
            timer::Duration,
        },
        parser::{Element, ElementKind, Parser, Symbol, SymbolKind, Visibility},
        resolver::{Resolvable, Scope},
        scanner::{Scanner, Token, TokenKind},
        tracker::{self, Location, Peekable, Span, TrackError},
    },
    inkwell::targets::TargetMachine,
    std::{
        fs,
        hash::{DefaultHasher, Hash, Hasher},
    },
};

impl<'session> Session<'session> {
    pub fn compile(&mut self) {
        self.load();

        'pipeline: {
            self.prepare();

            self.scan();
            if !self.errors.is_empty() {
                break 'pipeline;
            }

            self.parse();
            if !self.errors.is_empty() {
                break 'pipeline;
            }

            self.populate();
            self.plan();

            self.resolve();
            if !self.errors.is_empty() {
                break 'pipeline;
            }

            self.analyze();
            if !self.errors.is_empty() {
                break 'pipeline;
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

            self.save();
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

    pub fn load(&mut self) {
        let initial = self.errors.len();
        self.report_start("loading");

        let base = self.base();
        let cache = base.join("build").join(".cache").join("hashes");

        if create_dir_all(&cache).is_ok() {
            if let Ok(entries) = fs::read_dir(cache) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Ok(data) = fs::read(&path) {
                            if data.len() >= 8 {
                                let mut cursor = 0;
                                let hash = u64::decode(&data, &mut cursor);
                                let stem = path.file_stem().unwrap().to_string_lossy().to_string();
                                let location = Location::Entry(Str::from(stem));
                                self.cache.insert(location, hash);
                            }
                        }
                    }
                }
            }
        }

        let duration = Duration::from_nanos(self.timer.lap().unwrap());
        self.report_finish("loading", duration, self.errors.len() - initial);
    }

    pub fn save(&mut self) {
        let initial = self.errors.len();
        self.report_start("saving");

        let base = self.base();
        let cache = base.join("build").join(".cache").join("hashes");
        let _ = create_dir_all(&cache);

        for (location, hash) in &self.cache {
            let file = cache.join(location.to_string().replace('/', "_").replace('\\', "_"));
            let mut buffer = Vec::new();
            hash.encode(&mut buffer);
            let _ = fs::write(file, buffer);
        }

        let duration = Duration::from_nanos(self.timer.lap().unwrap());
        self.report_finish("saving", duration, self.errors.len() - initial);
    }

    pub fn prepare(&mut self) {
        let mut keys: Vec<_> = self.records.keys().copied().collect();
        keys.sort();

        for key in keys {
            let record = self.records.get_mut(&key).unwrap();

            if record.kind == InputKind::Source {
                let location = record.location;
                let path = location.to_string();

                if let Ok(content) = fs::read_to_string(&path) {
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

                let symbol = Symbol::new(
                    SymbolKind::Module(Module::new(head)),
                    span,
                    Visibility::Public,
                );

                record.module = Some(symbol.identity);
                Some(symbol)
            })
            .collect();

        for module in modules {
            self.resolver.insert(module);
        }
    }

    pub fn plan(&mut self) {
        let mut sources: Vec<_> = self
            .records
            .iter()
            .filter_map(|(&identity, record)| {
                if record.kind == InputKind::Source {
                    Some(identity)
                } else {
                    None
                }
            })
            .collect();

        sources.sort();

        let mut graph = Map::new();
        let mut degrees = Map::new();

        for &identity in &sources {
            graph.insert(identity, Vec::new());
            degrees.insert(identity, 0);
        }

        for &identity in &sources {
            let dependencies = self.dependencies(identity);

            for symbol in dependencies {
                let resolved = self.records.iter().find_map(|(&key, record)| {
                    if record.module == Some(symbol) {
                        Some(key)
                    } else {
                        None
                    }
                });

                if let Some(dependency) = resolved {
                    if graph.contains_key(&dependency) {
                        graph.get_mut(&dependency).unwrap().push(identity);
                        *degrees.get_mut(&identity).unwrap() += 1;
                    }
                }
            }
        }

        let mut queue: Vec<_> = degrees
            .iter()
            .filter_map(|(&identity, &degree)| if degree == 0 { Some(identity) } else { None })
            .collect();

        queue.sort();

        let mut order = Vec::new();

        while !queue.is_empty() {
            let identity = queue.remove(0);
            order.push(identity);

            let is_dirty = self.records.get(&identity).unwrap().dirty;

            if let Some(neighbors) = graph.get(&identity) {
                for &neighbor in neighbors {
                    if is_dirty {
                        self.records.get_mut(&neighbor).unwrap().dirty = true;
                    }

                    let degree = degrees.get_mut(&neighbor).unwrap();
                    *degree -= 1;

                    if *degree == 0 {
                        queue.push(neighbor);
                    }
                }
                queue.sort();
            }
        }

        if order.len() != sources.len() {
            for &identity in &sources {
                if !order.contains(&identity) {
                    order.push(identity);
                }
            }
        }

        self.order = order;

        let sequence: Vec<String> = self
            .order
            .iter()
            .map(|key| {
                self.records
                    .get(key)
                    .unwrap()
                    .location
                    .stem()
                    .unwrap()
                    .to_string()
            })
            .collect();

        if self.is_active() && !sequence.is_empty() {
            xprintln!(
                "{}{} {}" => Color::White,
                "Order" => Color::Magenta,
                ":" => Color::White,
                sequence.join(" -> ") => Color::White
            );
            xprintln!();
        }
    }

    fn dependencies(&mut self, identity: Identity) -> Set<Identity> {
        let elements = &self
            .records
            .get(&identity)
            .unwrap()
            .elements
            .as_ref()
            .unwrap();

        for element in elements.iter() {
            element.depending(&mut self.resolver);
        }

        let dependencies = self.resolver.dependencies.clone();
        self.resolver.dependencies.clear();

        dependencies
    }

    pub fn scan(&mut self) {
        let initial = self.errors.len();
        self.report_start("scanning");

        let mut keys: Vec<_> = self.records.keys().copied().collect();
        keys.sort();

        for key in keys {
            let record = self.records.get(&key).unwrap();
            let is_source = record.kind == InputKind::Source;

            if is_source {
                let location = record.location;
                let mut scanner = Scanner::new(location);

                scanner.prepare();
                scanner.scan();

                let verbosity = self.get_verbosity().into();
                self.report_section(
                    "Tokens",
                    Color::Cyan,
                    scanner
                        .output
                        .iter()
                        .map(|token| format!("{}", token.format(verbosity)))
                        .collect::<Vec<String>>()
                        .join(", "),
                );

                self.errors.extend(
                    scanner
                        .errors
                        .iter()
                        .map(|error| CompileError::Scan(error.clone())),
                );

                self.records.get_mut(&key).unwrap().tokens = Some(scanner.output);
            }
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
            let record = self.records.get(&key).unwrap();
            let is_source = record.kind == InputKind::Source;

            if is_source {
                let location = record.location;
                let tokens = record.tokens.as_ref().unwrap().clone();
                let mut parser = Parser::new(location);

                parser.set_input(tokens);
                parser.parse();

                let verbosity = self.get_verbosity().into();
                self.report_section(
                    "Elements",
                    Color::Cyan,
                    parser
                        .output
                        .iter()
                        .map(|element| format!("{}", element.format(verbosity)))
                        .collect::<Vec<String>>()
                        .join("\n"),
                );

                self.errors.extend(
                    parser
                        .errors
                        .iter()
                        .map(|error| CompileError::Parse(error.clone())),
                );

                self.records.get_mut(&key).unwrap().elements = Some(parser.output);
            }
        }

        let duration = Duration::from_nanos(self.timer.lap().unwrap());
        self.report_finish("parsing", duration, self.errors.len() - initial);
    }

    pub fn resolve(&mut self) {
        let initial = self.errors.len();
        self.report_start("resolving");

        for &key in &self.order {
            let target = self.records.get(&key).unwrap().module.unwrap();
            let mut module = self.resolver.get_symbol(target).unwrap().clone();
            let scope = replace(&mut module.scope, Scope::new(None));

            self.resolver.enter_scope(scope);

            let elements = &mut self
                .records
                .get_mut(&key)
                .unwrap()
                .elements
                .as_mut()
                .unwrap();

            for element in elements.iter_mut() {
                element.declare(&mut self.resolver);
            }

            let active = self.resolver.active;
            self.resolver.exit();

            module.scope = self.resolver.scopes.remove(&active).unwrap();
            self.resolver.insert(module);
        }

        let verbosity = self.get_verbosity().into();
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
                        .map(|symbol| self.resolver.get_symbol(*symbol))
                        .collect::<Vec<_>>();
                    format!("{}\n{}", symbol.format(verbosity), children.format(verbosity))
                })
                .collect::<Vec<String>>()
                .join("\n"),
        );

        for &key in &self.order {
            let target = self.records.get(&key).unwrap().module.unwrap();
            let mut module = self.resolver.get_symbol(target).unwrap().clone();
            let scope = replace(&mut module.scope, Scope::new(None));

            self.resolver.enter_scope(scope);

            let elements = &mut self
                .records
                .get_mut(&key)
                .unwrap()
                .elements
                .as_mut()
                .unwrap();

            for element in elements.iter_mut() {
                element.resolve(&mut self.resolver);
            }

            let active = self.resolver.active;
            self.resolver.exit();

            module.scope = self.resolver.scopes.remove(&active).unwrap();
            self.resolver.insert(module);
        }

        for &key in &self.order {
            let target = self.records.get(&key).unwrap().module.unwrap();
            let mut module = self.resolver.get_symbol(target).unwrap().clone();
            let scope = replace(&mut module.scope, Scope::new(None));

            self.resolver.enter_scope(scope);

            let elements = &mut self
                .records
                .get_mut(&key)
                .unwrap()
                .elements
                .as_mut()
                .unwrap();

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

        for &key in &self.order {
            let elements = self
                .records
                .get(&key)
                .unwrap()
                .elements
                .as_ref()
                .unwrap()
                .clone();

            let mut analyzer = Analyzer::new(elements);
            analyzer.analyze(&mut self.resolver);

            let verbosity = self.get_verbosity().into();
            self.report_section(
                "Analysis",
                Color::Blue,
                analyzer
                    .output
                    .iter()
                    .map(|item| format!("{}", item.format(verbosity)))
                    .collect::<Vec<String>>()
                    .join("\n"),
            );

            self.errors.extend(
                analyzer
                    .errors
                    .iter()
                    .map(|error| CompileError::Analyze(error.clone())),
            );

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

        for &key in &self.order {
            let record = self.records.get_mut(&key).unwrap();
            let location = record.location;
            let schema = Self::schema(&base, location);

            if !record.dirty {
                record.output = Some(schema);
                continue;
            }

            let stem = Str::from(location.stem().unwrap().to_string());
            let analysis = record.analyses.as_ref().unwrap().clone();
            let module = self.generator.context.create_module(stem.as_str().unwrap());

            module.set_triple(&triple);

            self.generator.modules.insert(stem, module);
            self.generator.current_module = stem;

            self.generator.generate(analysis);

            match schema.as_path() {
                Ok(path) => {
                    let parent = path.parent().unwrap();
                    let _ = create_dir_all(parent);

                    match crate::internal::platform::File::create(&path) {
                        Ok(mut file) => {
                            use crate::internal::platform::Write;
                            let string = self.generator.current_module().print_to_string().to_string();
                            if let Err(error) = file.write_all(string.as_bytes()) {
                                let kind = tracker::error::ErrorKind::from_io(error, schema);
                                let track = TrackError::new(kind, Span::void());
                                self.errors.push(CompileError::Track(track));
                                return;
                            }
                            record.output = Some(schema);
                        }
                        Err(error) => {
                            let kind = tracker::error::ErrorKind::from_io(error, schema);
                            let track = TrackError::new(kind, Span::void());
                            self.errors.push(CompileError::Track(track));
                        }
                    }
                }
                Err(error) => self.errors.push(CompileError::Track(error)),
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
                let _ = create_dir_all(&parent);

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

        let key = self.order.last().copied().unwrap_or_else(|| *keys.last().expect("missing"));

        let record = self.records.get(&key).unwrap();
        let location = record.output.unwrap_or(record.location);

        let executable = Self::executable(&base, location, None);
        link.arg("-o").arg(executable.to_string());

        let status = link.status().expect("failed");

        if !status.success() {
            panic!("failed");
        }

        self.target = Some(executable);

        let duration = Duration::from_nanos(self.timer.lap().unwrap());
        self.report_external("emitting", duration);
    }

    pub fn run(&mut self) {
        self.report_start("running");

        let executable = self.target.unwrap();

        if self.is_active() {
            xprintln!(
                "Executing {}." => Color::Blue,
                format!("`{}`", executable) => Color::White
            );
            xprintln!();
        }

        let status = Command::new(executable.to_string()).status().expect("failed");

        if !status.success() {
            panic!("{}", status);
        }

        let duration = Duration::from_nanos(self.timer.lap().unwrap());
        self.report_external("running", duration);
    }
}

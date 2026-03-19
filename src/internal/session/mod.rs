mod registry;

use {
    crate::{
        analyzer::{AnalyzeError, Analyzer},
        data::{memory::replace, *},
        generator::{Backend, GenerateError, Generator},
        initializer::{InitializeError, Initializer},
        internal::{
            hash::{Set, Map},
            platform::{Command, File, PathBuf, Write, create_dir_all},
            timer::{DefaultTimer, Duration},
        },
        parser::{Element, ElementKind, ParseError, Parser, Symbol, SymbolKind, Visibility},
        reporter::Reporter,
        resolver::{Resolvable, ResolveError, Resolver, Scope},
        scanner::{ScanError, Scanner, Token, TokenKind},
        tracker::{self, Location, Peekable, Span, TrackError},
    },
    inkwell::context::{Context, ContextRef},
};

const FORMATTER: &[u8] = include_bytes!("/home/ali/Projects/axo/runtime/formatter.o");
const RUNTIME: &[u8] = include_bytes!("/home/ali/Projects/axo/runtime/runtime.o");

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InputKind {
    Source,
    Schema,
    Object,
}

impl InputKind {
    pub fn from_path(path: &str) -> Option<Self> {
        if path.ends_with(".axo") {
            Some(InputKind::Source)
        } else if path.ends_with(".ll") {
            Some(InputKind::Schema)
        } else if path.ends_with(".o") {
            Some(InputKind::Object)
        } else {
            None
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            InputKind::Source => "axo",
            InputKind::Schema => "ll",
            InputKind::Object => "o",
        }
    }
}

pub enum CompileError<'error> {
    Initialize(InitializeError<'error>),
    Scan(ScanError<'error>),
    Parse(ParseError<'error>),
    Resolve(ResolveError<'error>),
    Analyze(AnalyzeError<'error>),
    Generate(GenerateError<'error>),
    Track(TrackError<'error>),
    InvalidInput(Location<'error>),
}

pub struct Session<'session> {
    pub timer: DefaultTimer,
    pub reporter: Reporter,
    pub order: Vec<Identity>,
    pub inputs: Map<Identity, (InputKind, Location<'session>)>,
    pub modules: Map<Identity, Identity>,
    pub initializer: Initializer<'session>,
    pub scanners: Map<Identity, Scanner<'session>>,
    pub parsers: Map<Identity, Parser<'session>>,
    pub resolver: Resolver<'session>,
    pub analyzers: Map<Identity, Analyzer<'session>>,
    pub generator: Generator<'session>,
    pub context: Context,
    pub errors: Vec<CompileError<'session>>,
    pub outputs: Map<Identity, Location<'session>>,
}

impl<'session> Session<'session> {
    pub fn start() -> Self {
        let mut timer = DefaultTimer::new_default();
        let _ = timer.start();

        let mut initializer = Initializer::new(Location::Flag);
        let mut resolver = Resolver::new();

        let verbosity = Resolver::verbosity(&mut resolver);
        let reporter = Reporter::new(verbosity);

        reporter.start("initializing");

        let mut inputs = Map::new();
        let mut errors = Vec::new();

        initializer.initialize().iter().for_each(|target| {
            let kind = InputKind::from_path(&*target.to_string());

            if let Some(kind) = kind {
                inputs.insert(inputs.len(), (kind, target.clone()));
            } else {
                errors.push(CompileError::InvalidInput(target.clone()));
            }
        });

        errors.extend(
            initializer
                .errors
                .iter()
                .map(|error| CompileError::Initialize(error.clone()))
                .collect::<Vec<_>>(),
        );

        let configuration = Symbol::new(
            SymbolKind::Module(Module::new(Box::from(Element::new(
                ElementKind::Literal(Token::new(
                    TokenKind::Identifier(Str::from("config")),
                    Span::void(),
                )),
                Span::void(),
            )))),
            Span::void(),
            Visibility::Public,
        )
            .with_members(initializer.output.clone());

        resolver.insert(configuration);

        let duration = Duration::from_nanos(timer.lap().unwrap());
        let verbosity = Resolver::verbosity(&mut resolver);
        let reporter = Reporter::new(verbosity);

        let context = Context::create();
        let context_ref = unsafe { ContextRef::new(context.raw()) };
        let generator = Generator::new(context_ref);

        reporter.finish("initializing", duration);

        Session {
            timer,
            reporter,
            inputs,
            order: Vec::new(),
            modules: Map::new(),
            initializer,
            scanners: Map::new(),
            parsers: Map::new(),
            resolver,
            analyzers: Map::new(),
            generator,
            context,
            errors,
            outputs: Map::new(),
        }
    }

    pub fn compile(&mut self) {
        'pipeline: {
            self.scan();

            self.parse();
            if !self.errors.is_empty() {
                break 'pipeline;
            }

            self.populate();
            if !self.errors.is_empty() {
                break 'pipeline;
            }

            self.plan();

            self.resolve();
            if !self.errors.is_empty() {
                break 'pipeline;
            }

            self.analyze();
            if !self.errors.is_empty() {
                break 'pipeline;
            }

            self.generate();
            if !self.errors.is_empty() {
                break 'pipeline;
            }

            self.emit();
        }

        let duration = Duration::from_nanos(self.timer.stop().unwrap());
        self.reporter.finish("compilation", duration);

        for error in &self.errors {
            match error {
                CompileError::Initialize(error) => self.reporter.error(&error),
                CompileError::Scan(error) => self.reporter.error(&error),
                CompileError::Parse(error) => self.reporter.error(&error),
                CompileError::Resolve(error) => self.reporter.error(&error),
                CompileError::Analyze(error) => self.reporter.error(&error),
                CompileError::Generate(error) => self.reporter.error(&error),
                CompileError::Track(error) => self.reporter.error(&error),
                CompileError::InvalidInput(_) => {}
            }
        }
    }

    pub fn populate(&mut self) {
        let modules: Vec<_> = self
            .inputs
            .iter()
            .map(|(identity, (_, location))| {
                let stem = Str::from(location.stem().unwrap().to_string());
                let span = Span::file(Str::from(location.to_string())).unwrap();

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

                self.modules.insert(*identity, symbol.identity);
                symbol
            })
            .collect();

        for module in modules {
            self.resolver.insert(module);
        }
    }

    pub fn plan(&mut self) {
        let mut sources: Vec<_> = self
            .inputs
            .iter()
            .filter_map(|(&identity, (kind, _))| if *kind == InputKind::Source { Some(identity) } else { None })
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
                let resolved = self.modules.iter().find_map(|(&key, &value)| {
                    if value == symbol {
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

            if let Some(neighbors) = graph.get(&identity) {
                for &neighbor in neighbors {
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
    }

    fn dependencies(&mut self, identity: Identity) -> Set<Identity> {
        let elements = &self.parsers.get(&identity).unwrap().output;

        for element in elements.iter() {
            element.depending(&mut self.resolver);
        }

        let dependencies = self.resolver.dependencies.clone();

        self.resolver.dependencies.clear();

        dependencies
    }

    pub fn scan(&mut self) {
        self.reporter.start("scanning");

        let mut identities: Vec<_> = self.inputs.keys().copied().collect();
        identities.sort();

        for identity in identities {
            let (kind, location) = self.inputs.get(&identity).unwrap();

            if *kind == InputKind::Source {
                let mut scanner = Scanner::new(*location);

                scanner.prepare();
                scanner.scan();

                self.reporter.tokens(&scanner.output);

                self.errors.extend(
                    scanner
                        .errors
                        .iter()
                        .map(|error| CompileError::Scan(error.clone())),
                );

                self.scanners.insert(identity, scanner);
            }
        }

        let duration = Duration::from_nanos(self.timer.lap().unwrap());
        self.reporter.finish("scanning", duration);
    }

    pub fn parse(&mut self) {
        self.reporter.start("parsing");

        let mut identities: Vec<_> = self.inputs.keys().copied().collect();
        identities.sort();

        for identity in identities {
            let (kind, location) = self.inputs.get(&identity).unwrap();

            if *kind == InputKind::Source {
                let mut parser = Parser::new(*location);
                let tokens = self.scanners.get(&identity).unwrap().output.clone();

                parser.set_input(tokens);
                parser.parse();

                self.reporter.elements(&parser.output);

                self.errors.extend(
                    parser
                        .errors
                        .iter()
                        .map(|error| CompileError::Parse(error.clone())),
                );

                self.parsers.insert(identity, parser);
            }
        }

        let duration = Duration::from_nanos(self.timer.lap().unwrap());
        self.reporter.finish("parsing", duration);
    }

    pub fn resolve(&mut self) {
        self.reporter.start("resolving");

        for &identity in &self.order {
            let module_id = *self.modules.get(&identity).unwrap();
            let mut module = self.resolver.get_symbol(module_id).unwrap().clone();
            let module_scope = replace(&mut module.scope, Scope::new(None));

            self.resolver.enter_scope(module_scope);

            let elements = &mut self.parsers.get_mut(&identity).unwrap().output;

            for element in elements.iter_mut() {
                element.declare(&mut self.resolver);
            }

            let active = self.resolver.active;
            self.resolver.exit();

            module.scope = self.resolver.scopes.remove(&active).unwrap();
            self.resolver.insert(module);
        }

        let scope = self.resolver.scopes.get(&self.resolver.active).unwrap().clone();
        self.reporter.symbols(&scope.collect(&self.resolver.scopes, &self.resolver.registry));

        for &identity in &self.order {
            let module_id = *self.modules.get(&identity).unwrap();
            let mut module = self.resolver.get_symbol(module_id).unwrap().clone();
            let module_scope = replace(&mut module.scope, Scope::new(None));

            self.resolver.enter_scope(module_scope);

            let elements = &mut self.parsers.get_mut(&identity).unwrap().output;

            for element in elements.iter_mut() {
                element.resolve(&mut self.resolver);
            }

            let active = self.resolver.active;
            self.resolver.exit();

            module.scope = self.resolver.scopes.remove(&active).unwrap();
            self.resolver.insert(module);
        }

        for &identity in &self.order {
            let module_id = *self.modules.get(&identity).unwrap();
            let mut module = self.resolver.get_symbol(module_id).unwrap().clone();
            let module_scope = replace(&mut module.scope, Scope::new(None));

            self.resolver.enter_scope(module_scope);

            let elements = &mut self.parsers.get_mut(&identity).unwrap().output;

            for element in elements.iter_mut() {
                element.reify(&mut self.resolver);
            }

            let active = self.resolver.active;
            self.resolver.exit();

            module.scope = self.resolver.scopes.remove(&active).unwrap();
            self.resolver.insert(module);
        }

        self.errors.extend(self.resolver.errors.drain(..).map(CompileError::Resolve));

        let duration = Duration::from_nanos(self.timer.lap().unwrap());
        self.reporter.finish("resolving", duration);
    }

    pub fn analyze(&mut self) {
        for &identity in &self.order {
            self.reporter.start("analyzing");

            let elements = self.parsers.get(&identity).unwrap().output.clone();
            let mut analyzer = Analyzer::new(elements);
            analyzer.analyze(&mut self.resolver);

            self.reporter.analysis(&*analyzer.output);

            self.errors.extend(
                analyzer
                    .errors
                    .iter()
                    .map(|error| CompileError::Analyze(error.clone())),
            );

            self.analyzers.insert(identity, analyzer);

            let duration = Duration::from_nanos(self.timer.lap().unwrap());
            self.reporter.finish("analyzing", duration);
        }
    }

    pub fn generate(&mut self) {
        let triple = inkwell::targets::TargetMachine::get_default_triple();
        let base = self.base();

        for &identity in &self.order {
            let (_, location) = self.inputs.get(&identity).unwrap();
            let stem = Str::from(location.stem().unwrap().to_string());
            let analysis = self.analyzers.get(&identity).unwrap().output.clone();
            let module = self.generator.context.create_module(stem.as_str().unwrap());

            module.set_triple(&triple);

            self.generator.modules.insert(stem, module);
            self.generator.current_module = stem.clone();

            let schema = Self::schema(&base, *location, Resolver::schema(&mut self.resolver, identity));

            self.reporter.start("generating");
            self.generator.generate(analysis);

            match schema.as_path() {
                Ok(path) => {
                    let dir = path.parent().unwrap();
                    let _ = create_dir_all(dir);

                    match File::create(&path) {
                        Ok(mut file) => {
                            if let Err(error) = file.write_all(
                                self.generator.current_module().print_to_string().to_string().as_bytes(),
                            ) {
                                self.errors.push(CompileError::Track(TrackError::new(
                                    tracker::error::ErrorKind::from_io(error, schema),
                                    Span::void(),
                                )));
                                return;
                            }
                            self.outputs.insert(identity, schema);
                        }
                        Err(error) => self.errors.push(CompileError::Track(TrackError::new(
                            tracker::error::ErrorKind::from_io(error, schema),
                            Span::void(),
                        ))),
                    }
                }
                Err(error) => self.errors.push(CompileError::Track(error)),
            }

            let duration = Duration::from_nanos(self.timer.lap().unwrap());
            self.reporter.finish("generating", duration);
        }

        self.errors.extend(
            self.generator
                .errors
                .iter()
                .map(|error| CompileError::Generate(error.clone())),
        );
    }

    pub fn emit(&mut self) {
        self.reporter.start("emitting");

        let base = self.base();
        let mut objects = Map::new();
        let mut direct = Vec::new();

        for (&identity, &(ref kind, ref location)) in &self.inputs {
            let location = match kind {
                InputKind::Source => {
                    if let Some(output) = self.outputs.get(&identity) {
                        Some(*output)
                    } else {
                        None
                    }
                }
                InputKind::Schema => Some(*location),
                InputKind::Object => {
                    direct.push(*location);
                    None
                }
            };

            if let Some(target) = location {
                let object = Self::object(&base, target, None);
                let directory = object.to_path().unwrap().parent().unwrap().to_path_buf();
                let _ = create_dir_all(&directory);

                let mut command = Command::new("clang");

                command
                    .arg("-c")
                    .arg(target.to_string())
                    .arg("-o")
                    .arg(object.to_string());

                let status = command.status().expect("failed");

                if status.success() {
                    objects.insert(identity, object);
                } else {
                    panic!("failed {}", target);
                }
            }
        }

        let object_directory = base.join("build").join("objects");
        let _ = create_dir_all(&object_directory);

        let formatter = object_directory.join("formatter.o");
        let runtime = object_directory.join("runtime.o");

        File::create(&formatter).unwrap().write_all(FORMATTER).unwrap();
        File::create(&runtime).unwrap().write_all(RUNTIME).unwrap();

        let mut link = Command::new("clang");

        for object in objects.values() {
            link.arg(object.to_string());
        }

        for object in direct {
            link.arg(object.to_string());
        }

        link.arg(formatter);
        link.arg(runtime);

        let mut identities: Vec<_> = self.inputs.keys().copied().collect();
        identities.sort();

        let identity = self.order.first().copied().unwrap_or_else(|| {
            *identities.first().expect("missing")
        });

        let location = self
            .outputs
            .get(&identity)
            .copied()
            .unwrap_or_else(|| self.inputs.get(&identity).unwrap().1);

        let executable = Self::executable(&base, location, None);
        link.arg("-o").arg(executable.to_string());

        let program = link.get_program().to_string_lossy();
        let arguments: Vec<String> = link
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();

        let command = if arguments.is_empty() {
            program.into_owned()
        } else {
            format!("{} {}", program, arguments.join(" "))
        };

        self.reporter.run(format!("{}", command));

        let status = link.status().expect("failed");

        if !status.success() {
            panic!("failed");
        }

        let duration = Duration::from_nanos(self.timer.lap().unwrap());
        self.reporter.finish("emitting", duration);
        self.reporter.run(format!("{}", executable));

        Command::new(executable.to_string()).status().expect("failed");
    }

    fn base(&self) -> PathBuf {
        let paths: Vec<_> = self.inputs.values().filter_map(|(_, loc)| loc.to_path().ok()).collect();

        if paths.is_empty() {
            return PathBuf::from(".");
        }

        let mut base = paths[0].parent().unwrap().to_path_buf();

        for path in &paths[1..] {
            let directory = path.parent().unwrap();
            let mut current = PathBuf::new();
            let mut base_iterator = base.components();
            let mut directory_iterator = directory.components();

            while let (Some(b), Some(d)) = (base_iterator.next(), directory_iterator.next()) {
                if b == d {
                    current.push(b);
                } else {
                    break;
                }
            }

            base = current;
        }

        base
    }

    fn schema(base: &PathBuf, location: Location<'session>, configuration: Option<Str<'session>>) -> Location<'session> {
        let target = if let Some(schema) = configuration {
            PathBuf::from(schema.to_string())
        } else {
            base.join("build").join("schema").join(location.stem().unwrap()).with_extension("ll")
        };

        Location::Entry(Str::from(target))
    }

    fn object(base: &PathBuf, location: Location<'session>, configuration: Option<Str<'session>>) -> Location<'session> {
        let target = if let Some(schema) = configuration {
            PathBuf::from(schema.to_string())
        } else {
            base.join("build").join("objects").join(location.stem().unwrap()).with_extension("o")
        };

        Location::Entry(Str::from(target))
    }

    fn executable(base: &PathBuf, location: Location<'session>, configuration: Option<Str<'session>>) -> Location<'session> {
        let target = if let Some(schema) = configuration {
            PathBuf::from(schema.to_string())
        } else {
            base.join("build").join(location.stem().unwrap()).with_extension("")
        };

        Location::Entry(Str::from(target))
    }
}

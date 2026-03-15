mod registry;

use {
    crate::{
        data::*,
        initializer::{
            Initializer,
            InitializeError,
        },
        internal::{
            platform::{
                PathBuf,
                File,
                Command,
                Write,
            },
            hash::{
                Map,
            },
            timer::{DefaultTimer, Duration},
        },
        parser::{
            Element, ElementKind,
            Symbol, SymbolKind,
            ParseError,
            Parser, Visibility,
        },
        reporter::Reporter,
        resolver::{
            Resolver,
            Resolvable,
            ResolveError,
        },
        analyzer::Analyzer,
        scanner::{
            Scanner,
            Token, TokenKind,
            ScanError,
        },
        tracker::{
            self,
            Location, Span,
            TrackError,
        },
    },
    inkwell::{
        context::{Context, ContextRef},
    },
};

use {
    crate::{
        analyzer::AnalyzeError,
        generator::{Generator, GenerateError, Backend},
        tracker::Peekable,
    }
};

pub enum CompileError<'error> {
    Initialize(InitializeError<'error>),
    Scan(ScanError<'error>),
    Parse(ParseError<'error>),
    Resolve(ResolveError<'error>),
    Analyze(AnalyzeError<'error>),
    Generate(GenerateError<'error>),
    Track(TrackError<'error>),
}

pub struct Session<'session> {
    pub timer: DefaultTimer,
    pub reporter: Reporter,
    pub order: Vec<Identity>,
    pub inputs: Map<Identity, Location<'session>>,
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
        let logger = Reporter::new(verbosity);

        logger.start("initializing");

        let mut inputs = Map::new();

        initializer.initialize().iter().for_each(|target| {
            inputs.insert(inputs.len(), target.clone());
        });

        let errors =
            initializer
                .errors
                .iter()
                .map(|error| {
                    CompileError::Initialize(error.clone())
                }).collect::<Vec<_>>();

        let configuration = Symbol::new(
            SymbolKind::Module(
                Module::new(
                    Box::from(
                        Element::new(
                            ElementKind::Literal(
                                Token::new(
                                    TokenKind::Identifier(
                                        Str::from("config")
                                    ),
                                    Span::void(),
                                ),
                            ),
                            Span::void(),
                        )
                    )
                )
            ),
            Span::void(),
            Visibility::Public,
        ).with_members(initializer.output.clone());

        resolver.insert(configuration);

        let duration = Duration::from_nanos(timer.lap().unwrap());

        let verbosity = Resolver::verbosity(&mut resolver);

        let reporter = Reporter::new(verbosity);

        let context = Context::create();
        let context_ref = unsafe {
            ContextRef::new(context.raw())
        };

        let generator = Generator::new(context_ref);

        logger.finish("initializing", duration);

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
        'pipeline : {
            self.scan();

            self.parse();
            if !self.errors.is_empty() { break 'pipeline; }

            self.plan();

            self.register();
            if !self.errors.is_empty() { break 'pipeline; }

            self.resolve();
            if !self.errors.is_empty() { break 'pipeline; }

            self.analyze();
            if !self.errors.is_empty() { break 'pipeline; }

            self.generate();
            if !self.errors.is_empty() { break 'pipeline; }

            let duration = Duration::from_nanos(self.timer.lap().unwrap());

            self.reporter.finish("compilation", duration);

            self.emit();
        }

        for error in &self.errors {
            match error {
                CompileError::Initialize(error) => self.reporter.error(&error),
                CompileError::Scan(error) => self.reporter.error(&error),
                CompileError::Parse(error) => self.reporter.error(&error),
                CompileError::Resolve(error) => self.reporter.error(&error),
                CompileError::Analyze(error) => self.reporter.error(&error),
                CompileError::Generate(error) => self.reporter.error(&error),
                CompileError::Track(error) => self.reporter.error(&error),
            }
        }
    }

    /// Computes and establishes compilation priority mapping using Topological Sort.
    pub fn plan(&mut self) {
        let mut identities: Vec<_> = self.inputs.keys().copied().collect();

        identities.sort();

        let mut graph = Map::new();
        let mut degreed = Map::new();

        for &id in &identities {
            graph.insert(id, Vec::new());
            degreed.insert(id, 0);
        }

        for &identity in &identities {
            let dependencies = self.dependencies(identity);
            for dependency in dependencies {
                if graph.contains_key(&dependency) {
                    graph.get_mut(&dependency).unwrap().push(identity);
                    *degreed.get_mut(&identity).unwrap() += 1;
                }
            }
        }

        let mut queue: Vec<_> = degreed
            .iter()
            .filter_map(|(&id, &deg)| if deg == 0 { Some(id) } else { None })
            .collect();

        queue.sort();

        let mut sorted = Vec::new();

        while !queue.is_empty() {
            let identity = queue.remove(0);

            sorted.push(identity);

            if let Some(neighbors) = graph.get(&identity) {
                for &next in neighbors {
                    let degree = degreed.get_mut(&next).unwrap();

                    *degree -= 1;

                    if *degree == 0 {
                        queue.push(next);
                    }
                }
                queue.sort();
            }
        }

        if sorted.len() != identities.len() {
            for &identity in &identities {
                if !sorted.contains(&identity) {
                    sorted.push(identity);
                }
            }
        }

        self.order = sorted;
    }

    fn dependencies(&self, _identity: Identity) -> Vec<Identity> {
        // TODO: Access AST from `self.parsers.get(&_identity).unwrap().output`
        // to detect cross-module usages (e.g. `import` or `use` nodes)
        // and map them back to their respective module Identity.
        Vec::new()
    }

    pub fn scan(&mut self) {
        self.reporter.start("scanning");

        let mut identities: Vec<_> = self.inputs.keys().copied().collect();
        identities.sort();

        for identity in identities {
            let location = self.inputs.get(&identity).unwrap();
            let mut scanner = Scanner::new(*location);

            scanner.prepare();
            scanner.scan();

            self.reporter.tokens(&scanner.output);

            self.errors.extend(
                scanner
                    .errors
                    .iter()
                    .map(|error| {
                        CompileError::Scan(error.clone())
                    })
            );

            self.scanners.insert(identity, scanner);
        }

        let duration = Duration::from_nanos(self.timer.lap().unwrap());

        self
            .reporter
            .finish("scanning", duration);
    }

    pub fn parse(&mut self) {
        self.reporter.start("parsing");

        let mut identities: Vec<_> = self.inputs.keys().copied().collect();
        identities.sort();

        for identity in identities {
            let location = self.inputs.get(&identity).unwrap();
            let mut parser = Parser::new(*location);

            let tokens = self.scanners.get(&identity).unwrap().output.clone();

            parser.set_input(tokens);
            parser.parse();

            self.reporter.elements(&parser.output);

            self.errors.extend(
                parser
                    .errors
                    .iter()
                    .map(|error| {
                        CompileError::Parse(error.clone())
                    })
            );

            self.parsers.insert(identity, parser);
        }

        let duration = Duration::from_nanos(self.timer.lap().unwrap());

        self.reporter.finish("parsing", duration);
    }

    pub fn register(&mut self) {
        let modules: Vec<_> = self.order
            .iter()
            .map(|&identity| {
                let location = self.inputs.get(&identity).unwrap();
                let stem = Str::from(location.stem().unwrap().to_string());
                let span = Span::file(Str::from(location.to_string())).unwrap();

                let head = Element::new(
                    ElementKind::Literal(
                        Token::new(
                            TokenKind::Identifier(stem),
                            span,
                        )
                    ),
                    span,
                ).into();

                let symbol = Symbol::new(
                    SymbolKind::Module(Module::new(head)),
                    span,
                    Visibility::Public,
                );

                self.modules.insert(identity, symbol.identity);

                symbol
            })
            .collect();

        for module in modules {
            self.resolver.insert(module);
        }
    }

    pub fn resolve(&mut self) {
        self.reporter.start("resolving");

        for &identity in &self.order {
            let module_identity = *self.modules.get(&identity).unwrap();
            let mut module = self.resolver.scope.find(module_identity).unwrap().clone();

            self.resolver.enter_scope(module.scope.clone());

            let elements = &mut self.parsers.get_mut(&identity).unwrap().output;

            for element in elements.iter_mut() {
                element.declare(&mut self.resolver);
            }

            let mut scope = self.resolver.scope.clone();
            scope.parent = None;
            module.scope = scope;

            self.resolver.exit();

            self.resolver.insert(module);
        }

        for &identity in &self.order {
            let module_identity = *self.modules.get(&identity).unwrap();
            let mut module = self.resolver.scope.find(module_identity).unwrap().clone();

            self.resolver.enter_scope(module.scope.clone());

            let elements = &mut self.parsers.get_mut(&identity).unwrap().output;

            for element in elements.iter_mut() {
                element.resolve(&mut self.resolver);
            }

            let mut scope = self.resolver.scope.clone();
            scope.parent = None;
            module.scope = scope;

            self.resolver.exit();

            self.resolver.insert(module);
        }

        self.errors.extend(
            self.resolver
                .errors
                .iter()
                .map(|error| {
                    CompileError::Resolve(error.clone())
                })
        );

        let duration = Duration::from_nanos(self.timer.lap().unwrap());

        self.reporter.symbols(&self.resolver.scope.collect());

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
                    .map(|error| {
                        CompileError::Analyze(error.clone())
                    })
            );

            self.analyzers.insert(identity, analyzer);

            let duration = Duration::from_nanos(self.timer.lap().unwrap());

            self.reporter.finish("analyzing", duration);
        }
    }

    pub fn generate(&mut self) {
        let target_triple = inkwell::targets::TargetMachine::get_default_triple();

        for &identity in &self.order {
            let location = self.inputs.get(&identity).unwrap();
            let stem = Str::from(location.stem().unwrap().to_string());
            let analysis = self.analyzers.get(&identity).unwrap().output.clone();
            let module = self.generator.context.create_module(stem.as_str().unwrap());

            module.set_triple(&target_triple);

            self.generator.modules.insert(stem, module);
            self.generator.current_module = stem.clone();

            let schema =
                Self::schema(
                    *location,
                    Resolver::schema(&mut self.resolver, identity),
                );

            self.reporter.start("generating");

            self.generator.generate(analysis);

            match schema.as_path() {
                Ok(path) => {
                    match File::create(&path) {
                        Ok(mut file) => {
                            if let Err(error) = file.write_all(self.generator.current_module().print_to_string().to_string().as_bytes()) {
                                self.errors.push(
                                    CompileError::Track(TrackError::new(tracker::error::ErrorKind::from_io(error, schema), Span::void()))
                                );

                                return;
                            }

                            self.outputs.insert(identity, schema);
                        }

                        Err(error) => {
                            self.errors.push(
                                CompileError::Track(TrackError::new(tracker::error::ErrorKind::from_io(error, schema), Span::void()))
                            )
                        }
                    }
                }

                Err(error) => {
                    self.errors.push(
                        CompileError::Track(error)
                    )
                }
            }

            let duration = Duration::from_nanos(self.timer.lap().unwrap());

            self.reporter
                .finish("generating", duration);
        }

        self.errors.extend(
            self.generator
                .errors
                .iter()
                .map(|error| {
                    CompileError::Generate(error.clone())
                })
        );
    }

    pub fn emit(&mut self) {
        self.reporter.start("emitting");

        let mut objects = Map::new();

        for &identity in &self.order {
            if let Some(location) = self.outputs.get(&identity) {
                let object = Self::object(*location, None);

                let mut command = Command::new("clang");
                command
                    .arg("-c")
                    .arg(location.to_string())
                    .arg("-o")
                    .arg(object.to_string());

                let status = command.status().expect("failed to run clang");

                if status.success() {
                    objects.insert(identity, object);
                } else {
                    panic!("clang failed compiling {}", location);
                }
            }
        }

        let mut link = Command::new("clang");

        for &identity in &self.order {
            if let Some(object) = objects.get(&identity) {
                link.arg(object.to_string());
            }
        }

        link.arg("/home/ali/Projects/axo/examples/libc/formatter.o".to_string());

        let executable = Self::executable(*self.outputs.get(&self.order[0]).unwrap(), None);

        link.arg("-o").arg(executable.to_string());

        let program = link.get_program().to_string_lossy();

        let args: Vec<String> = link
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();

        let clean_cmd_string = if args.is_empty() {
            program.into_owned()
        } else {
            format!("{} {}", program, args.join(" "))
        };

        self.reporter.run(format!("{}", clean_cmd_string));

        let status = link.status().expect("failed to link");

        if !status.success() {
            panic!("linking failed");
        }

        self.reporter.run(format!("{}", executable));

        Command::new(executable.to_string()).status().expect("failed to execute");

        let duration = Duration::from_nanos(self.timer.lap().unwrap());

        self.reporter.finish("emitting", duration);
    }

    fn schema(location: Location<'session>, configuration: Option<Str<'session>>) -> Location<'session> {
        let schema = if let Some(schema) = configuration {
            PathBuf::from(schema.to_string())
        } else {
            let path = location.to_path().unwrap();
            let parent = path.parent().unwrap();

            parent.join(location.stem().unwrap()).with_extension("ll")
        };

        Location::Entry(Str::from(schema))
    }

    fn object(location: Location<'session>, configuration: Option<Str<'session>>) -> Location<'session> {
        let schema = if let Some(schema) = configuration {
            PathBuf::from(schema.to_string())
        } else {
            let path = location.to_path().unwrap();
            let parent = path.parent().unwrap();

            parent.join(location.stem().unwrap()).with_extension("o")
        };

        Location::Entry(Str::from(schema))
    }

    fn executable(location: Location<'session>, configuration: Option<Str<'session>>) -> Location<'session> {
        let schema = if let Some(schema) = configuration {
            PathBuf::from(schema.to_string())
        } else {
            let path = location.to_path().unwrap();
            let parent = path.parent().unwrap();

            parent.join(location.stem().unwrap()).with_extension("")
        };

        Location::Entry(Str::from(schema))
    }
}
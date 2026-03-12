mod registry;

use {
    crate::{
        data::*,
        initializer::{
            Initializer,
            InitializeError,
        },
        internal::{
            platform::{PathBuf, File, Write},
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
        generator::{Generator, GenerateError, Backend, Inkwell},
        resolver::scope::Scope,
        tracker::Peekable,
    }
};
use crate::analyzer::AnalyzeError;
use crate::checker::{CheckError, Checker};

pub enum CompileError<'error> {
    Initialize(InitializeError<'error>),
    Scan(ScanError<'error>),
    Parse(ParseError<'error>),
    Resolve(ResolveError<'error>),
    Check(CheckError<'error>),
    Analyze(AnalyzeError<'error>),
    Generate(GenerateError<'error>),
    Track(TrackError<'error>),
}

pub struct Session<'session> {
    pub timer: DefaultTimer,
    pub reporter: Reporter,
    pub inputs: Map<Identity, Location<'session>>,
    pub modules: Map<Identity, Identity>,
    pub initializer: Initializer<'session>,
    pub scanners: Map<Identity, Scanner<'session>>,
    pub parsers: Map<Identity, Parser<'session>>,
    pub resolver: Resolver<'session>,
    pub analyzers: Map<Identity, Analyzer<'session>>,
    pub generator: Generator<'session, Inkwell<'session>>,
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

        resolver.add(configuration);

        let duration = Duration::from_nanos(timer.lap().unwrap());

        let verbosity = Resolver::verbosity(&mut resolver);

        let reporter = Reporter::new(verbosity);

        let context = Context::create();
        let context_ref = unsafe {
            ContextRef::new(context.raw())
        };

        let backend = Inkwell::new(context_ref);

        let generator = Generator::new(backend);

        logger.finish("initializing", duration);

        Session {
            timer,
            reporter,
            inputs,
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

            self.register();
            if !self.errors.is_empty() { break 'pipeline; }

            self.resolve();
            if !self.errors.is_empty() { break 'pipeline; }

            self.check();
            //if !self.errors.is_empty() { break 'pipeline; }

            self.analyze();
            if !self.errors.is_empty() { break 'pipeline; }

            self.generate();
            if !self.errors.is_empty() { break 'pipeline; }

            self.emit();
        }

        let duration = Duration::from_nanos(self.timer.lap().unwrap());

        for error in &self.errors {
            match error {
                CompileError::Initialize(error) => self.reporter.error(&error),
                CompileError::Scan(error) => self.reporter.error(&error),
                CompileError::Parse(error) => self.reporter.error(&error),
                CompileError::Resolve(error) => self.reporter.error(&error),
                CompileError::Check(error) => self.reporter.error(&error),
                CompileError::Analyze(error) => self.reporter.error(&error),
                CompileError::Generate(error) => self.reporter.error(&error),
                CompileError::Track(error) => self.reporter.error(&error),
            }
        }

        self.reporter.finish("compilation", duration);
    }

    pub fn scan(&mut self) {
        self.reporter.start("scanning");

        for (identity, location) in &self.inputs {
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
            
            self.scanners.insert(*identity, scanner);
        }

        let duration = Duration::from_nanos(self.timer.lap().unwrap());

        self
            .reporter
            .finish("scanning", duration);
    }

    pub fn parse(&mut self) {
        self.reporter.start("parsing");

        for (identity, location) in &self.inputs {
            let mut parser = Parser::new(*location);

            let tokens = self.scanners.get(identity).unwrap().output.clone();
            
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

            self.parsers.insert(*identity, parser);
        }

        let duration = Duration::from_nanos(self.timer.lap().unwrap());

        self.reporter.finish("parsing", duration);
    }

    pub fn register(&mut self) {
        let modules: Vec<_> = self.inputs
            .iter()
            .map(|(identity, location)| {
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

                let elements = &mut self.parsers.get_mut(identity).unwrap().output;

                let mut scope = Scope::new();

                for element in elements.iter_mut() {
                    if let ElementKind::Symbolize(symbol) = &mut element.kind {
                        element.reference = Some(symbol.identity);
                        scope.symbols.insert(symbol.clone());
                    }
                }
                
                let symbol = Symbol::new(
                    SymbolKind::Module(Module::new(head)),
                    span,
                    Visibility::Public,
                ).with_scope(scope);

                self.modules.insert(*identity, symbol.identity);

                symbol
            })
            .collect();

        for module in modules {
            self.resolver.add(module);
        }

        self.reporter.symbols(&self.resolver.scope.all());
    }
    
    pub fn resolve(&mut self) {
        self.reporter.start("resolving");

        for (identity, _location) in &self.inputs {
            let elements = self.parsers.get(identity).unwrap().output.clone();
            let identity = self.modules.get(identity).unwrap();
            let module = self.resolver.scope.get_identity(*identity).unwrap();
            
            self.resolver.enter_scope(module.scope.clone());

            self.resolver.set_input(elements);
            
            self.resolver.resolve();
            
            self.resolver.exit();
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

        self.reporter.finish("resolving", duration);
    }

    pub fn check(&mut self) {
        self.reporter.start("checking");

        let identities: Vec<_> = self.inputs.keys().copied().collect();

        for identity in identities {
            let elements = &mut self.parsers.get_mut(&identity).unwrap().output;

            let mut checker = Checker::new(elements);

            checker.check();

            self.errors.extend(
                checker
                    .errors
                    .into_iter()
                    .map(|error| {
                        CompileError::Check(error.clone())
                    })
            );
        }

        let duration = Duration::from_nanos(self.timer.lap().unwrap());

        self.reporter.finish("checking", duration);
    }
    
    pub fn analyze(&mut self) {
        let identities: Vec<_> = self.inputs.keys().copied().collect();
        
        for identity in identities {
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


        for (identity, location) in &self.inputs.clone() {
            let stem = Str::from(location.stem().unwrap().to_string());
            let analysis = self.analyzers.get(identity).unwrap().output.clone();
            let module = self.generator.backend.context.create_module(stem.as_str().unwrap());

            module.set_triple(&target_triple);

            self.generator.backend.modules.insert(stem, module);
            self.generator.backend.current_module = stem.clone();

            let schema =
                Self::schema(
                    *location,
                    Resolver::schema(&mut self.resolver, *identity),
                );

            self.reporter.start("generating");

            self.generator.backend.generate(analysis);

            self.generator.errors.extend(self.generator.backend.errors.clone());

            match schema.as_path() {
                Ok(path) => {
                    match File::create(&path) {
                        Ok(mut file) => {
                            if let Err(error) = file.write_all(self.generator.backend.current_module().print_to_string().to_string().as_bytes()) {
                                self.errors.push(
                                    CompileError::Track(TrackError::new(tracker::error::ErrorKind::from_io(error, schema), Span::void()))
                                );

                                return;
                            }

                            self.outputs.insert(*identity, schema);
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

            self.errors.extend(
                self.generator
                    .errors
                    .iter()
                    .map(|error| {
                        CompileError::Generate(error.clone())
                    })
            );

            let duration = Duration::from_nanos(self.timer.lap().unwrap());

            self.reporter
                .finish("generating", duration);
        }
    }

    pub fn emit(&mut self) {
        let mut objects = Map::new();

        for (identity, location) in &self.outputs.clone() {
            let object = Self::object(*location, None);

            let mut command = std::process::Command::new("clang");
            command
                .arg("-c")
                .arg(location.to_string())
                .arg("-o")
                .arg(object.to_string());

            let status = command.status().expect("failed to run clang");

            if status.success() {
                objects.insert(*identity, object);
            } else {
                panic!("clang failed compiling {}", location);
            }
        }

        let mut link = std::process::Command::new("clang");

        for object in objects.values() {
            link.arg(object.to_string());
        }

        link.arg("/home/ali/Projects/axo/examples/libc/formatter.o".to_string());

        let executable = Self::executable(*objects.get(&0).unwrap(), None);

        link.arg("-o").arg(executable.to_string());

        let status = link.status().expect("failed to link");

        if !status.success() {
            panic!("linking failed");
        }

        std::process::Command::new(executable.to_string()).status().expect("failed to execute");
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

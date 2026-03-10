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
            Spanned,
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
use crate::checker::{CheckError, Checker};

pub enum CompileError<'error> {
    Initialize(InitializeError<'error>),
    Scan(ScanError<'error>),
    Parse(ParseError<'error>),
    Resolve(ResolveError<'error>),
    Check(CheckError<'error>),
    Generate(GenerateError<'error>),
    Track(TrackError<'error>),
}

pub struct Session<'session> {
    pub timer: DefaultTimer,
    pub reporter: Reporter,
    pub inputs: Map<Identity, Location<'session>>,
    pub initializer: Initializer<'session>,
    pub scanners: Map<Identity, Scanner<'session>>,
    pub parsers: Map<Identity, Parser<'session>>,
    pub resolver: Resolver<'session>,
    pub analyzers: Map<Identity, Analyzer<'session>>,
    pub generator: Generator<'session, Inkwell<'session>>,
    pub context: Context,
    pub errors: Vec<CompileError<'session>>,
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
            let identity = resolver.next_identity();

            inputs.insert(identity, target.clone());
        });

        let mut errors = Vec::new();

        errors.extend(
            initializer
                .errors
                .iter()
                .map(|error| {
                    CompileError::Initialize(error.clone())
                })
        );

        let preferences = initializer
            .output
            .clone()
            .into_iter()
            .map(|preference| {
                let span = preference.borrow_span();

                Symbol::new(
                    resolver.next_identity(),
                    SymbolKind::Preference(preference),
                    span,
                    Visibility::Public,
                )
            })
            .collect::<Vec<Symbol>>();

        resolver.scope.extend(preferences);

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
            initializer,
            scanners: Map::new(),
            parsers: Map::new(),
            resolver,
            analyzers: Map::new(),
            generator,
            context,
            errors,
        }
    }

    pub fn compile(&mut self) {
        self.scan();
        self.parse();
        self.register();
        self.resolve();
        self.check();
        self.analyze();
        self.generate();

        let duration = Duration::from_nanos(self.timer.lap().unwrap());

        for error in &self.errors {
            match error {
                CompileError::Initialize(error) => self.reporter.error(&error),
                CompileError::Scan(error) => self.reporter.error(&error),
                CompileError::Parse(error) => self.reporter.error(&error),
                CompileError::Resolve(error) => self.reporter.error(&error),
                CompileError::Check(error) => self.reporter.error(&error),
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
                        let symbol_id = self.resolver.next_identity();
                        symbol.id = symbol_id;
                        element.reference = Some(symbol_id);
                        scope.symbols.insert(symbol.clone());
                    }
                }
                
                let symbol = Symbol::new(
                    *identity,
                    SymbolKind::Module(Module::new(head)),
                    span,
                    Visibility::Public,
                ).with_scope(scope);

                symbol
            })
            .collect();

        for module in modules {
            self.resolver.add(module);
        }
    }
    
    pub fn resolve(&mut self) {
        self.reporter.start("resolving");

        for (identity, _location) in &self.inputs {
            let elements = self.parsers.get(identity).unwrap().output.clone();
            let module = self.resolver.scope.get_identity(*identity).unwrap();
            
            self.resolver.enter_scope(module.scope.clone());

            self.resolver.set_input(elements);
            
            self.resolver.resolve();
            
            self.resolver.exit();
        }

        self.reporter.symbols(&self.resolver.scope.all());

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

        for identity in self.inputs.keys()  {
            let mut elements = self.parsers.get(identity).unwrap().output.clone();
            let mut checker = Checker::new(&mut elements);

            checker.check();

            self.errors.extend(
                checker
                    .errors
                    .iter()
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

            self.analyzers.insert(identity, analyzer);

            let duration = Duration::from_nanos(self.timer.lap().unwrap());

            self.reporter.finish("analyzing", duration);
        }
    }
    
    pub fn generate(&mut self) {
        for (identity, location) in &self.inputs.clone() {
            let stem = Str::from(location.stem().unwrap().to_string());
            let analysis = self.analyzers.get(identity).unwrap().output.clone();
            let module = self.generator.backend.context.create_module(stem.as_str().unwrap());
            self.generator.backend.modules.insert(stem, module);
            self.generator.backend.current_module = stem.clone();

            let schema =
                Self::output(
                    *location,
                    Resolver::schema(&mut self.resolver, *identity),
                );

            self.reporter.start("generating");

            self.generator.backend.generate(analysis);

            self.generator.errors.extend(self.generator.backend.errors.clone());

            match File::create(&schema) {
                Ok(mut file) => {
                    if let Err(error) = file.write_all(self.generator.backend.current_module().print_to_string().to_string().as_bytes()) {
                        let location = Location::Entry(Str::from(schema.clone()));

                        self.errors.push(
                            CompileError::Track(TrackError::new(tracker::error::ErrorKind::from_io(error, location), Span::void()))
                        )
                    }
                }

                Err(error) => {
                    let location = Location::Entry(Str::from(schema.clone()));

                    self.errors.push(
                        CompileError::Track(TrackError::new(tracker::error::ErrorKind::from_io(error, location), Span::void()))
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

    fn output(location: Location<'session>, schema: Option<Str<'session>>) -> PathBuf {
        let schema = if let Some(schema) = schema {
            PathBuf::from(schema.to_string())
        } else {
            let path = location.to_path().unwrap();
            let parent = path.parent().unwrap();
            
            parent.join(location.stem().unwrap()).with_extension("ll")
        };

        schema
    }
}

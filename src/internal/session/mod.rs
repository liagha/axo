mod registry;

use {
    crate::{
        analyzer::{AnalyzeError, Analyzer},
        data::{memory::replace, *},
        generator::{Backend, GenerateError, Generator},
        initializer::{InitializeError, Initializer},
        internal::{
            hash::{Map, Set},
            platform::{create_dir_all, Command, File, PathBuf, Write},
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

const RUNTIME: &[&str] = &[
    "/home/ali/Projects/axo/runtime/cast.axo",
    "/home/ali/Projects/axo/runtime/cast.c",
    "/home/ali/Projects/axo/runtime/file.axo",
    "/home/ali/Projects/axo/runtime/file.c",
    "/home/ali/Projects/axo/runtime/memory.axo",
    "/home/ali/Projects/axo/runtime/memory.c",
    "/home/ali/Projects/axo/runtime/print.axo",
    "/home/ali/Projects/axo/runtime/print.c",
    "/home/ali/Projects/axo/runtime/process.axo",
    "/home/ali/Projects/axo/runtime/process.c",
    "/home/ali/Projects/axo/runtime/string.axo",
    "/home/ali/Projects/axo/runtime/string.c",
    "/home/ali/Projects/axo/runtime/input.axo",
    "/home/ali/Projects/axo/runtime/input.c",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InputKind {
    Source,
    Schema,
    Object,
    C,
}

impl InputKind {
    pub fn from_path(string: &str) -> Option<Self> {
        if string.ends_with(".axo") {
            Some(InputKind::Source)
        } else if string.ends_with(".ll") {
            Some(InputKind::Schema)
        } else if string.ends_with(".o") {
            Some(InputKind::Object)
        } else if string.ends_with(".c") {
            Some(InputKind::C)
        } else {
            None
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            InputKind::Source => "axo",
            InputKind::Schema => "ll",
            InputKind::Object => "o",
            InputKind::C => "c",
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
    Invalid(Location<'error>),
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
    fn traverse(target: &Location<'session>, inputs: &mut Map<Identity, (InputKind, Location<'session>)>) -> bool {
        let Ok(path) = target.to_path() else {
            return false;
        };

        if !path.is_dir() {
            return false;
        }

        let mut stack = vec![path];

        while let Some(current) = stack.pop() {
            if let Ok(entries) = std::fs::read_dir(current) {
                for entry in entries.flatten() {
                    let child = entry.path();
                    if child.is_dir() {
                        stack.push(child);
                    } else {
                        let string = child.to_string_lossy().into_owned();
                        if let Some(kind) = InputKind::from_path(&string) {
                            let location = Location::Entry(Str::from(string));
                            inputs.insert(inputs.len(), (kind, location));
                        }
                    }
                }
            }
        }

        true
    }

    pub fn start() -> Self {
        let mut timer = DefaultTimer::new_default();
        let _ = timer.start();

        let mut initializer = Initializer::new(Location::Flag);
        let mut resolver = Resolver::new();

        let verbose = Resolver::verbosity(&mut resolver);
        let reporter = Reporter::new(verbose);

        reporter.start("initializing");

        let mut inputs = Map::new();
        let mut errors = Vec::new();

        for path in RUNTIME {
            if let Some(kind) = InputKind::from_path(path) {
                let location = Location::Entry(Str::from(path.to_string()));
                inputs.insert(inputs.len(), (kind, location));
            }
        }

        initializer.initialize().iter().for_each(|target| {
            if !Self::traverse(target, &mut inputs) {
                let string = target.to_string();
                if let Some(kind) = InputKind::from_path(&string) {
                    inputs.insert(inputs.len(), (kind, target.clone()));
                } else {
                    errors.push(CompileError::Invalid(target.clone()));
                }
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
                    TokenKind::Identifier(Str::from("configuration")),
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
        let verbose = Resolver::verbosity(&mut resolver);
        let reporter = Reporter::new(verbose);

        let context = Context::create();
        let reference = unsafe { ContextRef::new(context.raw()) };
        let generator = Generator::new(reference);

        let initial = errors.len();
        reporter.finish("initializing", duration, initial);

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
        self.reporter.finish("compilation", duration, self.errors.len());

        for error in &self.errors {
            match error {
                CompileError::Initialize(error) => self.reporter.error(error),
                CompileError::Scan(error) => self.reporter.error(error),
                CompileError::Parse(error) => self.reporter.error(error),
                CompileError::Resolve(error) => self.reporter.error(error),
                CompileError::Analyze(error) => self.reporter.error(error),
                CompileError::Generate(error) => self.reporter.error(error),
                CompileError::Track(error) => self.reporter.error(error),
                CompileError::Invalid(_) => {}
            }
        }
    }

    pub fn populate(&mut self) {
        let mut keys: Vec<_> = self.inputs.keys().copied().collect();
        keys.sort();

        let modules: Vec<_> = keys
            .into_iter()
            .filter_map(|identity| {
                let (kind, location) = self.inputs.get(&identity).unwrap();

                if *kind != InputKind::Source {
                    return None;
                }

                let stem = Str::from(location.stem().unwrap().to_string());
                let span = Span::file(Str::from(location.to_string())).unwrap();

                let head = Element::new(
                    ElementKind::Literal(Token::new(TokenKind::Identifier(stem), span)),
                    span,
                ).into();

                let symbol = Symbol::new(
                    SymbolKind::Module(Module::new(head)),
                    span,
                    Visibility::Public,
                );

                self.modules.insert(identity, symbol.identity);
                Some(symbol)
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

        let sequence: Vec<String> = self
            .order
            .iter()
            .map(|key| self.inputs.get(key).unwrap().1.stem().unwrap().to_string())
            .collect();

        self.reporter.order(&sequence);
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
        let initial = self.errors.len();
        self.reporter.start("scanning");

        let mut keys: Vec<_> = self.inputs.keys().copied().collect();
        keys.sort();

        for key in keys {
            let (kind, location) = self.inputs.get(&key).unwrap();

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

                self.scanners.insert(key, scanner);
            }
        }

        let duration = Duration::from_nanos(self.timer.lap().unwrap());
        self.reporter.finish("scanning", duration, self.errors.len() - initial);
    }

    pub fn parse(&mut self) {
        let initial = self.errors.len();
        self.reporter.start("parsing");

        let mut keys: Vec<_> = self.inputs.keys().copied().collect();
        keys.sort();

        for key in keys {
            let (kind, location) = self.inputs.get(&key).unwrap();

            if *kind == InputKind::Source {
                let mut parser = Parser::new(*location);
                let tokens = self.scanners.get(&key).unwrap().output.clone();

                parser.set_input(tokens);
                parser.parse();

                self.reporter.elements(&parser.output);

                self.errors.extend(
                    parser
                        .errors
                        .iter()
                        .map(|error| CompileError::Parse(error.clone())),
                );

                self.parsers.insert(key, parser);
            }
        }

        let duration = Duration::from_nanos(self.timer.lap().unwrap());
        self.reporter.finish("parsing", duration, self.errors.len() - initial);
    }

    pub fn resolve(&mut self) {
        let initial = self.errors.len();
        self.reporter.start("resolving");

        for &key in &self.order {
            let target = *self.modules.get(&key).unwrap();
            let mut module = self.resolver.get_symbol(target).unwrap().clone();
            let scope = replace(&mut module.scope, Scope::new(None));

            self.resolver.enter_scope(scope);

            let elements = &mut self.parsers.get_mut(&key).unwrap().output;

            for element in elements.iter_mut() {
                element.declare(&mut self.resolver);
            }

            let active = self.resolver.active;
            self.resolver.exit();

            module.scope = self.resolver.scopes.remove(&active).unwrap();
            self.resolver.insert(module);
        }

        self.reporter.symbols(&self.resolver.collect());

        for &key in &self.order {
            let target = *self.modules.get(&key).unwrap();
            let mut module = self.resolver.get_symbol(target).unwrap().clone();
            let scope = replace(&mut module.scope, Scope::new(None));

            self.resolver.enter_scope(scope);

            let elements = &mut self.parsers.get_mut(&key).unwrap().output;

            for element in elements.iter_mut() {
                element.resolve(&mut self.resolver);
            }

            let active = self.resolver.active;
            self.resolver.exit();

            module.scope = self.resolver.scopes.remove(&active).unwrap();
            self.resolver.insert(module);
        }

        for &key in &self.order {
            let target = *self.modules.get(&key).unwrap();
            let mut module = self.resolver.get_symbol(target).unwrap().clone();
            let scope = replace(&mut module.scope, Scope::new(None));

            self.resolver.enter_scope(scope);

            let elements = &mut self.parsers.get_mut(&key).unwrap().output;

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
        self.reporter.finish("resolving", duration, self.errors.len() - initial);
    }

    pub fn analyze(&mut self) {
        for &key in &self.order {
            let initial = self.errors.len();
            self.reporter.start("analyzing");

            let elements = self.parsers.get(&key).unwrap().output.clone();
            let mut analyzer = Analyzer::new(elements);
            analyzer.analyze(&mut self.resolver);

            self.reporter.analysis(&analyzer.output);

            self.errors.extend(
                analyzer
                    .errors
                    .iter()
                    .map(|error| CompileError::Analyze(error.clone())),
            );

            self.analyzers.insert(key, analyzer);

            let duration = Duration::from_nanos(self.timer.lap().unwrap());
            self.reporter.finish("analyzing", duration, self.errors.len() - initial);
        }
    }

    pub fn generate(&mut self) {
        let triple = inkwell::targets::TargetMachine::get_default_triple();
        let base = self.base();

        for &key in &self.order {
            let initial = self.errors.len();
            let (_, location) = self.inputs.get(&key).unwrap();
            let stem = Str::from(location.stem().unwrap().to_string());
            let analysis = self.analyzers.get(&key).unwrap().output.clone();
            let module = self.generator.context.create_module(stem.as_str().unwrap());

            module.set_triple(&triple);

            self.generator.modules.insert(stem, module);
            self.generator.current_module = stem;

            let custom = Resolver::schema(&mut self.resolver, key);
            let schema = Self::schema(&base, *location, custom);

            self.reporter.start("generating");
            self.generator.generate(analysis);

            match schema.as_path() {
                Ok(path) => {
                    let parent = path.parent().unwrap();
                    let _ = create_dir_all(parent);

                    match File::create(&path) {
                        Ok(mut file) => {
                            let string = self.generator.current_module().print_to_string().to_string();
                            if let Err(error) = file.write_all(string.as_bytes()) {
                                let kind = tracker::error::ErrorKind::from_io(error, schema);
                                let track = TrackError::new(kind, Span::void());
                                self.errors.push(CompileError::Track(track));
                                return;
                            }
                            self.outputs.insert(key, schema);
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

            let duration = Duration::from_nanos(self.timer.lap().unwrap());
            self.reporter.finish("generating", duration, self.errors.len() - initial);
        }

        self.errors.extend(
            self.generator
                .errors
                .iter()
                .map(|error| CompileError::Generate(error.clone())),
        );
    }

    pub fn emit(&mut self) {
        let initial = self.errors.len();
        self.reporter.start("emitting");

        let base = self.base();
        let mut objects = Map::new();
        let mut direct = Vec::new();

        for (&key, &(ref kind, ref location)) in &self.inputs {
            let target = match kind {
                InputKind::Source => {
                    if let Some(output) = self.outputs.get(&key) {
                        Some(*output)
                    } else {
                        None
                    }
                }
                InputKind::Schema | InputKind::C => Some(*location),
                InputKind::Object => {
                    direct.push(*location);
                    None
                }
            };

            if let Some(path) = target {
                let object = Self::object(&base, *location, kind, None);
                let parent = object.to_path().unwrap().parent().unwrap().to_path_buf();
                let _ = create_dir_all(&parent);

                let mut command = Command::new("clang");

                command
                    .arg("-c")
                    .arg(path.to_string())
                    .arg("-o")
                    .arg(object.to_string());

                let status = command.status().expect("failed");

                if status.success() {
                    objects.insert(key, object);
                } else {
                    panic!("failed {}", path);
                }
            }
        }

        let mut link = Command::new("clang");

        for object in objects.values() {
            link.arg(object.to_string());
        }

        for object in direct {
            link.arg(object.to_string());
        }

        let mut keys: Vec<_> = self.inputs.keys().copied().collect();
        keys.sort();

        let key = self.order.last().copied().unwrap_or_else(|| {
            *keys.last().expect("missing")
        });

        let location = self
            .outputs
            .get(&key)
            .copied()
            .unwrap_or_else(|| self.inputs.get(&key).unwrap().1);

        let executable = Self::executable(&base, location, None);
        link.arg("-o").arg(executable.to_string());

        let program = link.get_program().to_string_lossy();
        let arguments: Vec<String> = link
            .get_args()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect();

        let execution = if arguments.is_empty() {
            program.into_owned()
        } else {
            format!("{} {}", program, arguments.join(" "))
        };

        self.reporter.run(execution);

        let status = link.status().expect("failed");

        if !status.success() {
            panic!("failed");
        }

        let duration = Duration::from_nanos(self.timer.lap().unwrap());
        self.reporter.finish("emitting", duration, self.errors.len() - initial);
        self.reporter.run(format!("{}", executable));

        Command::new(executable.to_string()).status().expect("failed");
    }

    fn base(&self) -> PathBuf {
        let paths: Vec<_> = self.inputs.values().filter_map(|(_, location)| location.to_path().ok()).collect();

        if paths.is_empty() {
            return PathBuf::from(".");
        }

        let mut base = paths[0].parent().unwrap().to_path_buf();

        for path in &paths[1..] {
            let parent = path.parent().unwrap();
            let mut current = PathBuf::new();
            let mut left = base.components();
            let mut right = parent.components();

            while let (Some(first), Some(second)) = (left.next(), right.next()) {
                if first == second {
                    current.push(first);
                } else {
                    break;
                }
            }

            base = current;
        }

        base
    }

    fn schema(base: &PathBuf, location: Location<'session>, custom: Option<Str<'session>>) -> Location<'session> {
        let target = if let Some(path) = custom {
            PathBuf::from(path.to_string())
        } else {
            base.join("build").join("schema").join(location.stem().unwrap()).with_extension("ll")
        };

        Location::Entry(Str::from(target))
    }

    fn object(base: &PathBuf, location: Location<'session>, kind: &InputKind, custom: Option<Str<'session>>) -> Location<'session> {
        let target = if let Some(path) = custom {
            PathBuf::from(path.to_string())
        } else {
            base.join("build")
                .join("objects")
                .join(kind.extension())
                .join(location.stem().unwrap())
                .with_extension("o")
        };

        Location::Entry(Str::from(target))
    }

    fn executable(base: &PathBuf, location: Location<'session>, custom: Option<Str<'session>>) -> Location<'session> {
        let target = if let Some(path) = custom {
            PathBuf::from(path.to_string())
        } else {
            base.join("build").join(location.stem().unwrap()).with_extension("")
        };

        Location::Entry(Str::from(target))
    }
}

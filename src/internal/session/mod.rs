mod registry;

use {
    crate::{
        analyzer::{AnalyzeError, Analyzer},
        data::{memory::replace, *},
        format::{Display, Show, Verbosity},
        initializer::{InitializeError, Initializer},
        internal::{
            hash::{Map, Set},
            platform::{create_dir_all, Command, PathBuf},
            timer::{DefaultTimer, Duration},
        },
        parser::{Element, ElementKind, ParseError, Parser, Symbol, SymbolKind, Visibility},
        reporter::Error,
        resolver::{Resolvable, ResolveError, Resolver, Scope},
        scanner::{ScanError, Scanner, Token, TokenKind},
        tracker::{self, Location, Peekable, Span, TrackError},
    },
    broccli::{xprintln, Color},
};

#[cfg(feature = "generator")]
use {
    crate::{
        generator::{Backend, GenerateError, Generator},
        internal::platform::{File, Write},
    },
    inkwell::{
        context::{Context, ContextRef},
        targets::TargetMachine,
    },
};

const RUNTIME: &[&str] = &[
    "./runtime/cast.axo",
    "./runtime/cast.c",
    "./runtime/file.axo",
    "./runtime/file.c",
    "./runtime/memory.axo",
    "./runtime/memory.c",
    "./runtime/print.axo",
    "./runtime/print.c",
    "./runtime/process.axo",
    "./runtime/process.c",
    "./runtime/string.axo",
    "./runtime/string.c",
    "./runtime/input.axo",
    "./runtime/input.c",
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
    #[cfg(feature = "generator")]
    Generate(GenerateError<'error>),
    Track(TrackError<'error>),
}

pub struct Session<'session> {
    pub timer: DefaultTimer,
    pub order: Vec<Identity>,
    pub inputs: Map<Identity, (InputKind, Location<'session>)>,
    pub modules: Map<Identity, Identity>,
    pub initializer: Initializer<'session>,
    pub scanners: Map<Identity, Scanner<'session>>,
    pub parsers: Map<Identity, Parser<'session>>,
    pub resolver: Resolver<'session>,
    pub analyzers: Map<Identity, Analyzer<'session>>,
    #[cfg(feature = "generator")]
    pub generator: Generator<'session>,
    #[cfg(feature = "generator")]
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
        let verbosity = Verbosity::from(verbose);

        if verbosity != Verbosity::Off {
            xprintln!(
                "Started {}." => Color::Blue,
                "`initializing`" => Color::White
            );
            xprintln!();
        }

        let mut inputs = Map::new();
        let mut errors = Vec::new();

        for path in RUNTIME {
            if let Some(kind) = InputKind::from_path(path) {
                let location = Location::Entry(Str::from(path.to_string()));
                inputs.insert(inputs.len(), (kind, location));
            }
        }

        initializer.initialize().iter().for_each(|(target, span)| {
            if !Self::traverse(target, &mut inputs) {
                let string = target.to_string();

                if let Some(kind) = InputKind::from_path(&string) {
                    inputs.insert(inputs.len(), (kind, target.clone()));
                } else {
                    errors.push(
                        CompileError::Track(
                            TrackError::new(
                                tracker::error::ErrorKind::UnSupportedInput(target.clone()),
                                span.clone(),
                            )
                        )
                    );
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

        for symbol in initializer.output.clone() {
            resolver.insert(symbol);
        }

        let directive = Symbol::new(
            SymbolKind::Module(Module::new(Box::from(Element::new(
                ElementKind::Literal(Token::new(
                    TokenKind::Identifier(Str::from("directive")),
                    Span::void(),
                )),
                Span::void(),
            )))),
            Span::void(),
            Visibility::Public,
        )
            .with_members(initializer.output.clone());

        resolver.insert(directive);

        let duration = Duration::from_nanos(timer.lap().unwrap());
        let verbose = Resolver::verbosity(&mut resolver);
        let verbosity = Verbosity::from(verbose);

        #[cfg(feature = "generator")]
        let (context, generator) = {
            let context = Context::create();
            let reference = unsafe { ContextRef::new(context.raw()) };
            let generator = Generator::new(reference);

            (context, generator)
        };

        let initial = errors.len();

        if verbosity != Verbosity::Off {
            let suffix = if initial > 0 {
                format!(" ({} errors)", initial)
            } else {
                String::new()
            };

            xprintln!(
                "Finished {} {}s{}" => Color::Green,
                "`initializing` in" => Color::White,
                duration.as_secs_f64(),
                suffix => Color::Red
            );
            xprintln!();
        }

        Session {
            timer,
            inputs,
            order: Vec::new(),
            modules: Map::new(),
            initializer,
            scanners: Map::new(),
            parsers: Map::new(),
            resolver,
            analyzers: Map::new(),
            #[cfg(feature = "generator")]
            generator,
            #[cfg(feature = "generator")]
            context,
            errors,
            outputs: Map::new(),
        }
    }

    pub fn get_verbosity(&self) -> Verbosity {
        #[allow(invalid_reference_casting)]
        let resolver = unsafe { &mut *(&self.resolver as *const _ as *mut Resolver) };
        Verbosity::from(Resolver::verbosity(resolver))
    }

    pub fn is_active(&self) -> bool {
        self.get_verbosity() != Verbosity::Off
    }

    pub fn report_start(&self, stage: &str) {
        if self.is_active() {
            xprintln!(
                "Started {}." => Color::Blue,
                format!("`{}`", stage) => Color::White
            );
            xprintln!();
        }
    }

    pub fn report_finish(&self, stage: &str, duration: Duration, count: usize) {
        if self.is_active() {
            let suffix = if count > 0 {
                format!(" ({} errors)", count)
            } else {
                String::new()
            };

            xprintln!(
                "Finished {} {}s{}" => Color::Green,
                format!("`{}` in", stage) => Color::White,
                duration.as_secs_f64(),
                suffix => Color::Red
            );
            xprintln!();
        }
    }

    pub fn report_section(&self, head: &str, color: Color, body: String) {
        if self.is_active() && !body.is_empty() {
            xprintln!(
                "{}{}\n{}" => Color::White,
                head => color,
                ":" => Color::White,
                Str::from(body).indent(self.get_verbosity().into()) => Color::White
            );
            xprintln!();
        }
    }

    pub fn report_error<K, H>(&self, error: &Error<K, H>)
    where
        K: Clone + Display,
        H: Clone + Display,
    {
        let (message, details) = error.handle();
        xprintln!(
            "{}\n{}" => Color::Red,
            message => Color::White,
            details => Color::White
        );
        xprintln!();
    }

    pub fn compile(&mut self) {
        'pipeline: {
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

            #[cfg(feature = "generator")]
            self.emit();
        }

        let duration = Duration::from_nanos(self.timer.stop().unwrap());
        self.report_finish("compilation", duration, self.errors.len());

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
        self.report_start("scanning");

        let mut keys: Vec<_> = self.inputs.keys().copied().collect();
        keys.sort();

        for key in keys {
            let (kind, location) = self.inputs.get(&key).unwrap();

            if *kind == InputKind::Source {
                let mut scanner = Scanner::new(*location);

                scanner.prepare();
                scanner.scan();

                let verbosity = self.get_verbosity().into();
                self.report_section(
                    "Tokens",
                    Color::Cyan,
                    scanner.output
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

                self.scanners.insert(key, scanner);
            }
        }

        let duration = Duration::from_nanos(self.timer.lap().unwrap());
        self.report_finish("scanning", duration, self.errors.len() - initial);
    }

    pub fn parse(&mut self) {
        let initial = self.errors.len();
        self.report_start("parsing");

        let mut keys: Vec<_> = self.inputs.keys().copied().collect();
        keys.sort();

        for key in keys {
            let (kind, location) = self.inputs.get(&key).unwrap();

            if *kind == InputKind::Source {
                let mut parser = Parser::new(*location);
                let tokens = self.scanners.get(&key).unwrap().output.clone();

                parser.set_input(tokens);
                parser.parse();

                let verbosity = self.get_verbosity().into();
                self.report_section(
                    "Elements",
                    Color::Cyan,
                    parser.output
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

                self.parsers.insert(key, parser);
            }
        }

        let duration = Duration::from_nanos(self.timer.lap().unwrap());
        self.report_finish("parsing", duration, self.errors.len() - initial);
    }

    pub fn resolve(&mut self) {
        let initial = self.errors.len();
        self.report_start("resolving");

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

        let verbosity = self.get_verbosity().into();
        self.report_section(
            "Symbols",
            Color::Blue,
            self.resolver.collect()
                .iter()
                .map(|symbol| {
                    let children = symbol.scope.symbols.iter().map(|symbol| self.resolver.get_symbol(*symbol)).collect::<Vec<_>>();
                    format!("{}\n{}", symbol.format(verbosity), children.format(verbosity))
                })
                .collect::<Vec<String>>()
                .join("\n"),
        );

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
        self.report_finish("resolving", duration, self.errors.len() - initial);
    }

    pub fn analyze(&mut self) {
        let initial = self.errors.len();

        self.report_start("analyzing");

        for &key in &self.order {
            let elements = self.parsers.get(&key).unwrap().output.clone();
            let mut analyzer = Analyzer::new(elements);
            analyzer.analyze(&mut self.resolver);

            let verbosity = self.get_verbosity().into();
            self.report_section(
                "Analysis",
                Color::Blue,
                analyzer.output
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

            self.analyzers.insert(key, analyzer);
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
            let (_, location) = self.inputs.get(&key).unwrap();
            let stem = Str::from(location.stem().unwrap().to_string());
            let analysis = self.analyzers.get(&key).unwrap().output.clone();
            let module = self.generator.context.create_module(stem.as_str().unwrap());

            module.set_triple(&triple);

            self.generator.modules.insert(stem, module);
            self.generator.current_module = stem;

            let custom = Resolver::schema(&mut self.resolver, key);
            let schema = Self::schema(&base, *location, custom);
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
        let initial = self.errors.len();
        self.report_start("emitting");

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

        if self.is_active() {
            xprintln!(
                "Running {}." => Color::Blue,
                format!("`{}`", execution) => Color::White
            );
            xprintln!();
        }

        let status = link.status().expect("failed");

        if !status.success() {
            panic!("failed");
        }

        let duration = Duration::from_nanos(self.timer.lap().unwrap());
        self.report_finish("emitting", duration, self.errors.len() - initial);

        if self.is_active() {
            xprintln!(
                "Running {}." => Color::Blue,
                format!("`{}`", executable) => Color::White
            );
            xprintln!();
        }

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

    #[cfg(feature = "generator")]
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

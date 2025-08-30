use {
    broccli::{Color, xprintln},
    crate::{
        data::{memory, Str},
        format::{format_tokens, Show, Display},
        initial::{Initializer, Preference},
        internal::{platform::Path},
        parser::{Element, ElementKind, Parser, Symbol, Symbolic},
        reporter::{Error},
        resolver::Resolver,
        scanner::{Scanner, Token, TokenKind},
        schema::{Module},
        tracker::{Location, Peekable, Position, Span, Spanned},
    },
    super::timer::{
        DefaultTimer, Duration,
    },
};

pub struct Pipeline<'pipeline, T> {
    data: T,
    resolver: &'pipeline mut Resolver<'pipeline>,
}

impl<'pipeline, T> Pipeline<'pipeline, T> {
    pub fn new(data: T, resolver: &'pipeline mut Resolver<'pipeline>) -> Self {
        Self { data, resolver }
    }

    pub fn then<U, S>(mut self, mut stage: S) -> Pipeline<'pipeline, U>
    where
        S: Stage<'pipeline, T, U>,
    {
        let output = stage.execute(self.resolver, self.data);
        Pipeline {
            data: output,
            resolver: self.resolver,
        }
    }

    pub fn then_with<U, F>(mut self, f: F) -> Pipeline<'pipeline, U>
    where
        F: FnOnce(&mut Resolver<'pipeline>, T) -> U,
    {
        let output = f(self.resolver, self.data);
        Pipeline {
            data: output,
            resolver: self.resolver,
        }
    }

    pub fn finish(self) -> T {
        self.data
    }

    pub fn resolver(&mut self) -> &mut Resolver<'pipeline> {
        self.resolver
    }

    pub fn with_resolver<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Resolver<'pipeline>) -> R,
    {
        f(self.resolver)
    }
}

pub trait Stage<'stage, Input, Output> {
    fn execute(&mut self, resolver: &mut Resolver<'stage>, input: Input) -> Output;
}

#[derive(Debug)]
pub struct Registry<'registry> {
    pub resolver: Resolver<'registry>,
}

impl<'registry> Registry<'registry> {
    pub fn new() -> Self {
        Registry {
            resolver: Resolver::new(),
        }
    }

    pub fn get_preference(resolver: &mut Resolver<'registry>, identifier: Str<'registry>) -> Option<Token<'registry>> {
        let identifier = Element::new(
            ElementKind::Literal(
                Token::new(
                    TokenKind::Identifier(identifier),
                    Span::default(Location::Flag)
                )
            ),
            Span::default(Location::Flag)
        );

        let result = resolver.try_get(&identifier);

        if let Ok(found) = result {
            if let Symbolic::Preference(preference) = found.value {
                return Some(preference.value.clone())
            }
        }

        None
    }

    pub fn get_verbosity(resolver: &mut Resolver<'registry>) -> bool {
        let identifier = Element::new(
            ElementKind::Literal(
                Token::new(
                    TokenKind::Identifier(Str::from("Verbosity")),
                    Span::default(Location::Flag)
                ),
            ),
            Span::default(Location::Flag)
        );

        let result = resolver.try_get(&identifier);

        if let Ok(found) = result {
            if let Symbolic::Preference(preference) = found.value {
                if let TokenKind::Boolean(verbosity) = preference.value.kind {
                    return verbosity
                }
            }
        }

        false
    }

    pub fn get_path(resolver: &mut Resolver<'registry>) -> Str<'registry> {
        let identifier = Element::new(
            ElementKind::Literal(
                Token::new(
                    TokenKind::Identifier(Str::from("Path")),
                    Span::default(Location::Flag)
                ),
            ),
            Span::default(Location::Flag)
        );

        let result = resolver.try_get(&identifier);

        if let Ok(found) = result {
            if let Symbolic::Preference(preference) = found.value {
                if let TokenKind::Identifier(path) = preference.value.kind.clone() {
                    return path.clone()
                }
            }
        }

        Str::default()
    }
}

pub struct CompileLogger {
    verbosity: bool,
    current_target: Option<String>,
    target_count: usize,
    current_index: usize,
}

impl CompileLogger {
    fn new(verbosity: bool, target_count: usize) -> Self {
        Self {
            verbosity,
            current_target: None,
            target_count,
            current_index: 0,
        }
    }

    fn start(&self, stage: &str) {
        if self.verbosity {
            if let Some(ref target) = self.current_target {
                xprintln!(
                    "Started {} {} ({}/{})." => Color::Blue,
                    format!("`{}`", stage) => Color::White,
                    target,
                    self.current_index,
                    self.target_count,
                );
            } else {
                xprintln!(
                    "Started {}." => Color::Blue,
                    format!("`{}`", stage) => Color::White,
                );
            }
            xprintln!();
        }
    }

    fn finish(&self, stage: &str, duration: Duration, error_count: usize) {
        if self.verbosity {
            let target_info = if let Some(ref target) = self.current_target {
                format!(" {} ({}/{})", target, self.current_index, self.target_count)
            } else {
                String::new()
            };

            if error_count > 0 {
                xprintln!(
                    "Finished {}{} {}s with {} {}." => Color::Green,
                    format!("`{}` in", stage) => Color::White,
                    target_info,
                    duration.as_secs_f64(),
                    error_count => Color::BrightRed,
                    "errors" => Color::Red,
                );
            } else {
                xprintln!(
                    "Finished {}{} {}s." => Color::Green,
                    format!("`{}` in", stage) => Color::White,
                    target_info,
                    duration.as_secs_f64(),
                );
            }
            xprintln!();
        }
    }

    fn tokens(&self, tokens: &[Token]) {
        if self.verbosity {
            xprintln!("Tokens:\n{}", &format_tokens(tokens).indent());
            xprintln!();
        }
    }

    fn elements(&self, elements: &[Element]) {
        if self.verbosity {
            let tree = elements
                .iter()
                .map(|element| {
                    Str::from(format!("{:#?}", element))
                })
                .collect::<Vec<Str>>()
                .join("\n");

            if !tree.is_empty() {
                xprintln!("Elements:\n{}" => Color::Green, &tree.indent());
                xprintln!();
            }
        }
    }

    fn symbols(&self, symbols: &[Symbol]) {
        if self.verbosity {
            let tree = symbols
                .iter()
                .map(|symbol| Str::from(format!("{:#?}", symbol)))
                .collect::<Vec<Str>>()
                .join("\n");

            if !tree.is_empty() {
                xprintln!(
                    "{}" => Color::Cyan,
                    format!("Symbols:\n{}", &tree.indent()),
                );
                xprintln!();
            }
        }
    }

    fn errors<K: Display, H: Display>(&self, errors: &[Error<K, H>]) {
        if self.verbosity && !errors.is_empty() {
            for error in errors {
                let (message, details) = error.format();
                xprintln!(
                    "{}\n{}" => Color::Red,
                    message => Color::Orange,
                    details
                );
                xprintln!();
            }
        }
    }

    fn set_current(&mut self, target: String) {
        self.current_index += 1;
        self.current_target = Some(target);
    }

    fn clear_current(&mut self) {
        self.current_target = None;
    }
}

pub struct Compiler<'compiler> {
    pub registry: Registry<'compiler>,
}

impl<'compiler> Compiler<'compiler> {
    pub fn new() -> Self {
        let registry = Registry::new();
        Compiler { registry }
    }

    pub fn compile(&mut self) -> () {
        let mut timer = DefaultTimer::new_default();
        timer.start();

        let verbosity = self.compile_pipeline();

        if verbosity {
            let duration = Duration::from_nanos(timer.elapsed().unwrap());

            xprintln!(
                "Finished {} {}s." => Color::Green,
                "`compilation` in" => Color::White,
                duration.as_secs_f64(),
            );

            xprintln!();
        }
    }

    fn compile_pipeline(&mut self) -> bool {
        let targets = {
            let mut initializer = Initialization::new();
            initializer.execute(&mut self.registry.resolver, ())
        };

        let verbosity = Registry::get_verbosity(&mut self.registry.resolver);
        let mut logger = CompileLogger::new(verbosity, targets.len());

        for target in targets {
            self.registry.resolver.enter();

            let display = target.to_string();
            logger.set_current(display.clone().to_string());

            let tokens = {
                let mut scanner = Scanner::new(target);
                scanner.execute_pipeline(&mut self.registry.resolver, target, &logger)
            };

            let elements = {
                let mut parser = Parser::new(target);
                parser.execute(&mut self.registry.resolver, tokens, &logger)
            };

            {
                self.registry.resolver.execute(elements.clone(), &logger)
            };

            #[cfg(feature = "analyzer")]
            {
                let mut analyzer = crate::analyzer::Analyzer::new();
                analyzer.with_input(elements);

                analyzer.process();

                println!("Instructions:\n{:#?}", analyzer.output);

                #[cfg(feature = "generator")]
                {
                    let context = &inkwell::context::Context::create();
                    let mut generator = crate::generator::Inkwell::new(context);
                    generator.instruct(analyzer.output);
                }
            }

            let span = Span::file(Str::from(target.to_string()));
            let module_name = Path::new(&display)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            let identifier = Element::new(
                ElementKind::Literal(
                    Token::new(
                        TokenKind::Identifier(Str::from(module_name)),
                        span
                    ),
                ),
                span
            );

            let mut module = Symbol::new(Symbolic::Module(Module::new(Box::new(identifier))), span);
            module.with_scope(self.registry.resolver.scope.clone());

            self.registry.resolver.exit();

            self.registry.resolver.define(module);

            logger.clear_current();
        }

        logger.symbols(&*self.registry.resolver.scope.symbols.clone().into_iter().collect::<Vec<_>>());

        verbosity
    }

    pub fn compile_with<Function, Type>(&mut self, pipeline: Function) -> Type
    where
        Function: FnOnce(&mut Self) -> Type,
    {
        pipeline(self)
    }
}

pub struct Initialization<'initialization> {
    initializer: Initializer<'initialization>,
}

impl<'initialization> Initialization<'initialization> {
    pub fn new() -> Self {
        Self {
            initializer: Initializer::new(Location::Flag),
        }
    }
}

impl<'initialization> Stage<'initialization, (), Vec<Location<'initialization>>> for Initialization<'initialization> {
    fn execute(&mut self, resolver: &mut Resolver<'initialization>, _input: ()) -> Vec<Location<'initialization>> {
        let mut timer = DefaultTimer::new_default();
        timer.start();

        let verbosity = Registry::get_verbosity(resolver);
        let logger = CompileLogger::new(verbosity, 0);

        logger.start("initializing");

        let targets = self.initializer.initialize();

        let symbols = self.initializer.output.clone().into_iter().map(|preference| {
            let span = preference.borrow_span();
            Symbol::new(
                Symbolic::Preference(preference),
                span
            )
        }).collect::<Vec<Symbol>>();

        resolver.extend(symbols);

        let duration = Duration::from_nanos(timer.elapsed().unwrap());
        logger.finish("initializing", duration, 0);

        targets
    }
}

impl<'scanner> Scanner<'scanner> {
    pub fn execute_pipeline(&mut self, resolver: &mut Resolver<'scanner>, location: Location<'scanner>, logger: &CompileLogger) -> Vec<Token<'scanner>> {
        let mut timer = DefaultTimer::new_default();
        timer.start();

        self.set_location(location);
        logger.start("scanning");

        let content = location.get_value();

        self.set_input(content);
        self.scan();

        logger.tokens(&self.output);
        logger.errors(&self.errors);

        let duration = Duration::from_nanos(timer.elapsed().unwrap());
        logger.finish("scanning", duration, self.errors.len());

        self.output.clone()
    }
}

impl<'parser> Parser<'parser> {
    pub fn execute(&mut self, resolver: &mut Resolver<'parser>, tokens: Vec<Token<'parser>>, logger: &CompileLogger) -> Vec<Element<'parser>> {
        let mut timer = DefaultTimer::new_default();
        timer.start();

        logger.start("parsing");

        self.set_input(tokens);
        self.parse();

        logger.elements(&self.output);
        logger.errors(&self.errors);

        let duration = Duration::from_nanos(timer.elapsed().unwrap());
        logger.finish("parsing", duration, self.errors.len());

        self.output.clone()
    }
}

impl<'resolver> Resolver<'resolver> {
    pub fn execute(&mut self, elements: Vec<Element<'resolver>>, logger: &CompileLogger) -> () {
        let mut timer = DefaultTimer::new_default();
        timer.start();

        logger.start("resolving");
        
        self.with_input(elements);

        self.process();

        let symbols = self.scope.all();
        logger.errors(self.errors.as_slice());

        let duration = Duration::from_nanos(timer.elapsed().unwrap());
        logger.finish("resolving", duration, self.errors.len());
    }
}
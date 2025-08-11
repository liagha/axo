use {
    broccli::{Color, xprintln},
    crate::{
        data::{memory, string::Str},
        format::{format_tokens, Show},
        initial::{Initializer, Preference},
        parser::{Element, ElementKind, Parser, Symbol},
        resolver::Resolver,
        scanner::{Scanner, Token, TokenKind},
        tracker::{Location, Peekable, Span, Spanned},
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
        S: PipelineStage<'pipeline, T, U>,
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

pub trait PipelineStage<'stage, Input, Output> {
    fn execute(&mut self, resolver: &mut Resolver<'stage>, input: Input) -> Output;
}

pub trait Marked<'marked> {
    #[track_caller]
    fn registry(&self) -> &Registry<'marked>;
    #[track_caller]
    fn registry_mut(&mut self) -> &mut Registry<'marked>;
    #[track_caller]
    fn resolver(&self) -> &Resolver<'marked> {
        &self.registry().resolver
    }
    #[track_caller]
    fn resolver_mut(&mut self) -> &mut Resolver<'marked> {
        &mut self.registry_mut().resolver
    }
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

    pub fn get_verbosity(resolver: &mut Resolver<'registry>) -> bool {
        let identifier = Element::new(ElementKind::Identifier(Str::from("Verbosity")), Span::default(Location::Flag));

        let result = resolver.try_get(&identifier);

        if let Ok(found) = result {
            if let Some(symbol) = found {
                if let Some(preference) = symbol.cast::<Preference<'static>>() {
                    if let TokenKind::Boolean(verbosity) = preference.value.kind {
                        return verbosity
                    }
                }
            }
        }

        false
    }

    pub fn get_path(resolver: &mut Resolver<'registry>) -> Str<'registry> {
        let identifier = Element::new(ElementKind::Identifier(Str::from("Path")), Span::default(Location::Flag));

        let result = resolver.try_get(&identifier);

        if let Ok(found) = result {
            if let Some(symbol) = found {
                if let Some(preference) = symbol.cast::<Preference<'static>>() {
                    if let TokenKind::Identifier(path) = preference.value.kind.clone() {
                        return path.clone()
                    }
                }
            }
        }

        Str::default()
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
        let location = {
            let mut initializer = Initialization::new();
            initializer.execute(&mut self.registry.resolver, ())
        };

        let verbosity = Registry::get_verbosity(&mut self.registry.resolver);

        let tokens = {
            let mut scanner = Scanner::new(location);
            scanner.execute_pipeline(&mut self.registry.resolver, location)
        };

        let elements = {
            let mut parser = Parser::new(location);
            parser.execute_pipeline(&mut self.registry.resolver, tokens)
        };

        self.registry.resolver.execute_pipeline(elements.clone());

        #[cfg(feature = "generator")]
        {
            let mut generation = Generation;
            generation.execute_pipeline(&mut self.registry.resolver, elements.clone());
        }

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

impl<'initialization> PipelineStage<'initialization, (), Location<'initialization>> for Initialization<'initialization> {
    fn execute(&mut self, resolver: &mut Resolver<'initialization>, _input: ()) -> Location<'initialization> {
        let mut timer = DefaultTimer::new_default();
        timer.start();

        let verbosity = Registry::get_verbosity(resolver);

        if verbosity {
            xprintln!(
                "Started {}." => Color::Blue,
                "`initializing`" => Color::White,
            );
            xprintln!();
        }

        self.initializer.initialize();

        let symbols = self.initializer.output.clone().into_iter().map(|preference| {
            let span = preference.borrow_span();
            Symbol::new(
                unsafe { memory::transmute::<_, Preference<'static>>(preference) },
                unsafe { memory::transmute(span) }
            )
        }).collect::<Vec<Symbol>>();

        resolver.extend(symbols);

        let verbosity = Registry::get_verbosity(resolver);

        if verbosity {
            let duration = Duration::from_nanos(timer.elapsed().unwrap());

            xprintln!(
                "Finished {} {}s." => Color::Green,
                "`initializing` in" => Color::White,
                duration.as_secs_f64(),
            );
            xprintln!();
        }

        let path = Registry::get_path(resolver);
        Location::File(Str::from(path))
    }
}

impl<'scanner> Scanner<'scanner> {
    pub fn execute_pipeline(&mut self, resolver: &mut Resolver<'scanner>, location: Location<'scanner>) -> Vec<Token<'scanner>> {
        let mut timer = DefaultTimer::new_default();
        timer.start();

        let verbosity = Registry::get_verbosity(resolver);
        self.set_location(location);

        if verbosity {
            xprintln!(
                "Started {}." => Color::Blue,
                "`scanning`" => Color::White,
            );
            xprintln!();
        }

        let content = location.get_value();

        self.set_input(content);
        self.scan();

        if verbosity {
            xprintln!("Tokens:\n{}", &format_tokens(&self.output).indent());
            xprintln!();

            if !self.errors.is_empty() {
                let duration = Duration::from_nanos(timer.elapsed().unwrap());

                for error in &self.errors {
                    let (message, details) = error.format();
                    xprintln!(
                    "{}\n{}" => Color::Red,
                    message => Color::Orange,
                    details
                );
                    xprintln!();
                }

                xprintln!(
                    "Finished {} {}s with {} {}." => Color::Green,
                    "`scanning` in" => Color::White,
                    duration.as_secs_f64(),
                    self.errors.len() => Color::BrightRed,
                    "errors" => Color::Red,
                );
                xprintln!();
            } else {
                let duration = Duration::from_nanos(timer.elapsed().unwrap());

                xprintln!(
                    "Finished {} {}s." => Color::Green,
                    "`scanning` in" => Color::White,
                    duration.as_secs_f64(),
                );
                xprintln!();
            }
        }

        self.output.clone()
    }
}

impl<'parser> Parser<'parser> {
    pub fn execute_pipeline(&mut self, resolver: &mut Resolver<'parser>, tokens: Vec<Token<'parser>>) -> Vec<Element<'parser>> {
        let mut timer = DefaultTimer::new_default();
        timer.start();

        let verbosity = Registry::get_verbosity(resolver);

        if verbosity {
            xprintln!(
                "Started {}." => Color::Blue,
                "`parsing`" => Color::White,
            );
            xprintln!();
        }

        self.set_input(tokens);
        self.parse();

        if verbosity {
            let tree = self.output
                .iter()
                .map(|element| Str::from(format!("{:?}", element)))
                .collect::<Vec<Str>>()
                .join("\n");

            if !tree.is_empty() {
                xprintln!("Elements:\n{}" => Color::Green, &tree.indent());
                xprintln!();
            }

            if !self.errors.is_empty() {
                let duration = Duration::from_nanos(timer.elapsed().unwrap());

                for error in &self.errors {
                    let (message, details) = error.format();
                    xprintln!(
                        "{}\n{}" => Color::Red,
                        message => Color::Orange,
                        details
                    );
                    xprintln!();
                }

                xprintln!(
                    "Finished {} {}s with {} {}." => Color::Green,
                    "`parsing` in" => Color::White,
                    duration.as_secs_f64(),
                    self.errors.len() => Color::BrightRed,
                    "errors" => Color::Red,
                );
                xprintln!();
            } else {
                let duration = Duration::from_nanos(timer.elapsed().unwrap());

                xprintln!(
                    "Finished {} {}s." => Color::Green,
                    "`parsing` in" => Color::White,
                    duration.as_secs_f64(),
                );
                xprintln!();
            }
        }

        self.output.clone()
    }
}

impl<'resolver> Resolver<'resolver> {
    pub fn execute_pipeline(&mut self, elements: Vec<Element<'resolver>>) -> () {
        let mut timer = DefaultTimer::new_default();
        timer.start();

        let verbosity = Registry::get_verbosity(self);

        if verbosity {
            xprintln!(
                "Started {}." => Color::Blue,
                "`resolving`" => Color::White,
            );
            xprintln!();
        }

        self.process(elements);

        if verbosity {
            let symbols = self.scope.all();

            let tree = symbols
                .iter()
                .map(|symbol| Str::from(format!("{:?}", symbol)))
                .collect::<Vec<Str>>()
                .join("\n");

            if !tree.is_empty() {
                xprintln!(
                    "{}" => Color::Cyan,
                    format!("Symbols:\n{}", &tree.indent()),
                );
                xprintln!();
            }

            if !self.errors.is_empty() {
                for error in &self.errors {
                    let (message, details) = error.format();
                    xprintln!(
                        "{}\n{}" => Color::Red,
                        message => Color::Orange,
                        details
                    );
                    xprintln!();
                }

                let duration = Duration::from_nanos(timer.elapsed().unwrap());

                xprintln!(
                        "Finished {} {}s with {} {}." => Color::Green,
                        "`resolving` in" => Color::White,
                        duration.as_secs_f64(),
                        self.errors.len() => Color::BrightRed,
                        "errors" => Color::Red,
                    );
                xprintln!();
            } else {
                let duration = Duration::from_nanos(timer.elapsed().unwrap());

                xprintln!(
                        "Finished {} {}s." => Color::Green,
                        "`resolving` in" => Color::White,
                        duration.as_secs_f64(),
                    );

                xprintln!();
            }
        }
    }
}

pub struct Generation;

#[cfg(feature = "generator")]
impl<'generator> Generation {
    pub fn execute_pipeline(&mut self, resolver: &mut Resolver<'generator>, elements: Vec<Element<'generator>>) -> () {
        let context = inkwell::context::Context::create();
        let backend = crate::generator::Inkwell::new(&context);
        let mut generator = crate::generator::Generator::new(backend);
        let mut timer = DefaultTimer::new_default();
        timer.start();

        let verbosity = Registry::get_verbosity(resolver);

        if verbosity {
            xprintln!(
                "Started {}." => Color::Blue,
                "`generating`" => Color::White,
            );
            xprintln!();
        }

        let result = generator.generate(elements);

        if verbosity {
            let duration = Duration::from_nanos(timer.elapsed().unwrap());

            xprintln!(
                "Finished {} {}s." => Color::Green,
                "`generating` in" => Color::White,
                duration.as_secs_f64(),
            );
            xprintln!();
        }

        result
    }
}
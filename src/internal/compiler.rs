use {
    broccli::{Color, xprintln},
    crate::{
        data::{memory, string::Str},
        format::{format_tokens, indent},
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
        let identifier = Element::new(ElementKind::Identifier("Verbosity".to_string()), Span::default(Location::Flag));

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

    pub fn get_path(resolver: &mut Resolver<'registry>) -> String {
        let identifier = Element::new(ElementKind::Identifier("Path".to_string()), Span::default(Location::Flag));

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

        String::new()
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
        let mut initializer_stage = InitializerStage::new();
        let location = initializer_stage.execute(&mut self.registry.resolver, ());

        let verbosity = Registry::get_verbosity(&mut self.registry.resolver);

        let mut scanner = Scanner::new(location);
        let tokens = scanner.execute_pipeline(&mut self.registry.resolver, location);

        let path = Registry::get_path(&mut self.registry.resolver);

        let location = if path.is_empty() {
            Location::Flag
        } else {
            Location::File(Str::from(path))
        };

        let mut parser = Parser::new(location);
        let elements = parser.execute_pipeline(&mut self.registry.resolver, tokens);

        let mut timer = DefaultTimer::new_default();
        timer.start();

        let generator = crate::generator::Generator::new();
        generator.print();

        let resolver_verbosity = Registry::get_verbosity(&mut self.registry.resolver);

        if resolver_verbosity {
            xprintln!(
                "Started {}." => Color::Blue,
                "`resolving`" => Color::White,
            );
            xprintln!();
        }

        self.registry.resolver.process(elements);

        let errors = &self.registry.resolver.errors;

        if resolver_verbosity {
            let symbols = self.registry.resolver.scope.all();

            let tree = symbols
                .iter()
                .map(|symbol| {
                    format!("{:?}", symbol)
                })
                .collect::<Vec<String>>()
                .join("\n");

            if !tree.is_empty() {
                xprintln!(
                    "{}" => Color::Cyan,
                    format!("Symbols:\n{}", indent(&tree)),
                );
                xprintln!();
            }

            if !errors.is_empty() {
                for error in errors {
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
                        errors.len() => Color::BrightRed,
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

        verbosity
    }

    pub fn compile_with<Function, Type>(&mut self, pipeline: Function) -> Type
    where
        Function: FnOnce(&mut Self) -> Type,
    {
        pipeline(self)
    }
}
pub struct InitializerStage<'init> {
    initializer: Initializer<'init>,
}

impl<'init> InitializerStage<'init> {
    pub fn new() -> Self {
        Self {
            initializer: Initializer::new(Location::Flag),
        }
    }
}

impl<'init> PipelineStage<'init, (), Location<'init>> for InitializerStage<'init> {
    fn execute(&mut self, resolver: &mut Resolver<'init>, _input: ()) -> Location<'init> {
        self.initializer.initialize();

        let symbols = self.initializer.output.clone().into_iter().map(|preference| {
            let span = preference.borrow_span();
            Symbol::new(
                unsafe { memory::transmute::<_, Preference<'static>>(preference) },
                unsafe { memory::transmute(span) }
            )
        }).collect::<Vec<Symbol>>();

        resolver.extend(symbols);

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

        self.set_input(content.to_string());
        self.scan();

        if verbosity {
            xprintln!("Tokens:\n{}", indent(&format_tokens(&self.output)));
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
                .map(|element| format!("{:?}", element))
                .collect::<Vec<String>>()
                .join("\n");

            if !tree.is_empty() {
                xprintln!("Elements:\n{}" => Color::Green, indent(&tree));
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
use {
    broccli::{Color, xprintln},
    crate::{
        data::string::Str,
        format::{format_tokens, indent},
        initial::initializer::{Initializer, Preference},
        parser::{Element, ElementKind, Parser},
        resolver::Resolver,
        scanner::{Scanner, Token, TokenKind},
        tracker::{Location, Peekable, Span},
        Timer, TIMER,
    },
    super::timer::Duration,
};

pub trait Marked<'marked> {
    fn registry(&self) -> &Registry<'marked>;
    fn registry_mut(&mut self) -> &mut Registry<'marked>;
    fn resolver(&self) -> &Resolver<'marked> {
        &self.registry().resolver
    }
    fn resolver_mut(&mut self) -> &mut Resolver<'marked> {
        &mut self.registry_mut().resolver
    }
}

#[derive(Debug)]
pub struct Registry<'registry> {
    pub resolver: Resolver<'registry>,
}

impl Clone for Registry<'_> {
    fn clone(&self) -> Self {
        Self {
            resolver: self.resolver.clone(),
        }
    }
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

pub trait Stage<'stage, Input, Output> {
    fn execute(&mut self, input: Input) -> Output;
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
        let timer = Timer::new(TIMER);

        let location = {
            let mut initializer = Initializer::new(self.registry.clone(), Location::Flag);
            let result = initializer.execute(());

            self.registry = initializer.registry;
            result
        };

        let tokens = {
            let mut scanner = Scanner::new(self.registry.clone(), location);
            let result = scanner.execute(location);

            self.registry = scanner.registry;
            result
        };

        let elements = {
            let mut parser = Parser::new(self.registry.clone(), location);
            let result = parser.execute(tokens);
            self.registry = parser.registry;
            result
        };

        self.registry.resolver.execute(elements);

        let verbosity = Registry::get_verbosity(&mut self.registry.resolver);

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

    pub fn compile_with<Function, Type>(&mut self, pipeline: Function) -> Type
    where
        Function: FnOnce(&mut Self) -> Type,
    {
        pipeline(self)
    }
}

impl<'initializer> Stage<'initializer, (), Location<'initializer>> for Initializer<'initializer> {
    fn execute(&mut self, _input: ()) -> Location<'initializer> {
        self.initialize();
        let path = Registry::get_path(&mut self.registry.resolver);
        Location::File(Str::from(path))
    }
}

impl<'scanner> Stage<'scanner, Location<'scanner>, Vec<Token<'scanner>>> for Scanner<'scanner> {
    fn execute(&mut self, location: Location<'scanner>) -> Vec<Token<'scanner>> {
        let timer = Timer::new(TIMER);

        let verbosity = Registry::get_verbosity(&mut self.registry.resolver);
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

impl<'parser> Stage<'parser, Vec<Token<'parser>>, Vec<Element<'parser>>> for Parser<'parser> {
    fn execute(&mut self, tokens: Vec<Token<'parser>>) -> Vec<Element<'parser>> {
        let timer = Timer::new(TIMER);

        let verbosity = Registry::get_verbosity(&mut self.registry.resolver);

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

impl<'resolver> Stage<'resolver, Vec<Element<'resolver>>, ()> for Resolver<'resolver> {
    fn execute(&mut self, elements: Vec<Element<'resolver>>) -> () {
        let timer = Timer::new(TIMER);
        let verbosity = Registry::get_verbosity(self);

        if verbosity {
            xprintln!(
                "Started {}." => Color::Blue,
                "`resolving`" => Color::White,
            );
            xprintln!();
        }

        self.process(elements);

        let errors = &self.errors;

        if verbosity {
            let symbols = self.scope.all();

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

        ()
    }
}
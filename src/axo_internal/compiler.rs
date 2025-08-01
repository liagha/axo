use {
    broccli::{xprintln, Color},
    core::time::Duration,

    crate::{
        axo_cursor::{
            Location, Span,
            Peekable
        },
        axo_initial::{
            initializer::{
                Preference,
                Initializer,
            },
        },
        axo_scanner::{
            Scanner,
            Token,
        },
        axo_parser::{
            Element, ElementKind,
            Parser, 
        },
        axo_resolver::{
            Resolver,
        },
        file::{
            read_to_string,
        },
        format_tokens,
        indent,
        Timer, TIMER,
    }
};
use crate::axo_scanner::TokenKind;

pub trait Marked {
    fn registry(&self) -> &Registry;
    fn registry_mut(&mut self) -> &mut Registry;
    fn resolver(&self) -> &Resolver {
        &self.registry().resolver
    }
    fn resolver_mut(&mut self) -> &mut Resolver {
        &mut self.registry_mut().resolver
    }
}

#[derive(Debug)]
pub struct Registry {
    pub resolver: Resolver,
}

impl Registry {
    pub fn new() -> Self {
        Registry {
            resolver: Resolver::new(),
        }
    }

    pub fn get_verbosity(resolver: &mut Resolver) -> Option<bool> {
        let identifier = Element::new(ElementKind::Identifier("Verbosity".to_string()), Span::default(Location::Flag));

        let found = resolver.get(&identifier);

        if let Some(symbol) = found {
            if let Some(preference) = symbol.cast::<Preference>() {
                if let TokenKind::Boolean(verbosity) = preference.value.kind {
                    Some(verbosity)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_path(resolver: &mut Resolver) -> Option<String> {
        let identifier = Element::new(ElementKind::Identifier("Path".to_string()), Span::default(Location::Flag));

        let found = resolver.get(&identifier);

        if let Some(symbol) = found {
            if let Some(preference) = symbol.cast::<Preference>() {
                if let TokenKind::Identifier(path) = preference.value.kind.clone() {
                    Some(path.clone())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub trait Stage<Input, Output> {
    fn execute(&mut self, input: Input) -> Output;
}

pub struct Compiler {
    pub registry: Registry,
}

impl Compiler {
    pub fn new() -> Self {
        let registry = Registry::new();

        Compiler { registry }
    }

    pub fn compile(&mut self) -> () {
        let timer = Timer::new(TIMER);

        let result = self.compile_with(|compiler| {
            let location = {
                let mut initializer = Initializer::new(&mut compiler.registry, Location::Flag);
                initializer.execute(())
            };

            let scanned = {
                let mut scanner = Scanner::new(&mut compiler.registry, location);
                scanner.execute(location)
            };

            let parsed = {
                let mut parser = Parser::new(&mut compiler.registry, location);
                parser.execute(scanned)
            };

            {
                let resolver = &mut compiler.registry.resolver;

                resolver.execute(parsed)
            }
        });

        let verbosity = Registry::get_verbosity(&mut self.registry.resolver).unwrap_or(false);

        if verbosity {
            let duration = Duration::from_nanos(timer.elapsed().unwrap());
            xprintln!(
                "Finished {} {}s." => Color::Green,
                "`compilation` in" => Color::White,
                duration.as_secs_f64(),
            );
            xprintln!();
        }

        result
    }

    pub fn compile_with<Function, Type>(&mut self, pipeline: Function) -> Type
    where
        Function: FnOnce(&mut Self) -> Type,
    {
        let result = pipeline(self);

        result
    }
}

impl<'initializer> Stage<(), Location> for Initializer<'initializer> {
    fn execute(&mut self, _input: ()) -> Location {
        self.initialize();

        Registry::get_path(&mut self.registry.resolver).map_or(Location::Flag, |path| { Location::File(path.leak()) })
    }
}

impl<'scanner> Stage<Location, Vec<Token>> for Scanner<'scanner> {
    fn execute(&mut self, location: Location) -> Vec<Token> {
        let timer = Timer::new(TIMER);

        let verbosity = Registry::get_verbosity(&mut self.registry.resolver).unwrap_or(false);
        self.set_location(location);

        if verbosity {
            xprintln!(
                "Started {}." => Color::Blue,
                "`scanning`" => Color::White,
            );
            xprintln!();
        }

        let content = if let Location::File(path) = location {
            read_to_string(&path).expect("")
        } else {
            "".to_string()
        };

        self.set_input(content);
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

impl<'parser> Stage<Vec<Token>, Vec<Element>> for Parser<'parser> {
    fn execute(&mut self, tokens: Vec<Token>) -> Vec<Element> {
        let timer = Timer::new(TIMER);

        let verbosity = Registry::get_verbosity(&mut self.registry.resolver).unwrap_or(false);

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

impl Stage<Vec<Element>, ()> for Resolver {
    fn execute(&mut self, elements: Vec<Element>) -> () {
        let timer = Timer::new(TIMER);
        let _location = Registry::get_path(self).map_or(Location::Flag, |path| { Location::File(path.leak()) });
        let verbosity = Registry::get_verbosity(self).unwrap_or(false);

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
use {
    broccli::{xprintln, Color},
    core::time::Duration,

    crate::{
        axo_cursor::{
            Location, Span,
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
            Symbol,
            Parser,
        },
        axo_resolver::{
            Resolver,
        },
        file::{
            read_to_string,
        },
        environment,
        format_tokens,
        indent,
        Timer, TIMER,
    }
};

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

#[derive(Clone)]
pub struct Registry {
    pub resolver: Resolver,
}

impl Registry {
    pub fn new() -> Self {
        Registry {
            resolver: Resolver::new(),
        }
    }

    pub fn get_verbosity(&mut self) -> Option<bool> {
        let identifier = Element::new(ElementKind::Identifier("Verbosity".to_string()), Span::default());

        let found = self.resolver.get(&identifier);

        if let Some(symbol) = found {
            if let Some(preference) = symbol.as_any().downcast_ref::<Preference>() {
                if let Preference::Verbosity(verbosity) = preference {
                    Some(*verbosity)
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

    pub fn get_path(&mut self) -> Option<String> {
        let identifier = Element::new(ElementKind::Identifier("Path".to_string()), Span::default());

        let found = self.resolver.get(&identifier);

        if let Some(symbol) = found {
            if let Some(preference) = symbol.as_any().downcast_ref::<Preference>() {
                if let Preference::Path(path) = preference {
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
    fn execute(&mut self, registry: &mut Registry, input: Input) -> Output;
}

macro_rules! pipeline {
    ($registry:expr, $input:expr, $stage:expr) => {{
        $stage.execute($registry, $input)
    }};
    ($registry:expr, $input:expr, $stage:expr, $($remaining:expr),+) => {{
        let output = $stage.execute($registry, $input);
        pipeline!($registry, output, $($remaining),+)
    }};
}

pub struct Compiler {
    pub registry: Registry,
}

impl Compiler {
    pub fn new() -> Self  {
        let registry = Registry::new();

        Compiler { registry }
    }

    pub fn compile(&mut self) -> () {
        let timer = Timer::new(TIMER);

        let result = self.compile_with(|registry| {
            pipeline!(
                registry,
                (),
                Initializing,
                Scanning,
                Parsing,
                Resolving
            )
        });

        let verbosity = self.registry.get_verbosity().unwrap_or(false);

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

    pub fn compile_with<Function, Type>(&mut self, build_pipeline: Function) -> Type
    where
        Function: FnOnce(&mut Registry) -> Type,
    {
        let result = build_pipeline(&mut self.registry);

        result
    }
}

pub struct Initializing;

impl Stage<(), ()> for Initializing {
    fn execute(&mut self, registry: &mut Registry, _input: ()) -> () {
        let timer = Timer::new(TIMER);
        let verbosity = registry.get_verbosity().unwrap_or(false);

        if verbosity {
            xprintln!(
                "Started {}." => Color::Blue,
                "`initializing`" => Color::White,
            );
            xprintln!();
        }

        let input = environment::args().skip(1).collect::<Vec<String>>().join(" ");
        let mut scanner = Scanner::new(registry.clone(), input, Location::Void);
        scanner.scan();
        let mut initializer = Initializer::new(registry.clone(), scanner.output, Location::Void);
        initializer.initialize();

        let preferences = initializer.output.iter().map(|preference| {
            Symbol::new(preference.clone(), Span::default())
        }).collect::<Vec<Symbol>>();

        registry.resolver.extend(preferences);

        if verbosity {
            let preferences_tree = initializer.output
                .iter()
                .map(|preference| format!("{:?}", preference))
                .collect::<Vec<String>>()
                .join("\n");

            if !preferences_tree.is_empty() {
                xprintln!("Preferences:\n{}" => Color::Magenta, indent(&preferences_tree));
                xprintln!();
            }

            if !initializer.errors.is_empty() {
                let duration = Duration::from_nanos(timer.elapsed().unwrap());

                for error in &initializer.errors {
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
                    "`initializing` in" => Color::White,
                    duration.as_secs_f64(),
                    initializer.errors.len() => Color::BrightRed,
                    "errors" => Color::Red,
                );
                xprintln!();
            } else {
                let duration = Duration::from_nanos(timer.elapsed().unwrap());

                xprintln!(
                    "Finished {} {}s." => Color::Green,
                    "`initializing` in" => Color::White,
                    duration.as_secs_f64(),
                );
                xprintln!();
            }
        }

        ()
    }
}

pub struct Scanning;

impl Stage<(), Vec<Token>> for Scanning {
    fn execute(&mut self, registry: &mut Registry, _input: ()) -> Vec<Token> {
        let scanner_timer = Timer::new(TIMER);

        let location = registry.get_path().map_or(Location::Void, |path| { Location::File(path.leak()) });
        let verbosity = registry.get_verbosity().unwrap_or(false);

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

        let mut scanner = Scanner::new(registry.clone(), content, location);
        scanner.scan();

        if verbosity {
            xprintln!("Tokens:\n{}", indent(&format_tokens(&scanner.output)));
            xprintln!();

            if !scanner.errors.is_empty() {
                let duration = Duration::from_nanos(scanner_timer.elapsed().unwrap());

                for error in &scanner.errors {
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
                    scanner.errors.len() => Color::BrightRed,
                    "errors" => Color::Red,
                );
                xprintln!();
            } else {
                let duration = Duration::from_nanos(scanner_timer.elapsed().unwrap());

                xprintln!(
                    "Finished {} {}s." => Color::Green,
                    "`scanning` in" => Color::White,
                    duration.as_secs_f64(),
                );
                xprintln!();
            }
        }

        scanner.output
    }
}

pub struct Parsing;

impl Stage<Vec<Token>, Vec<Element>> for Parsing {
    fn execute(&mut self, registry: &mut Registry, tokens: Vec<Token>) -> Vec<Element> {
        let timer = Timer::new(TIMER);

        let location = registry.get_path().map_or(Location::Void, |path| { Location::File(path.leak()) });

        let verbosity = registry.get_verbosity().unwrap_or(false);

        if verbosity {
            xprintln!(
                "Started {}." => Color::Blue,
                "`parsing`" => Color::White,
            );
            xprintln!();
        }

        let mut parser = Parser::new(registry.clone(), tokens, location);
        parser.parse();

        if verbosity {
            let tree = parser.output
                .iter()
                .map(|element| format!("{:?}", element))
                .collect::<Vec<String>>()
                .join("\n");

            if !tree.is_empty() {
                xprintln!("Elements:\n{}" => Color::Green, indent(&tree));
                xprintln!();
            }

            if !parser.errors.is_empty() {
                let duration = Duration::from_nanos(timer.elapsed().unwrap());

                for error in &parser.errors {
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
                    parser.errors.len() => Color::BrightRed,
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

        parser.output
    }
}

pub struct Resolving;

impl Stage<Vec<Element>, ()> for Resolving {
    fn execute(&mut self, registry: &mut Registry, elements: Vec<Element>) -> () {
        let timer = Timer::new(TIMER);
        let _location = registry.get_path().map_or(Location::Void, |path| { Location::File(path.leak()) });
        let verbosity = registry.get_verbosity().unwrap_or(false);

        if verbosity {
            xprintln!(
                "Started {}." => Color::Blue,
                "`resolving`" => Color::White,
            );
            xprintln!();
        }

        registry.resolver.process(elements);

        let errors = registry.resolver.errors.clone();

        if verbosity {
            let symbols = registry.resolver.scope.all();

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
                for error in &errors {
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
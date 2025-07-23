use {
    broccli::{xprintln, Color},
    hashish::HashSet,

    core::time::Duration,

    crate::{
        axo_cursor::Location,
        axo_initial::{
            initializer::{Initializer, Preference},
        },
        axo_scanner::{
            Scanner,
            Token,
        },
        axo_parser::{
            Element,
            Parser,
        },
        axo_resolver::{
            Resolver,
        },
        file::{
            read_to_string,
            Error,
        },
        format::{
            self,
            Debug, Display,
            Formatter,
        },
        environment,
        format_tokens,
        indent,
        Timer, TIMER,
    }
};

pub trait Marked {
    fn context(&self) -> &Context;
    fn context_mut(&mut self) -> &mut Context;
    fn resolver(&self) -> &Resolver {
        &self.context().resolver
    }
    fn resolver_mut(&mut self) -> &mut Resolver {
        &mut self.context_mut().resolver
    }
}

#[derive(Debug)]
pub enum CompilerError {
    PathRequired,
    FileReadError(Error),
    ArgumentParsing(String),
    HelpRequested,
}

impl Display for CompilerError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> format::Result {
        match self {
            CompilerError::PathRequired => write!(formatter, "No input file specified"),
            CompilerError::FileReadError(error) => write!(formatter, "Failed to read file: {}", error),
            CompilerError::ArgumentParsing(msg) => write!(formatter, "{}", msg),
            CompilerError::HelpRequested => Ok(()),
        }
    }
}

#[derive(Clone)]
pub struct Context {
    pub resolver: Resolver,
    pub preferences: HashSet<Preference>
}

impl Context {
    pub fn new() -> Self {
        Context {
            resolver: Resolver::new(),
            preferences: HashSet::new(),
        }
    }

    pub fn get_verbosity(&self) -> Option<bool> {
        self.preferences
            .iter()
            .find_map(|p| match p {
                Preference::Verbosity(v) => Some(*v),
                _ => None,
            })
    }

    pub fn get_path(&self) -> Option<String> {
        self.preferences
            .iter()
            .find_map(|p| match p {
                Preference::Path(path) => Some(path.clone()),
                _ => None,
            })
    }
}

pub trait Stage<Input, Output> {
    fn execute(&mut self, context: &mut Context, input: Input) -> Result<Output, CompilerError>;
}

macro_rules! pipeline {
    ($context:expr, $input:expr, $stage:expr) => {
        $stage.execute($context, $input)
    };
    ($context:expr, $input:expr, $stage:expr, $($remaining:expr),+) => {
        match $stage.execute($context, $input) {
            Ok(output) => pipeline!($context, output, $($remaining),+),
            Err(error) => Err(error),
        }
    };
}

pub struct Compiler {
    pub context: Context,
}

impl Compiler {
    pub fn new() -> Result<Self, CompilerError> {
        let context = Context::new();

        Ok(Compiler { context })
    }

    pub fn compile(&mut self) -> Result<(), CompilerError> {
        self.compile_with(|context| {
            pipeline!(
                context,
                (),
                Initializing,
                Scanning,
                Parsing,
                Resolving
            ).map(|_| ())
        })
    }

    pub fn compile_with<Function, Type>(&mut self, build_pipeline: Function) -> Result<Type, CompilerError>
    where
        Function: FnOnce(&mut Context) -> Result<Type, CompilerError>,
    {
        build_pipeline(&mut self.context)
    }
}

pub struct Initializing;

impl Stage<(), ()> for Initializing {
    fn execute(&mut self, context: &mut Context, _input: ()) -> Result<(), CompilerError> {
        let input = environment::args().skip(1).collect::<Vec<String>>().join(" ");
        let mut scanner = Scanner::new(context.clone(), input, Location::Void);
        scanner.scan();
        let mut initializer = Initializer::new(context.clone(), scanner.output, Location::Void);
        initializer.initialize();

        context.preferences.extend(initializer.output);

        Ok(())
    }
}

pub struct Scanning;

impl Stage<(), Vec<Token>> for Scanning {
    fn execute(&mut self, context: &mut Context, _input: ()) -> Result<Vec<Token>, CompilerError> {
        let scanner_timer = Timer::new(TIMER);

        let location = context.get_path().map_or(Location::Void, |path| { Location::File(path.leak()) });
        let verbosity = context.get_verbosity().unwrap_or(false);

        let content = if let Location::File(path) = location {
            read_to_string(&path)
                .map_err(CompilerError::FileReadError)?
        } else {
            "".to_string()
        };

        let mut scanner = Scanner::new(context.clone(), content, location);
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

        Ok(scanner.output)
    }
}

pub struct Parsing;

impl Stage<Vec<Token>, Vec<Element>> for Parsing {
    fn execute(&mut self, context: &mut Context, tokens: Vec<Token>) -> Result<Vec<Element>, CompilerError> {
        let timer = Timer::new(TIMER);

        let location = context.get_path().map_or(Location::Void, |path| { Location::File(path.leak()) });

        let verbosity = context.get_verbosity().unwrap_or(false);

        let mut parser = Parser::new(context.clone(), tokens, location);
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

        Ok(parser.output)
    }
}

pub struct Resolving;

impl Stage<Vec<Element>, ()> for Resolving {
    fn execute(&mut self, context: &mut Context, elements: Vec<Element>) -> Result<(), CompilerError> {
        let timer = Timer::new(TIMER);
        let _location = context.get_path().map_or(Location::Void, |path| { Location::File(path.leak()) });
        let verbosity = context.get_verbosity().unwrap_or(false);

        context.resolver.settle(elements);

        let errors = context.resolver.errors.clone();

        if verbosity {
            let symbols = context.resolver.scope.symbols();

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

            for error in &errors {
                let (message, details) = error.format();
                xprintln!(
                        "{}\n{}" => Color::Red,
                        message => Color::Orange,
                        details
                    );
                xprintln!();
            }

            if !errors.is_empty() {
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

        Ok(())
    }
}
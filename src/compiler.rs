use {
    broccli::{xprintln, Color},

    core::time::Duration,

    crate::{
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
        format_tokens,
        indent,
        Timer, TIMERSOURCE,
    }
};
use crate::axo_cursor::Location;

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
    pub verbose: bool,
    pub resolver: Resolver,
    pub location: Location,
}

impl Context {
    pub fn new(location: Location) -> Self {
        Context {
            verbose: false,
            location,
            resolver: Resolver::new(),
        }
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
    pub fn new(path: &'static str, verbose: bool) -> Result<Self, CompilerError> {

        let mut context = Context::new(Location::File(path));
        context.verbose = verbose;

        Ok(Compiler { context })
    }

    pub fn compile(&mut self) -> Result<(), CompilerError> {
        self.compile_with(|context| {
            pipeline!(
                context,
                (),
                Scanning,
                ParserStage,
                ResolverStage
            ).map(|_| ())
        })
    }

    pub fn compile_with<Function, Type>(&mut self, build_pipeline: Function) -> Result<Type, CompilerError>
    where
        Function: FnOnce(&mut Context) -> Result<Type, CompilerError>,
    {
        xprintln!(
            "{} {}" => Color::Blue,
            "Compiling" => Color::Blue,
            self.context.location
        );
        xprintln!();

        build_pipeline(&mut self.context)
    }
}

pub struct Scanning;

impl Stage<(), Vec<Token>> for Scanning {
    fn execute(&mut self, context: &mut Context, _input: ()) -> Result<Vec<Token>, CompilerError> {
        let scanner_timer = Timer::new(TIMERSOURCE);

        let content = if let Location::File(path) = context.location {
            read_to_string(&path)
                .map_err(CompilerError::FileReadError)?
        } else {
            "".to_string()
        };

        let mut scanner = Scanner::new(context.clone(), content, context.location);
        let (tokens, errors) = scanner.scan();

        if context.verbose {
            xprintln!("Tokens:\n{}", indent(&format_tokens(&tokens)));
            xprintln!();

            if !errors.is_empty() {
                let duration = Duration::from_nanos(scanner_timer.elapsed().unwrap());

                for error in &errors {
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
                    errors.len() => Color::BrightRed,
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

        Ok(tokens)
    }
}

pub struct ParserStage;

impl Stage<Vec<Token>, Vec<Element>> for ParserStage {
    fn execute(&mut self, context: &mut Context, tokens: Vec<Token>) -> Result<Vec<Element>, CompilerError> {
        let parser_timer = Timer::new(TIMERSOURCE);

        let mut parser = Parser::new(context.clone(), tokens, context.location);
        let (elements, errors) = parser.parse();

        if context.verbose {
            let tree = elements
                .iter()
                .map(|element| format!("{:?}", element))
                .collect::<Vec<String>>()
                .join("\n");

            if !tree.is_empty() {
                xprintln!("Elements:\n{}" => Color::Green, indent(&tree));
                xprintln!();
            }

            if !errors.is_empty() {
                let duration = Duration::from_nanos(parser_timer.elapsed().unwrap());

                for error in &errors {
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
                    errors.len() => Color::BrightRed,
                    "errors" => Color::Red,
                );
                xprintln!();
            } else {
                let duration = Duration::from_nanos(parser_timer.elapsed().unwrap());

                xprintln!(
                    "Finished {} {}s." => Color::Green,
                    "`parsing` in" => Color::White,
                    duration.as_secs_f64(),
                );
                xprintln!();
            }
        }

        Ok(elements)
    }
}

pub struct ResolverStage;

impl Stage<Vec<Element>, ()> for ResolverStage {
    fn execute(&mut self, context: &mut Context, elements: Vec<Element>) -> Result<(), CompilerError> {
        let resolver_timer = Timer::new(TIMERSOURCE);

        context.resolver.settle(elements);

        let errors = context.resolver.errors.clone();

        if context.verbose {
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
                let duration = Duration::from_nanos(resolver_timer.elapsed().unwrap());

                xprintln!(
                        "Finished {} {}s with {} {}." => Color::Green,
                        "`resolving` in" => Color::White,
                        duration.as_secs_f64(),
                        errors.len() => Color::BrightRed,
                        "errors" => Color::Red,
                    );
                xprintln!();
            } else {
                let duration = Duration::from_nanos(resolver_timer.elapsed().unwrap());

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
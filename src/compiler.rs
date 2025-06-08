use {
    broccli::{xprintln, Color},

    crate::{
        axo_lexer::{
            LexError,
            Lexer,
            Token,
        }, axo_parser::{
            Element,
            ParseError,
            Parser,
        },
        axo_resolver::{
            ResolveError,
            Resolver,
        }, environment::current_dir, file::{
            read_to_string,
            Error,
        }, format::{
            Debug, Display,
            Formatter,
        },
        format_tokens,
        indent,
        Path,
        Peekable,
        Timer, TIMERSOURCE,
    }
};

pub trait Marked {
    fn context(&self) -> &Context;
    fn context_mut(&mut self) -> &mut Context;
}

#[derive(Debug)]
pub enum CompilerError {
    PathRequired,
    FileReadError(Error),
    LexingFailed(Vec<LexError>),
    ParsingFailed(Vec<ParseError>),
    ResolutionFailed(Vec<ResolveError>),
}

impl Display for CompilerError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> crate::format::Result {
        match self {
            CompilerError::PathRequired => write!(formatter, "No input file specified"),
            CompilerError::FileReadError(error) => write!(formatter, "Failed to read file: {}", error),
            CompilerError::LexingFailed(_) => write!(formatter, "Lexing failed with errors"),
            CompilerError::ParsingFailed(_) => write!(formatter, "Parsing failed with errors"),
            CompilerError::ResolutionFailed(_) => write!(formatter, "Resolution failed with errors"),
        }
    }
}

#[derive(Clone)]
pub struct Config {
    pub path: String,
    pub verbose: bool,
}

#[derive(Clone)]
pub struct Context {
    pub depth: usize,
    pub config: Config,
    pub resolver: Resolver,
    pub file_path: Path,
    pub content: String,
}

impl Context {
    pub fn new(config: Config, file_path: Path, content: String) -> Self {
        Context {
            depth: 0,
            config,
            file_path,
            content,
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
    pub fn new(config: Config) -> Result<Self, CompilerError> {
        let file_path = current_dir()
            .map(|mut path| {
                path.push(&config.path);
                path
            })
            .map_err(|error| CompilerError::FileReadError(error))?;

        let content = read_to_string(&file_path)
            .map_err(CompilerError::FileReadError)?;

        let context = Context::new(config, file_path, content);

        Ok(Compiler { context })
    }

    pub fn compile(&mut self) -> Result<(), CompilerError> {
        self.compile_with(|context| {
            pipeline!(
                context,
                (),
                LexerStage,
                ParserStage,
                ResolverStage
            )
        })
    }

    pub fn compile_with<Function, Type>(&mut self, build_pipeline: Function) -> Result<Type, CompilerError>
    where
        Function: FnOnce(&mut Context) -> Result<Type, CompilerError>,
    {
        xprintln!(
            "{} {}" => Color::Blue,
            "Compiling" => Color::Blue,
            self.context.file_path.display()
        );
        xprintln!();

        if self.context.config.verbose {
            xprintln!(
                "File Contents:\n{}" => Color::Magenta,
                indent(&self.context.content) => Color::BrightMagenta
            );
            xprintln!();
        }

        build_pipeline(&mut self.context)
    }
}

pub struct LexerStage;

impl Stage<(), Vec<Token>> for LexerStage {
    fn execute(&mut self, context: &mut Context, _input: ()) -> Result<Vec<Token>, CompilerError> {
        let lexer_timer = Timer::new(TIMERSOURCE);

        let mut lexer = Lexer::new(context.clone(), context.content.clone(), context.file_path.clone());
        let (tokens, errors) = lexer.lex();

        if !errors.is_empty() {
            for error in &errors {
                let (message, details) = error.format();
                xprintln!(
                    "{}\n{}" => Color::Red,
                    message => Color::Orange,
                    details
                );
            }
            xprintln!();
            return Err(CompilerError::LexingFailed(errors));
        }

        if context.config.verbose {
            xprintln!("Tokens:\n{}", indent(&format_tokens(&tokens)));
            xprintln!();
            
            println!(
                "Lexing Took {} ns\n",
                lexer_timer.to_nanoseconds(lexer_timer.elapsed().unwrap())
            );
        }

        Ok(tokens)
    }
}

pub struct ParserStage;

impl Stage<Vec<Token>, Vec<Element>> for ParserStage {
    fn execute(&mut self, context: &mut Context, tokens: Vec<Token>) -> Result<Vec<Element>, CompilerError> {
        let parser_timer = Timer::new(TIMERSOURCE);

        let mut parser = Parser::new(context.clone(), tokens, context.file_path.clone());
        let (elements, errors) = parser.parse();

        if context.config.verbose {
            let tree = elements
                .iter()
                .map(|element| format!("{:?}", element))
                .collect::<Vec<String>>()
                .join("\n");

            xprintln!("Elements:\n{}" => Color::Green, indent(&tree));
            xprintln!();
        }

        for error in &errors {
            let (message, details) = error.format();
            xprintln!(
                "{}\n{}" => Color::Red,
                message => Color::Orange,
                details
            );
        }

        parser.restore();

        if context.config.verbose {
            println!(
                "Parsing Took {} ns\n",
                parser_timer.to_nanoseconds(parser_timer.elapsed().unwrap())
            );
        }

        Ok(elements)
    }
}

pub struct ResolverStage;

impl Stage<Vec<Element>, ()> for ResolverStage {
    fn execute(&mut self, context: &mut Context, elements: Vec<Element>) -> Result<(), CompilerError> {
        let resolver_timer = Timer::new(TIMERSOURCE);

        context.resolver.resolve(elements);

        if !context.resolver.errors.is_empty() {
            for error in &context.resolver.errors {
                let (message, details) = error.format();
                xprintln!(
                    "{}\n{}" => Color::Red,
                    message => Color::Orange,
                    details
                );
            }
            return Err(CompilerError::ResolutionFailed(context.resolver.errors.clone()));
        }

        if context.config.verbose && !context.resolver.scope.all_symbols().is_empty() {
            xprintln!(
                "{}" => Color::Cyan,
                format!("Symbols:\n{:#?}", context.resolver.scope.all_symbols())
            );
        }

        if context.config.verbose {
            println!(
                "Resolution Took {} ns\n",
                resolver_timer.to_nanoseconds(resolver_timer.elapsed().unwrap())
            );
        }

        Ok(())
    }
}


use std::any::Any;
use std::hash::{Hash, Hasher};
use {
    crate::{
        Path, Peekable,
        
        format_tokens, indent, xprintln, Color,

        file::{
            read_to_string,
            Error,
        },
        
        environment::current_dir,
        
        format::{
            Debug, Display,
            Formatter,
        },
        
        axo_lexer::{
            LexError,
            Lexer,
            Token,
        },
        
        axo_parser::{
            ParseError,
            Parser,
            
            Element,
        },
        
        axo_resolver::{
            ResolveError,
            Resolver,
        },
        
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
    pub file_path: String,
    pub verbose: bool,
    pub show_tokens: bool,
    pub show_ast: bool,
    pub time_report: bool,
}

#[derive(Clone)]
pub struct Context {
    pub config: Config,
    pub resolver: Resolver,
    pub file_path: Path,
    pub content: String,
}

impl Context {
    pub fn new(config: Config, file_path: Path, content: String) -> Self {
        Context {
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
                path.push(&config.file_path);
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

        if context.config.show_tokens || context.config.verbose {
            xprintln!("Tokens:\n{}", indent(&format_tokens(&tokens)));
            xprintln!();
        }

        if context.config.time_report {
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

        if context.config.time_report {
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

        if context.config.time_report {
            println!(
                "Resolution Took {} ns\n",
                resolver_timer.to_nanoseconds(resolver_timer.elapsed().unwrap())
            );
        }

        Ok(())
    }
}

pub trait Artifact: Debug + Send + Sync {
    fn clone_box(&self) -> Box<dyn Artifact>;
    fn eq_box(&self, other: &dyn Artifact) -> bool;
    fn hash_box(&self, state: &mut dyn Hasher);
    fn as_any(&self) -> &dyn Any;
}

impl Clone for Box<dyn Artifact> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl PartialEq for Box<dyn Artifact> {
    fn eq(&self, other: &Self) -> bool {
        self.eq_box(other.as_ref())
    }
}

impl Eq for Box<dyn Artifact> {}

impl Hash for Box<dyn Artifact> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash_box(state);
    }
}

impl<T> Artifact for T
where
    T: Debug + Clone + Hash + PartialEq + Eq + Send + Sync + 'static,
{
    fn clone_box(&self) -> Box<dyn Artifact> {
        Box::new(self.clone())
    }

    fn eq_box(&self, other: &dyn Artifact) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<T>() {
            self == other
        } else {
            false
        }
    }

    fn hash_box(&self, mut state: &mut dyn Hasher) {
        self.hash(&mut state);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
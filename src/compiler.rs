use crate::{format_tokens, indent, xprintln, Color, Lexer, Parser, Resolver, Timer, TIMERSOURCE, Path, Peekable};
use crate::any::{Any, TypeId};
use crate::tree::{Node, Tree};
use hashish::HashMap;

#[derive(Debug)]
pub enum CompilerError {
    PathRequired,
    FileReadError(crate::file::Error),
    LexingFailed(Vec<crate::axo_lexer::LexError>),
    ParsingFailed(Vec<crate::axo_parser::ParseError>),
    ResolutionFailed(Vec<crate::axo_resolver::ResolveError>),
}

impl crate::format::Display for CompilerError {
    fn fmt(&self, f: &mut crate::format::Formatter<'_>) -> crate::format::Result {
        match self {
            CompilerError::PathRequired => write!(f, "No input file specified"),
            CompilerError::FileReadError(e) => write!(f, "Failed to read file: {}", e),
            CompilerError::LexingFailed(_) => write!(f, "Lexing failed with errors"),
            CompilerError::ParsingFailed(_) => write!(f, "Parsing failed with errors"),
            CompilerError::ResolutionFailed(_) => write!(f, "Resolution failed with errors"),
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

pub trait StageData: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl StageData for Vec<crate::Token> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl StageData for Vec<crate::axo_parser::Element> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct Context {
    pub config: Config,
    pub resolver: Resolver,
    pub file_path: Path,
    pub content: String,
    data: HashMap<TypeId, Box<dyn StageData>>,
}

impl Context {
    pub fn new(config: Config, file_path: Path, content: String) -> Self {
        Context {
            config,
            file_path,
            content,
            resolver: Resolver::new(),
            data: HashMap::new(),
        }
    }

    pub fn set_data<T: StageData + 'static>(&mut self, data: T) {
        self.data.insert(TypeId::of::<T>(), Box::new(data));
    }

    pub fn get_data<T: StageData + 'static>(&self) -> Option<&T> {
        self.data
            .get(&TypeId::of::<T>())
            .and_then(|data| data.as_any().downcast_ref::<T>())
    }

    pub fn get_data_mut<T: StageData + 'static>(&mut self) -> Option<&mut T> {
        self.data
            .get_mut(&TypeId::of::<T>())
            .and_then(|data| data.as_any_mut().downcast_mut::<T>())
    }
}

pub trait Stage {
    fn entry(&mut self, context: &mut Context) -> Result<(), CompilerError>;
}

pub struct Compiler {
    pub stages: Tree<Box<dyn Stage>>,
    pub context: Context,
}

impl Compiler {
    pub fn new(config: Config) -> Result<Self, CompilerError> {
        let file_path = crate::environment::current_dir()
            .map(|mut path| {
                path.push(&config.file_path);
                path
            })
            .map_err(|e| CompilerError::FileReadError(e))?;

        let content = crate::file::read_to_string(&file_path)
            .map_err(CompilerError::FileReadError)?;

        let context = Context::new(config, file_path, content);

        let mut stages = Tree::new();

        let lexer_stage = Box::new(LexerStage) as Box<dyn Stage>;
        let parser_stage = Box::new(ParserStage) as Box<dyn Stage>;

        let lexer_node = Node::new(lexer_stage);
        let parser_node = Node::new(parser_stage);

        let mut root_node = lexer_node;
        root_node.add_child(parser_node);

        stages.root = Some(root_node);

        Ok(Compiler { stages, context })
    }

    pub fn compile(&mut self) -> Result<(), CompilerError> {
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

        self.execute_stages()?;
        
        self.resolve()
    }

    fn execute_stages(&mut self) -> Result<(), CompilerError> {
        if let Some(mut root) = self.stages.root.take() {
            self.execute_node(&mut root)?;
            self.stages.root = Some(root);
        }
        Ok(())
    }

    fn execute_node(&mut self, node: &mut Node<Box<dyn Stage>>) -> Result<(), CompilerError> {
        node.value.entry(&mut self.context)?;
        for child in &mut node.children {
            self.execute_node(child)?;
        }
        Ok(())
    }

    fn resolve(&mut self) -> Result<(), CompilerError> {
        let resolver_timer = Timer::new(TIMERSOURCE);

        let _elements = self.context
            .get_data::<Vec<crate::axo_parser::Element>>()
            .ok_or_else(|| CompilerError::ResolutionFailed(vec![]))?;

        if !self.context.resolver.errors.is_empty() {
            for err in &self.context.resolver.errors {
                let (msg, details) = err.format();
                xprintln!(
                    "{}\n{}" => Color::Red,
                    msg => Color::Orange,
                    details
                );
            }
            return Err(CompilerError::ResolutionFailed(self.context.resolver.errors.clone()));
        }

        if self.context.config.verbose && !self.context.resolver.scope.all_symbols().is_empty() {
            xprintln!(
                "{}" => Color::Cyan,
                format!("Symbols:\n{:#?}", self.context.resolver.scope.all_symbols())
            );
        }

        if self.context.config.time_report {
            println!(
                "Resolution Took {} ns",
                resolver_timer.to_nanoseconds(resolver_timer.elapsed().unwrap())
            );
        }

        Ok(())
    }
}

pub struct LexerStage;

impl Stage for LexerStage {
    fn entry(&mut self, context: &mut Context) -> Result<(), CompilerError> {
        let lex_timer = Timer::new(TIMERSOURCE);

        let mut lexer = Lexer::new(context.content.clone(), context.file_path.clone());
        let (tokens, errors) = lexer.lex();

        if !errors.is_empty() {
            for err in &errors {
                let (msg, details) = err.format();
                xprintln!(
                    "{}\n{}" => Color::Red,
                    msg => Color::Orange,
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
                "Lexing Took {} ns",
                lex_timer.to_nanoseconds(lex_timer.elapsed().unwrap())
            );
        }

        context.set_data(tokens);
        Ok(())
    }
}

pub struct ParserStage;

impl Stage for ParserStage {
    fn entry(&mut self, context: &mut Context) -> Result<(), CompilerError> {
        let parse_timer = Timer::new(TIMERSOURCE);

        let tokens = context
            .get_data::<Vec<crate::Token>>()
            .ok_or_else(|| CompilerError::ParsingFailed(vec![]))?;

        let mut parser = Parser::new(tokens.clone(), context.file_path.clone());

        let (elements, errors) = parser.parse_program();

        if context.config.verbose {
            let ast = elements
                .iter()
                .map(|element| format!("{:?}", element))
                .collect::<Vec<String>>()
                .join("\n");

            xprintln!("Elements:\n{}" => Color::Green, indent(&ast));
            xprintln!();
        }

        for err in &errors {
            let (msg, details) = err.format();
            xprintln!(
                "{}\n{}" => Color::Red,
                msg => Color::Orange,
                details
            );
        }

        parser.restore();

        if context.config.time_report {
            println!(
                "Parsing Took {} ns",
                parse_timer.to_nanoseconds(parse_timer.elapsed().unwrap())
            );
        }

        context.set_data(elements);

        Ok(())
    }
}
use crate::{format_tokens, indent, xprintln, Color, Lexer, Parser, Resolver, Timer, TIMERSOURCE, Path, Peekable};
use crate::tree::{Tree, Node};
use crate::axo_parser::Element;

#[derive(Debug)]
pub enum CompilerError {
    PathRequired,
    FileReadError(std::io::Error),
    LexingFailed(Vec<crate::axo_lexer::LexError>),
    ParsingFailed(Vec<crate::axo_parser::ParseError>),
    ResolutionFailed(Vec<crate::axo_resolver::ResolveError>),
}

impl std::fmt::Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilerError::PathRequired => write!(f, "No input file specified"),
            CompilerError::FileReadError(e) => write!(f, "Failed to read file: {}", e),
            CompilerError::LexingFailed(_) => write!(f, "Lexing failed with errors"),
            CompilerError::ParsingFailed(_) => write!(f, "Parsing failed with errors"),
            CompilerError::ResolutionFailed(_) => write!(f, "Resolution failed with errors"),
        }
    }
}

impl std::error::Error for CompilerError {}

#[derive(Clone)]
pub struct Config {
    pub file_path: String,
    pub verbose: bool,
    pub show_tokens: bool,
    pub show_ast: bool,
    pub time_report: bool,
}

pub struct CompilerContext {
    pub config: Config,
    pub file_path: Path,
    pub content: String,
    pub tokens: Vec<crate::Token>,
    pub elements: Vec<Element>,
}

pub trait Stage {
    fn entry(&mut self, context: &mut CompilerContext) -> Result<(), CompilerError>;
}

pub struct Compiler {
    pub stages: Tree<Box<dyn Stage>>,
    pub context: CompilerContext,
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

        let context = CompilerContext {
            config,
            file_path,
            content,
            tokens: Vec::new(),
            elements: Vec::new(),
        };

        let mut stages = Tree::new();

        // Build the compilation pipeline as a tree
        let lexer_stage = Box::new(LexerStage) as Box<dyn Stage>;
        let parser_stage = Box::new(ParserStage) as Box<dyn Stage>;
        let resolver_stage = Box::new(ResolverStage) as Box<dyn Stage>;

        let lexer_node = Node::new(lexer_stage);
        let mut parser_node = Node::new(parser_stage);
        parser_node.add_child(Node::new(resolver_stage));

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

        self.execute_stages()
    }

    fn execute_stages(&mut self) -> Result<(), CompilerError> {
        // Take the root node out temporarily to avoid holding a mutable borrow
        if let Some(mut root) = self.stages.root.take() {
            self.execute_node(&mut root)?;
            // Put the root node back
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
}

pub struct LexerStage;

impl Stage for LexerStage {
    fn entry(&mut self, context: &mut CompilerContext) -> Result<(), CompilerError> {
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

        context.tokens = tokens;
        Ok(())
    }
}

pub struct ParserStage;

impl Stage for ParserStage {
    fn entry(&mut self, context: &mut CompilerContext) -> Result<(), CompilerError> {
        let parse_timer = Timer::new(TIMERSOURCE);

        let mut parser = Parser::new(context.tokens.clone(), context.file_path.clone());

        let (test_elements, test_errors) = parser.parse_program();

        if context.config.verbose {
            let test_ast = test_elements
                .iter()
                .map(|element| format!("{:?}", element))
                .collect::<Vec<String>>()
                .join("\n");

            xprintln!("Test Elements:\n{}" => Color::Green, indent(&test_ast));
            xprintln!();
        }

        for err in &test_errors {
            let (msg, details) = err.format();
            xprintln!(
                "{}\n{}" => Color::Red,
                msg => Color::Orange,
                details
            );
        }

        parser.restore();
        
        let elements = parser.parse();

        if !parser.errors.is_empty() {
            for err in &parser.errors {
                xprintln!("{}" => Color::Red, err);
            }
            return Err(CompilerError::ParsingFailed(parser.errors));
        }

        if context.config.show_ast || context.config.verbose {
            let ast = elements
                .iter()
                .map(|element| format!("{:?}", element))
                .collect::<Vec<String>>()
                .join("\n");
            xprintln!("Elements:\n{}" => Color::Green, indent(&ast));
            xprintln!();
        }

        if context.config.time_report {
            println!(
                "Parsing Took {} ns",
                parse_timer.to_nanoseconds(parse_timer.elapsed().unwrap())
            );
        }

        context.elements = elements;
        Ok(())
    }
}

pub struct ResolverStage;

impl Stage for ResolverStage {
    fn entry(&mut self, context: &mut CompilerContext) -> Result<(), CompilerError> {
        let resolver_timer = Timer::new(TIMERSOURCE);

        let mut resolver = Resolver::new();
        resolver.resolve(context.elements.clone());

        if !resolver.errors.is_empty() {
            for err in &resolver.errors {
                let (msg, details) = err.format();
                xprintln!(
                    "{}\n{}" => Color::Red,
                    msg => Color::Orange,
                    details
                );
            }
            return Err(CompilerError::ResolutionFailed(resolver.errors));
        }

        if context.config.verbose && !resolver.scope.all_symbols().is_empty() {
            xprintln!(
                "{}" => Color::Cyan,
                format!("Symbols:\n{:#?}", resolver.scope.all_symbols())
            );
        }

        if context.config.time_report {
            println!(
                "Resolution Took {} ns",
                resolver_timer.to_nanoseconds(resolver_timer.elapsed().unwrap())
            );
        }

        Ok(())
    }
}
mod registry;
mod stages;

use {
    crate::{
        data::*,
        checker::Type,
        analyzer::Analyzable,
        initializer::{
            Initializer,
            InitializeError,
        },
        internal::{
            platform::{create_dir_all, read_dir, PathBuf},
            timer::{DefaultTimer, Duration},
        },
        parser::{
            Element, ElementKind,
            Symbol, SymbolKind,
            ParseError,
        },
        reporter::Reporter,
        resolver::Resolution,
        resolver::{
            Resolver,
            ResolveError,
        },
        scanner::{
            Scanner,
            Token, TokenKind,
            ScanError,
        },
        tracker::{
            Location, Span,
            TrackError,
        },
    },
    broccli::{xprintln, Color},
};

#[cfg(feature = "generator")]
use crate::{
    generator::{Generator, Inkwell},
    internal::driver::Driver,
};
use crate::parser::{Parser, Visibility};

pub trait Stage<'stage, Input, Output> {
    fn execute(&mut self, compiler: &mut Compiler<'stage>, input: Input) -> Output;
}

pub enum CompileError<'error> {
    Initialize(InitializeError<'error>),
    Scan(ScanError<'error>),
    Parse(ParseError<'error>),
    Resolve(ResolveError<'error>),
    Track(TrackError<'error>),
}

pub struct Compiler<'compiler> {
    pub timer: DefaultTimer,
    pub reporter: Reporter,
    pub resolver: Resolver<'compiler>,
    pub errors: Vec<CompileError<'compiler>>,
    #[cfg(feature = "generator")]
    queue: Vec<PathBuf>,
}

impl<'compiler> Compiler<'compiler> {
    pub fn new() -> Self {
        let mut timer = DefaultTimer::new_default();
        let _ = timer.start();

        let resolver = Resolver::new();
        let reporter = Reporter::new(0);

        Compiler {
            timer,
            resolver,
            reporter,
            errors: vec![],
            #[cfg(feature = "generator")]
            queue: Vec::new(),
        }
    }

    pub fn compile(&mut self) {
        let mut initializer = Initializer::new(Location::Flag);
        let targets = initializer.execute(self, ());

        self.errors.extend(
            initializer
                .errors
                .iter()
                .map(|error| {
                    CompileError::Initialize(error.clone())
                })
        );

        let verbosity = Resolver::verbosity(&mut self.resolver);

        self.reporter.verbosity = verbosity;

        for (index, target) in targets.into_iter().enumerate() {
            self.build(target, index);
        }

        let duration = Duration::from_nanos(self.timer.lap().unwrap());

        for error in &self.errors {
            match error {
                CompileError::Initialize(error) => self.reporter.error(&error),
                CompileError::Scan(error) => self.reporter.error(&error),
                CompileError::Parse(error) => self.reporter.error(&error),
                CompileError::Resolve(error) => self.reporter.error(&error),
                CompileError::Track(error) => self.reporter.error(&error),
            }
        }

        self.reporter.finish("compilation", duration);

        #[cfg(feature = "generator")]
        self.run();
    }

    pub fn build(&mut self, target: Location<'compiler>, index: usize) {
        self.resolver.enter();

        let span = match Span::file(Str::from(target.to_string())) {
            Ok(span) => span,
            Err(error) => {
                self.errors.push(CompileError::Track(error));

                return;
            }
        };

        let path = match target.clone().to_path() {
            Ok(path) => {
                path
            },
            Err(error) => {
                self.errors.push(CompileError::Track(error));

                return;
            }
        };

        let name = path.file_name().unwrap().to_str().unwrap();

        if path.is_dir() {
            for entry in read_dir(&path).unwrap() {
                let entry = entry.unwrap();
                let child_path = entry.path();
                let child_loc = Location::file(Str::from(child_path));
                self.build(child_loc, index);
            }
        } else {
            let extension = path.extension().unwrap().to_str().unwrap();

            match extension {
                "ll" => {
                    #[cfg(feature = "generator")]
                    {
                        let executable = Resolver::binary(&mut self.resolver, index);
                        let run = Resolver::run(&mut self.resolver);
                        let (_, binary) = Driver::paths(target, name, None, executable);
                        let should_link = run || executable.is_some();

                        self.reporter.start("linking");

                        let linked = if should_link {
                            match Driver::link(&path, &binary) {
                                Ok(()) => true,
                                Err(error) => {
                                    xprintln!(
                                        "linker error while producing `{}`: {}" => Color::Red,
                                        binary.to_string_lossy(),
                                        error.to_string()
                                    );
                                    xprintln!();
                                    false
                                }
                            }
                        } else {
                            true
                        };

                        if linked {
                            self.reporter.generate("executable", &binary);
                        }

                        if linked && run {
                            self.queue.push(binary);
                        }
                    }
                }

                "axo" => {
                    let tokens = {
                        let mut scanner = Scanner::new(target);
                        scanner.execute(self, target)
                    };

                    let elements = {
                        let mut parser = Parser::new(target);
                        parser.execute(self, tokens)
                    };

                    let mut analysis = self.execute(elements.clone());

                    #[cfg(feature = "generator")]
                    {
                        let context = &inkwell::context::Context::create();
                        let backend = Inkwell::new(Str::from(name), context);

                        let mut generator = Generator::new(backend);

                        let code = Resolver::code(&mut self.resolver, index);
                        let executable = Resolver::binary(&mut self.resolver, index);
                        let run = Resolver::run(&mut self.resolver);

                        let should_link = run || executable.is_some();

                        let (code, binary) = Driver::paths(target, &name, code, executable);
                        let output = Str::from(code.to_string_lossy().to_string());

                        if let Some(parent) = code.parent() {
                            if !parent.as_os_str().is_empty() {
                                if let Err(error) = create_dir_all(parent) {
                                    xprintln!(
                                            "Output directory error for `{}`: {}" => Color::Red,
                                            parent.to_string_lossy(),
                                            error.to_string()
                                        );
                                    xprintln!();
                                }
                            }
                        }

                        let mut module_resolutions = Vec::new();
                        for symbol in self.resolver.scope.all() {
                            if matches!(symbol.kind, SymbolKind::Module(_)) {
                                if let Ok(analysis) =
                                    symbol.analyze(&mut self.resolver)
                                {
                                    module_resolutions.push(Resolution::new(
                                        None,
                                        Type::unit(Span::void()),
                                        analysis,
                                    ));
                                }
                            }
                        }
                        if !module_resolutions.is_empty() {
                            module_resolutions.extend(analysis);
                            analysis = module_resolutions;
                        }

                        generator.execute(
                            &mut self.timer,
                            &self.reporter,
                            analysis,
                            Some(output),
                        );

                        if generator.errors.is_empty() {
                            self.reporter.start("linking");

                            let linked = if should_link {
                                match Driver::link(&code, &binary) {
                                    Ok(()) => true,
                                    Err(error) => {
                                        xprintln!(
                                                "Linker error while producing `{}`: {}" => Color::Red,
                                                binary.to_string_lossy(),
                                                error.to_string()
                                            );
                                        xprintln!();
                                        false
                                    }
                                }
                            } else {
                                true
                            };

                            if linked {
                                self.reporter.generate("IR", &code);
                                self.reporter.generate("executable", &binary);

                                let duration = Duration::from_nanos(self.timer.lap().unwrap());

                                self.reporter.finish("linking", duration);
                            }

                            if linked && run {
                                self.queue.push(binary.clone());
                            }
                        }
                    }
                }

                _ => {}
            }
        }

        let identifier = Element::new(
            ElementKind::Literal(Token::new(
                TokenKind::Identifier(Str::from(name.to_string())),
                span,
            )),
            span,
        );

        let mut module = Symbol::new(
            self.resolver.next_id(),
            SymbolKind::Module(Module::new(Box::new(identifier))),
            span,
            Visibility::Public,
        );

        module.set_scope(self.resolver.scope.clone());
        self.resolver.exit();
        self.resolver.define(module.clone());
    }

    #[cfg(feature = "generator")]
    fn run(&mut self) {
        if self.queue.is_empty() {
            return;
        }

        for binary in self.queue.clone() {
            self.reporter.run(&binary);

            if let Err(error) = Driver::run(&binary) {
                xprintln!(
                    "Run error for `{}`: {}" => Color::Red,
                    binary.to_string_lossy(),
                    error.to_string()
                );
                xprintln!();
            }
        }

        self.queue.clear();
    }

}

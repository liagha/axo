use {
    super::{Compiler, Registry, Stage},
    crate::{
        data::Str,
        internal::{
            platform::create_dir_all,
            platform::PathBuf,
            timer::{DefaultTimer, Duration},
        },
        initializer::{
            Initializer,
        },
        parser::{Element, ElementKind, Symbol, SymbolKind},
        reporter::{Reporter},
        resolver::{
            analyzer::{symbol as analyze_symbol, Analyzer},
            checker::Type,
            Resolution,
        },
        scanner::{Token, TokenKind},
        schema::*,
        tracker::{Location, Span},
    },
    broccli::{xprintln, Color},
};

#[cfg(feature = "generator")]
use {
    crate::generator::{Backend, Generator},
    crate::internal::driver::Driver,
};

impl<'compiler> Compiler<'compiler> {
    pub fn new() -> Self {
        let timer = DefaultTimer::new_default();
        let registry = Registry::new();
        let reporter = Reporter::new(false);

        Compiler {
            timer,
            registry,
            reporter,
            #[cfg(feature = "generator")]
            queue: Vec::new(),
        }
    }

    pub fn compile(&mut self) {
        let mut timer = DefaultTimer::new_default();
        let _ = timer.start();

        let verbosity = self.pipeline();

        if verbosity {
            let duration = Duration::from_nanos(timer.elapsed().unwrap());
            xprintln!(
                "Finished {} {}s." => Color::Green,
                "`compilation` in" => Color::White,
                duration.as_secs_f64(),
            );
            xprintln!();
        }

        #[cfg(feature = "generator")]
        self.run_queue(verbosity);
    }

    #[cfg(feature = "generator")]
    fn run_queue(&mut self, verbosity: bool) {
        if self.queue.is_empty() {
            return;
        }

        for binary in self.queue.clone() {
            if verbosity {
                xprintln!(
                    "Running {}." => Color::Blue,
                    format!("`{}`", binary.to_string_lossy()) => Color::White
                );
                xprintln!();
            }

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

    #[cfg(feature = "generator")]
    fn compile_ir_input(
        &mut self,
        target: Location<'compiler>,
        index: usize,
        name: &str,
        verbosity: bool,
    ) {
        let source = match target {
            Location::File(path) => PathBuf::from(path.as_str().unwrap_or("")),
            _ => return,
        };

        let executable = Registry::binary(&mut self.registry.resolver, index);
        let bootstrap = Registry::bootstrap(&mut self.registry.resolver);
        let run = Registry::run(&mut self.registry.resolver);
        let (_, binary) = Driver::paths(target, name, None, executable);
        let should_link = run || executable.is_some();

        if verbosity {
            xprintln!(
                "Started {}." => Color::Blue,
                "`linking`" => Color::White
            );
            xprintln!();
        }

        let linked = if should_link {
            match Driver::link(&source, &binary, bootstrap) {
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

        if linked && verbosity {
            xprintln!(
                "Generated {} {}." => Color::Green,
                "(executable)" => Color::White,
                format!("`{}`", binary.to_string_lossy()) => Color::White
            );
            xprintln!();
        }

        if linked && run {
            self.queue.push(binary);
        }
    }

    pub fn module(&mut self, target: Location<'compiler>, index: usize) -> Symbol<'compiler> {
        self.registry.resolver.enter();

        let name = target.name();
        let span = Span::file(Str::from(target.to_string()));
        let verbosity = Registry::verbosity(&mut self.registry.resolver);

        if target.is_ir() {
            #[cfg(feature = "generator")]
            self.compile_ir_input(target, index, &name, verbosity);

            let identifier = Element::new(
                ElementKind::Literal(Token::new(TokenKind::Identifier(Str::from(name)), span)),
                span,
            );

            let mut module = Symbol::new(
                SymbolKind::Module(Module::new(Box::new(identifier))),
                span,
                self.registry.resolver.next_id(),
            );

            module.set_scope(self.registry.resolver.scope.clone());
            self.registry.resolver.exit();
            self.registry.resolver.define(module.clone());
            return module;
        }

        let tokens = {
            let mut scanner = crate::scanner::Scanner::new(target);
            scanner.execute(&mut self, target)
        };

        let elements = {
            let mut parser = crate::parser::Parser::new(target);
            parser.execute(&mut self, tokens)
        };

        let mut analysis = Vec::new();

        let faulty = if scan_count == 0 && parse_count == 0 {
            let prior = self.registry.resolver.errors.len();
            analysis = self.registry.resolver.execute(self, elements.clone());
            self.registry.resolver.errors.len() > prior
        } else {
            true
        };

        #[cfg(feature = "generator")]
        {
            if !faulty {
                let context = &inkwell::context::Context::create();
                let bootstrap = Registry::bootstrap(&mut self.registry.resolver);
                let backend = crate::generator::Inkwell::new(Str::from(name.clone()), context)
                    .with_bootstrap(bootstrap);
                let mut generator = Generator::new(backend);
                let code = Registry::code(&mut self.registry.resolver, index);
                let executable = Registry::binary(&mut self.registry.resolver, index);
                let run = Registry::run(&mut self.registry.resolver);
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
                for symbol in self.registry.resolver.scope.all() {
                    if matches!(symbol.kind, SymbolKind::Module(_)) {
                        if let Ok(analysis) =
                            analyze_symbol(&symbol, &self.registry.resolver, Analyzer::root())
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

                generator.execute(self, analysis, Some(output));

                if generator.errors.is_empty() {
                    let mut link_timer = DefaultTimer::new_default();
                    link_timer.start();
                    if verbosity {
                        xprintln!(
                            "Started {}." => Color::Blue,
                            "`linking`" => Color::White
                        );
                        xprintln!();
                    }
                    let linked = if should_link {
                        match Driver::link(&code, &binary, bootstrap) {
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

                    if linked && verbosity {
                        xprintln!(
                            "Generated {} {}." => Color::Green,
                            "(IR)" => Color::White,
                            format!("`{}`", code.to_string_lossy()) => Color::White
                        );
                        xprintln!();

                        xprintln!(
                            "Generated {} {}." => Color::Green,
                            "(executable)" => Color::White,
                            format!("`{}`", binary.to_string_lossy()) => Color::White
                        );
                        xprintln!();

                        let duration = Duration::from_nanos(link_timer.elapsed().unwrap());
                        xprintln!(
                            "Finished {} in {}s." => Color::Green,
                            "`linking`" => Color::White,
                            duration.as_secs_f64(),
                        );
                        xprintln!();
                    }

                    if linked && run {
                        self.queue.push(binary.clone());
                    }
                }
            }
        }

        let identifier = Element::new(
            ElementKind::Literal(Token::new(TokenKind::Identifier(Str::from(name)), span)),
            span,
        );

        let mut module = Symbol::new(
            SymbolKind::Module(Module::new(Box::new(identifier))),
            span,
            self.registry.resolver.next_id(),
        );

        module.set_scope(self.registry.resolver.scope.clone());

        self.registry.resolver.exit();
        self.registry.resolver.define(module.clone());

        module
    }

    fn pipeline(&mut self) -> bool {
        let targets = {
            let mut initializer = Initializer::new(Location::Flag);
            initializer.execute(self, ())
        };

        let verbosity = Registry::verbosity(&mut self.registry.resolver);
        let mut logger = Reporter::new(verbosity);

        for (index, target) in targets.into_iter().enumerate() {
            logger.set_current(target.to_string());
            self.module(target, index);
            logger.clear_current();
        }

        verbosity
    }

    pub fn with<Function, Type>(&mut self, pipeline: Function) -> Type
    where
        Function: FnOnce(&mut Self) -> Type,
    {
        pipeline(self)
    }
}

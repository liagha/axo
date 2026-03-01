use {
    super::{Compiler, Resolver, Stage},
    crate::{
        data::Str,
        initializer::Initializer,
        internal::{
            platform::create_dir_all,
            platform::PathBuf,
            timer::{DefaultTimer, Duration},
        },
        parser::{Element, ElementKind, Symbol, SymbolKind},
        reporter::Reporter,
        resolver::Resolution,
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
use crate::analyzer::{symbol as analyze_symbol, Analyzer};
use crate::checker::Type;

impl<'compiler> Compiler<'compiler> {
    pub fn new() -> Self {
        let mut timer = DefaultTimer::new_default();
        timer.start();

        let resolver = Resolver::new();
        let reporter = Reporter::new(0);

        Compiler {
            timer,
            resolver,
            reporter,
            #[cfg(feature = "generator")]
            queue: Vec::new(),
        }
    }

    pub fn compile(&mut self) {
        self.pipeline();

        let duration = Duration::from_nanos(self.timer.lap().unwrap());

        self.reporter.finish("compilation", duration);

        #[cfg(feature = "generator")]
        self.run_queue();
    }

    #[cfg(feature = "generator")]
    fn run_queue(&mut self) {
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

    #[cfg(feature = "generator")]
    fn compile_ir_input(
        &mut self,
        target: Location<'compiler>,
        index: usize,
        name: &str,
    ) {
        let source = match target {
            Location::File(path) => PathBuf::from(path.as_str().unwrap_or("")),
            _ => return,
        };

        let executable = Resolver::binary(&mut self.resolver, index);
        let bootstrap = Resolver::bootstrap(&mut self.resolver);
        let run = Resolver::run(&mut self.resolver);
        let (_, binary) = Driver::paths(target, name, None, executable);
        let should_link = run || executable.is_some();

        self.reporter.start("linking");

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

        if linked {
            self.reporter.generate("executable", &binary);
        }

        if linked && run {
            self.queue.push(binary);
        }
    }

    pub fn module(&mut self, target: Location<'compiler>, index: usize) -> Symbol<'compiler> {
        self.resolver.enter();

        let context = &inkwell::context::Context::create();

        let name = target.name();
        let span = Span::file(Str::from(target.to_string()));
        let verbosity = Resolver::verbosity(&mut self.resolver);

        if target.has_extension("ll") {
            #[cfg(feature = "generator")]
            self.compile_ir_input(target, index, &name);

            let identifier = Element::new(
                ElementKind::Literal(Token::new(TokenKind::Identifier(Str::from(name)), span)),
                span,
            );

            let mut module = Symbol::new(
                SymbolKind::Module(Module::new(Box::new(identifier))),
                span,
                self.resolver.next_id(),
            );

            module.set_scope(self.resolver.scope.clone());
            self.resolver.exit();
            self.resolver.define(module.clone());
            return module;
        }

        let (tokens, scan_errors) = {
            let mut scanner = crate::scanner::Scanner::new(target);
            scanner.execute(self, target)
        };

        let (elements, parse_errors) = {
            let mut parser = crate::parser::Parser::new(target);
            parser.execute(self, tokens)
        };

        let mut analysis = Vec::new();

        let faulty = if scan_errors.len() == 0 && parse_errors.len() == 0 {
            let prior = self.resolver.errors.len();
            analysis = self.execute(elements.clone());
            self.resolver.errors.len() > prior
        } else {
            true
        };

        #[cfg(feature = "generator")]
        {
            if !faulty {
                let bootstrap = Resolver::bootstrap(&mut self.resolver);
                let backend = crate::generator::Inkwell::new(Str::from(name.clone()), context)
                    .with_bootstrap(bootstrap);
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
                            analyze_symbol(&symbol, &self.resolver, Analyzer::root())
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

                generator.execute(&mut self.timer, &self.reporter, analysis, Some(output));

                if generator.errors.is_empty() {
                    self.reporter.start("linking");

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

        let identifier = Element::new(
            ElementKind::Literal(Token::new(TokenKind::Identifier(Str::from(name)), span)),
            span,
        );

        let mut module = Symbol::new(
            SymbolKind::Module(Module::new(Box::new(identifier))),
            span,
            self.resolver.next_id(),
        );

        module.set_scope(self.resolver.scope.clone());

        self.resolver.exit();
        self.resolver.define(module.clone());

        module
    }

    fn pipeline(&mut self) {
        let (targets, initial_errors) = {
            let mut initializer = Initializer::new(Location::Flag);
            initializer.execute(self, ())
        };

        let verbosity = Resolver::verbosity(&mut self.resolver);

        self.reporter.verbosity = verbosity;

        let mut logger = Reporter::new(verbosity);

        for (index, target) in targets.into_iter().enumerate() {
            self.module(target, index);
        }
    }
}

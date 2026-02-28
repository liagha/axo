use {
    super::{Registry, Resolver, Stage},
    crate::{
        data::Str,
        generator::Backend,
        internal::{
            compiler::Compiler,
            timer::{DefaultTimer, Duration},
        },
        parser::{
            Element,
            Symbol, SymbolKind,
            Parser
        },
        reporter::Reporter,
        resolver::Resolution,
        initializer::{Initializer},
        scanner::{Scanner, Token},
        tracker::{Location, Peekable, Position, Spanned},
    },
};

impl<'initializer> Stage<'initializer, (), Vec<Location<'initializer>>>
    for Initializer<'initializer>
{
    fn execute(
        &mut self,
        compiler: &mut Compiler<'initializer>,
        _input: (),
    ) -> Vec<Location<'initializer>> {
        let mut timer = DefaultTimer::new_default();
        timer.start();

        let verbosity = Registry::verbosity(&mut compiler.registry.resolver);
        let logger = Reporter::new(verbosity);

        logger.start("initializing");

        let targets = self.initialize();

        let preferences = self
            .output
            .clone()
            .into_iter()
            .map(|preference| {
                let span = preference.borrow_span();

                Symbol::new(
                    SymbolKind::Preference(preference),
                    span,
                    compiler.registry.resolver.next_id(),
                )
            })
            .collect::<Vec<Symbol>>();

        compiler.registry.resolver.scope.extend(preferences);

        let duration = Duration::from_nanos(timer.elapsed().unwrap());
        logger.finish("initializing", duration, 0);

        targets
    }
}

impl<'scanner> Stage<'scanner, Location<'scanner>, Vec<Token<'scanner>>> for Scanner<'scanner> {
    fn execute(
        &mut self,
        compiler: &mut Compiler<'scanner>,
        location: Location<'scanner>,
    ) -> Vec<Token<'scanner>> {
        let mut timer = DefaultTimer::new_default();
        timer.start();

        self.set_location(location);
        compiler.reporter.start("scanning");

        match location.get_value() {
            Ok(content) => {
                let characters =
                    Scanner::inspect(Position::new(location), content.chars().collect::<Vec<_>>());
                self.set_input(characters);

                self.scan();

                compiler.reporter.tokens(&self.output);
                compiler.reporter.errors(&self.errors);

                let duration = Duration::from_nanos(timer.elapsed().unwrap());
                compiler.reporter.finish("scanning", duration, self.errors.len());

                self.output.clone()
            }
            
            Err(error) => {
                let kind = crate::scanner::ErrorKind::Tracking(error.clone());
                let error = crate::scanner::ScanError::new(kind, error.span);

                self.errors.push(error);

                Vec::new()
            }
        }
    }
}

impl<'parser> Stage<'parser, Vec<Token<'parser>>, Vec<Element<'parser>>> for Parser<'parser> {
    fn execute(
        &mut self,
        compiler: &mut Compiler<'parser>,
        tokens: Vec<Token<'parser>>,
    ) -> Vec<Element<'parser>> {
        let mut timer = DefaultTimer::new_default();
        _ = timer.start();

        compiler.reporter.start("parsing");

        self.set_input(tokens);
        self.parse();

        compiler.reporter.elements(&self.output);
        compiler.reporter.errors(&self.errors);

        let duration = Duration::from_nanos(timer.elapsed().unwrap());
        compiler.reporter.finish("parsing", duration, self.errors.len());

        self.output.clone()
    }
}

impl<'resolver> Stage<'resolver, Vec<Element<'resolver>>, Vec<Resolution<'resolver>>> for Resolver<'resolver> {
    fn execute(
        &mut self,
        compiler: &mut Compiler<'resolver>,
        elements: Vec<Element<'resolver>>,
    ) -> Vec<Resolution<'resolver>> {
        let mut timer = DefaultTimer::new_default();
        timer.start();

        compiler.reporter.start("resolving");

        self.symbols.clear();
        self.with_input(elements);

        let resolutions = self.process();

        let scope_symbols = self.scope.all();
        for symbol in scope_symbols {
            if !self
                .symbols
                .iter()
                .any(|(item, _)| item.brand() == symbol.brand())
            {
                self.symbols.push((symbol, None));
            }
        }

        compiler.reporter.symbols(&self.symbols);
        compiler.reporter.resolutions(&*resolutions);

        compiler.reporter.errors(self.errors.as_slice());

        let duration = Duration::from_nanos(timer.elapsed().unwrap());
        compiler.reporter.finish("resolving", duration, self.errors.len());

        resolutions
    }
}

#[cfg(feature = "generator")]
impl<'resolver, B: Backend<'resolver>> crate::generator::Generator<'resolver, B> {
    pub fn execute(
        &mut self,
        compiler: &mut Compiler<'resolver>,
        resolutions: Vec<Resolution<'resolver>>,
        output: Option<Str<'resolver>>,
    ) -> () {
        let mut timer = DefaultTimer::new_default();
        timer.start();

        compiler.reporter.start("generating");

        self.backend.generate(
            resolutions
                .iter()
                .map(|resolution| resolution.analysis.clone())
                .collect::<Vec<_>>(),
        );

        self.errors.extend(self.backend.take_errors());

        if let Some(output) = output {
            let path = output.as_str().unwrap_or("");
            if !path.is_empty() {
                if let Err(error) = self.backend.write(path) {
                    self.errors.push(crate::generator::GenerateError::new(
                        crate::generator::ErrorKind::OutputWriteFailure {
                            path: path.to_string(),
                            reason: error.to_string(),
                        },
                        crate::tracker::Span::void(),
                    ));
                }
            }
        }

        compiler.reporter.errors(self.errors.as_slice());

        let duration = Duration::from_nanos(timer.elapsed().unwrap());
        compiler.reporter.finish("generating", duration, self.errors.len());
    }
}

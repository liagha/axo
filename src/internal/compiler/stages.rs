use {
    super::{Resolver, Stage},
    crate::{
        data::Str,
        generator::Backend,
        initializer::{
            Initializer,
            InitialError,
        },
        internal::{
            compiler::Compiler,
            timer::{DefaultTimer, Duration},
        },
        parser::{
            Parser,
            ParseError,
            Element,
            Symbol,
            SymbolKind
        },
        reporter::Reporter,
        resolver::Resolution,
        scanner::{
            Scanner,
            Token,
            ErrorKind, 
            ScanError,
        },
        tracker::{Location, Peekable, Position, Spanned},
    },
};

impl<'initializer> Stage<'initializer, (), (Vec<Location<'initializer>>, Vec<InitialError<'initializer>>)>
    for Initializer<'initializer>
{
    fn execute(
        &mut self,
        compiler: &mut Compiler<'initializer>,
        _input: (),
    ) -> (Vec<Location<'initializer>>, Vec<InitialError<'initializer>>) {
        let verbosity = Resolver::verbosity(&mut compiler.resolver);
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
                    compiler.resolver.next_id(),
                )
            })
            .collect::<Vec<Symbol>>();

        compiler.resolver.scope.extend(preferences);

        let duration = Duration::from_nanos(compiler.timer.lap().unwrap());
        
        logger.finish("initializing", duration);

        (targets, self.errors.clone())
    }
}

impl<'scanner> Stage<'scanner, Location<'scanner>, (Vec<Token<'scanner>>, Vec<ScanError<'scanner>>)> for Scanner<'scanner> {
    fn execute(
        &mut self,
        compiler: &mut Compiler<'scanner>,
        location: Location<'scanner>,
    ) -> (Vec<Token<'scanner>>, Vec<ScanError<'scanner>>) 
    {
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

                let duration = Duration::from_nanos(compiler.timer.lap().unwrap());
                
                compiler
                    .reporter
                    .finish("scanning", duration);

                (self.output.clone(), self.errors.clone())
            }

            Err(error) => {
                let kind = ErrorKind::Tracking(error.clone());
                let error = ScanError::new(kind, error.span);

                self.errors.push(error);

                (Vec::new(), self.errors.clone())
            }
        }
    }
}

impl<'parser> Stage<'parser, Vec<Token<'parser>>, (Vec<Element<'parser>>, Vec<ParseError<'parser>>)> for Parser<'parser> {
    fn execute(
        &mut self,
        compiler: &mut Compiler<'parser>,
        tokens: Vec<Token<'parser>>,
    ) -> (Vec<Element<'parser>>, Vec<ParseError<'parser>>) {
        compiler.reporter.start("parsing");

        self.set_input(tokens);
        self.parse();

        compiler.reporter.elements(&self.output);
        compiler.reporter.errors(&self.errors);

        let duration = Duration::from_nanos(compiler.timer.lap().unwrap());
        
        compiler
            .reporter
            .finish("parsing", duration);

        (self.output.clone(), self.errors.clone())
    }
}

impl<'resolver> Compiler<'resolver>
{
    pub fn execute(
        &mut self,
        elements: Vec<Element<'resolver>>,
    ) -> Vec<Resolution<'resolver>> {
        self.reporter.start("resolving");

        self.resolver.symbols.clear();
        self.resolver.with_input(elements);

        let resolutions = self.resolver.process();

        let scope_symbols = self.resolver.scope.all();
        for symbol in scope_symbols {
            if !self.resolver
                .symbols
                .iter()
                .any(|(item, _)| item.brand() == symbol.brand())
            {
                self.resolver.symbols.push((symbol, None));
            }
        }

        self.reporter.symbols(&self.resolver.symbols);
        self.reporter.resolutions(&*resolutions);

        self.reporter.errors(self.resolver.errors.as_slice());

        let duration = Duration::from_nanos(self.timer.lap().unwrap());

        self.reporter
            .finish("resolving", duration);

        resolutions
    }
}

#[cfg(feature = "generator")]
impl<'resolver, B: Backend<'resolver>> crate::generator::Generator<'resolver, B> {
    pub fn execute(
        &mut self,
        timer: &mut DefaultTimer,
        reporter: &Reporter,
        resolutions: Vec<Resolution<'resolver>>,
        output: Option<Str<'resolver>>,
    ) -> () {
        reporter.start("generating");

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

        reporter.errors(self.errors.as_slice());

        let duration = Duration::from_nanos(timer.lap().unwrap());
        
        reporter
            .finish("generating", duration);
    }
}

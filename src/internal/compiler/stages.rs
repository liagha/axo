use {
    super::{Resolver, Stage},
    crate::{
        data::Str,
        generator::Backend,
        initializer::{
            Initializer,
        },
        internal::{
            compiler::Compiler,
            timer::{DefaultTimer, Duration},
        },
        parser::{
            Parser,
            Element,
            Symbol,
            SymbolKind
        },
        reporter::Reporter,
        resolver::Resolution,
        scanner::{
            Scanner,
            Token,
        },
        tracker::{Location, Peekable, Spanned},
    },
};
use crate::internal::compiler::CompileError;
use crate::parser::Visibility;

impl<'initializer> Stage<'initializer, (), Vec<Location<'initializer>>>
    for Initializer<'initializer>
{
    fn execute(
        &mut self,
        compiler: &mut Compiler<'initializer>,
        _input: (),
    ) -> Vec<Location<'initializer>> {
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
                    compiler.resolver.next_id(),
                    SymbolKind::Preference(preference),
                    span,
                    Visibility::Public,
                )
            })
            .collect::<Vec<Symbol>>();

        compiler.resolver.scope.extend(preferences);

        let duration = Duration::from_nanos(compiler.timer.lap().unwrap());

        logger.finish("initializing", duration);

        targets
    }
}

impl<'scanner> Stage<'scanner, Location<'scanner>, Vec<Token<'scanner>>> for Scanner<'scanner> {
    fn execute(
        &mut self,
        compiler: &mut Compiler<'scanner>,
        location: Location<'scanner>,
    ) -> Vec<Token<'scanner>>
    {
        self.set_location(location);
        compiler.reporter.start("scanning");

        self.prepare();
        self.scan();

        compiler.reporter.tokens(&self.output);

        let duration = Duration::from_nanos(compiler.timer.lap().unwrap());

        compiler
            .reporter
            .finish("scanning", duration);

        compiler.errors.extend(
            self
                .errors
                .iter()
                .map(|error| {
                    CompileError::Scan(error.clone())
                })
        );

        self.output.clone()
    }
}

impl<'parser> Stage<'parser, Vec<Token<'parser>>, Vec<Element<'parser>>> for Parser<'parser> {
    fn execute(
        &mut self,
        compiler: &mut Compiler<'parser>,
        tokens: Vec<Token<'parser>>,
    ) -> Vec<Element<'parser>> {
        compiler.reporter.start("parsing");

        self.set_input(tokens);
        self.parse();

        compiler.reporter.elements(&self.output);

        let duration = Duration::from_nanos(compiler.timer.lap().unwrap());

        compiler
            .reporter
            .finish("parsing", duration);

        compiler.errors.extend(
            self
                .errors
                .iter()
                .map(|error| {
                    CompileError::Parse(error.clone())
                })
        );

        self.output.clone()
    }
}

impl<'resolver> Compiler<'resolver>
{
    pub fn execute(
        &mut self,
        elements: Vec<Element<'resolver>>,
    ) -> Vec<Resolution<'resolver>> {
        self.reporter.start("resolving");

        self.resolver.with_input(elements);

        self.resolver.resolve();

        self.reporter.symbols(&self.resolver.scope.all());
        self.reporter.resolutions(&*self.resolver.output);

        let duration = Duration::from_nanos(self.timer.lap().unwrap());

        self.reporter
            .finish("resolving", duration);

        self.errors.extend(
            self.resolver
                .errors
                .iter()
                .map(|error| {
                    CompileError::Resolve(error.clone())
                })
        );

        self.resolver.output.clone()
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

mod registry;
mod runner;
mod stages;

use crate::{
    initializer::Initializer,
    internal::{logger::Logger, platform::PathBuf, timer::DefaultTimer},
    reporter::Reporter,
    resolver::Resolver,
};

pub trait Stage<'stage, Input, Output> {
    fn execute(&mut self, compiler: &mut Compiler<'stage>, input: Input) -> Output;
}

pub struct Compiler<'compiler> {
    pub timer: DefaultTimer,
    pub reporter: Reporter,
    pub resolver: Resolver<'compiler>,
    #[cfg(feature = "generator")]
    queue: Vec<PathBuf>,
}

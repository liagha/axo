mod registry;
mod runner;
mod stages;

use {
    crate::{
        initializer::Initializer, 
        internal::{
            platform::PathBuf,
            logger::Logger,
            timer::DefaultTimer,
        }, 
        reporter::{
            Reporter,  
        },
        resolver::{
            Resolver,
        },
    },
};

pub trait Stage<'stage, Input, Output> {
    fn execute(&mut self, compiler: &mut Compiler<'stage>, input: Input) -> Output;
}

#[derive(Debug)]
pub struct Registry<'registry> {
    pub resolver: Resolver<'registry>,
}

pub struct Compiler<'compiler> {
    pub timer: DefaultTimer,
    pub reporter: Reporter,
    pub registry: Registry<'compiler>,
    #[cfg(feature = "generator")]
    queue: Vec<PathBuf>,
}
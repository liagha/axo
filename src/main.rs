#![allow(unused)]

#[cfg(feature = "internal")]
pub mod internal;
#[cfg(feature = "formation")]
pub mod formation;
#[cfg(feature = "initial")]
pub mod initial;
#[cfg(feature = "scanner")]
pub mod scanner;
#[cfg(feature = "parser")]
pub mod parser;
#[cfg(feature = "resolver")]
pub mod resolver;
#[cfg(feature = "generator")]
pub mod generator;

#[cfg(feature = "internal")]
mod format;

#[cfg(feature = "internal")]
pub mod reporter;

#[cfg(feature = "internal")]
pub mod schema;

#[cfg(feature = "internal")]
pub mod text;

#[cfg(feature = "internal")]
pub mod tracker;

#[cfg(feature = "internal")]
pub mod data;
mod interpreter;

fn main() {
    #[cfg(feature = "internal")]
    {
        use {
            log::Level,
            data::Str,
            internal::{
                logger::{LogInfo, LogPlan, Logger},
                compiler::Compiler,
            },
        };

        let plan = LogPlan::new(vec![LogInfo::Time, LogInfo::Level, LogInfo::Message]).with_separator(Str::from(" "));

        let logger = Logger::new(Level::max(), plan);
        logger.init().expect("fuck");

        let mut compiler = Compiler::new();

        compiler.compile();
    }
}
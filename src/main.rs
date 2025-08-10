#![allow(unused)]

#[cfg(feature = "checker")]
pub mod checker;
#[cfg(feature = "formation")]
pub mod formation;
#[cfg(feature = "initial")]
pub mod initial;
#[cfg(feature = "internal")]
pub mod internal;
#[cfg(feature = "parser")]
pub mod parser;
#[cfg(feature = "resolver")]
pub mod resolver;
#[cfg(feature = "scanner")]
pub mod scanner;
#[cfg(feature = "generator")]
pub mod generator;
#[cfg(feature = "text")]
pub mod text;

mod format;
pub mod reporter;
pub mod schema;
pub mod tracker;
pub(crate) mod data;

use {
    internal::{
        compiler::{
            Compiler,
        },
        logger::{LogInfo, LogPlan, Logger},
    },
    log::Level,
};

fn main() {
    let plan = LogPlan::new(vec![LogInfo::Time, LogInfo::Level, LogInfo::Message]) .with_separator(" ".to_string());

    let logger = Logger::new(Level::max(), plan);
    logger.init().expect("fuck");

    let mut compiler = Compiler::new();

    compiler.compile();
}
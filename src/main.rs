#![allow(unused)]

#[cfg(feature = "checker")]
mod checker;
mod tracker;
mod data;
mod reporter;
#[cfg(feature = "formation")]
mod formation;
mod format;
#[cfg(feature = "initial")]
mod initial;
#[cfg(feature = "internal")]
mod internal;
#[cfg(feature = "parser")]
mod parser;
#[cfg(feature = "resolver")]
mod resolver;
#[cfg(feature = "scanner")]
mod scanner;
mod schema;
#[cfg(feature = "text")]
mod text;

use {
    internal::{
        compiler::{
            Compiler,
        },
        logger::{LogInfo, LogPlan, Logger},
        timer::{
            Timer,
        },
    },
    log::Level,
};

#[cfg(target_arch = "x86_64")]
pub const TIMER: internal::timer::CPUCycleSource = internal::timer::CPUCycleSource;

#[cfg(target_arch = "aarch64")]
pub const TIMER: internal::timer::ARMGenericTimerSource = internal::timer::ARMGenericTimerSource;

fn main() {
    let plan = LogPlan::new(vec![LogInfo::Time, LogInfo::Level, LogInfo::Message]) .with_separator(" ".to_string());

    let logger = Logger::new(Level::max(), plan);
    logger.init().expect("fuck");

    let mut compiler = Compiler::new();

    compiler.compile();
}
#![allow(dead_code)]
extern crate core;

mod checker;
mod tracker;
mod data;
mod error;
mod formation;
mod format;
mod initial;
mod internal;
mod parser;
mod resolver;
mod scanner;
mod schema;
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
#[cfg(feature = "formation")]
pub mod formation;
#[cfg(feature = "generator")]
pub mod generator;
#[cfg(feature = "initial")]
pub mod initializer;
#[cfg(feature = "internal")]
pub mod internal;
#[cfg(feature = "parser")]
pub mod parser;
#[cfg(feature = "resolver")]
pub mod resolver;
#[cfg(feature = "scanner")]
pub mod scanner;

#[cfg(feature = "internal")]
mod format;

#[cfg(feature = "internal")]
pub mod reporter;

#[cfg(feature = "internal")]
pub mod text;

#[cfg(feature = "internal")]
pub mod tracker;

#[cfg(feature = "internal")]
pub mod data;

pub mod analyzer;

fn main() {
    #[cfg(feature = "internal")]
    {
        use {
            data::Str,
            internal::{
                Session,
                logger::{LogInfo, LogPlan, Logger},
            },
            log::Level,
        };

        let plan = LogPlan::new(vec![LogInfo::Time, LogInfo::Level, LogInfo::Message])
            .with_separator(Str::from(" "));

        let logger = Logger::new(Level::max(), plan);

        if let Err(error) = logger.init() {
            eprintln!("error: failed to initialize logger: {}", error);
            return;
        }

        let mut compiler = Session::start();

        compiler.compile();
    }
}

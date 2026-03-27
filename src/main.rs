extern crate core;

#[cfg(feature = "combinator")]
pub mod combinator;
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
mod interpreter;

fn main() {
    #[cfg(feature = "internal")]
    {
        use internal::Session;

        let mut compiler = Session::start();

        compiler.compile();
    }
}
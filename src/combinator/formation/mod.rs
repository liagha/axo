mod combinator;
mod flow;
mod form;
mod formation;
mod former;
mod memo;
mod outcome;
mod sink;
mod traits;

pub use flow::*;
pub use form::*;
pub use formation::*;
pub use former::*;
pub use memo::*;
pub use outcome::*;
pub use sink::*;

pub type Joint<'a, 'source, Source, Input, Output, Failure> = (
    &'source mut Former<'a, 'source, Source, Input, Output, Failure>,
    &'source mut Formation<'a, 'source, Source, Input, Output, Failure>,
);
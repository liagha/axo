mod combinator;
mod command;
mod operation;
mod operator;
mod plan;
mod trigger;

pub use command::*;
pub use operation::*;
pub use operator::*;
pub use plan::*;
pub use trigger::*;

pub type Joint<'op, 'source, Store> = (
    &'op mut Operator<Store>,
    &'op mut Operation<'source, Store>,
);
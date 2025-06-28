#![allow(dead_code)]

mod format;
mod traits;
pub mod pattern;
pub mod order;
pub mod form;
pub mod former;

pub mod functions {
    use {
        super::{
            form::Form,
            former::Draft,
            pattern::Pattern,
        },
        crate::{
            thread::{Arc, Mutex},
            compiler::Context,
            axo_cursor::Position,
        },
    };

    pub type Emitter<Input, Output, Failure> = Arc<dyn Fn(&mut Context, Form<Input, Output, Failure>) -> Failure + Send + Sync>;
    pub type Evaluator<Input, Output, Failure> = Arc<dyn Fn() -> Pattern<Input, Output, Failure> + Send + Sync>;
    pub type Executor = Arc<Mutex<dyn FnMut() -> () + Send + Sync>>;
    pub type Inspector<Input, Output, Failure> = dyn Fn(Draft<Input, Output, Failure>) -> crate::axo_form::order::Order<Input, Output, Failure> + Send + Sync;
    pub type Predicate<Input> = Arc<Mutex<dyn FnMut(&Input) -> bool + Send + Sync>>;
    pub type Shifter = Arc<dyn Fn(&mut usize, &mut Position)>;
    pub type Transformer<Input, Output, Failure> = Arc<Mutex<dyn FnMut(&mut Context, Form<Input, Output, Failure>) -> Result<Output, Failure> + Send + Sync>>;
    pub type Tweaker<Input, Output, Failure> = Arc<dyn Fn(&mut Draft<Input, Output, Failure>) + Send + Sync>;
}
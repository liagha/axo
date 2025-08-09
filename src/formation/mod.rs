#![allow(dead_code)]

mod format;
pub mod classifier;
pub mod form;
pub mod former;
pub mod order;

pub mod helper {
    use {
        super::{
            classifier::Classifier,
            form::Form,
            former::Draft,
            order::Order,
        },
        crate::{
            data::thread::{Arc, Mutex},
            format::Debug,
            internal::{
                hash::{Hash},
                compiler::{Registry, Marked},
            },
            tracker::Peekable,
        },
    };

    pub trait Formable<'formable>:
        Clone + Debug + Eq + Hash + PartialEq + 'formable
    {}

    impl<'formable, T> Formable<'formable> for T
    where
        T: Clone + Debug + Eq + Hash + PartialEq + 'formable
    {}

    pub trait Source<'source, Input>: Peekable<'source, Input> + Marked<'source>
    where
        Input: Formable<'source>,
    {}

    impl<'source, Target, Input> Source<'source, Input> for Target
    where
        Target: Peekable<'source, Input> + Marked<'source>,
        Input: Formable<'source>,
    {}

    pub type Emitter<'emitter, Input, Output, Failure> = Arc<dyn Fn(&mut Registry, Form<'emitter, Input, Output, Failure>) -> Failure + 'emitter>;
    pub type Evaluator<'evaluator, Input, Output, Failure> = Arc<dyn Fn() -> Classifier<'evaluator, Input, Output, Failure> + 'evaluator>;
    pub type Inspector<'inspector, Input, Output, Failure> = Arc<dyn Fn(Draft<'inspector, Input, Output, Failure>) -> Arc<dyn Order<'inspector, Input, Output, Failure> + 'inspector> + 'inspector>;
    pub type Performer<'performer> = Arc<Mutex<dyn FnMut() -> () + 'performer>>;
    pub type Predicate<'predicate, Input> = Arc<dyn Fn(&Input) -> bool + 'predicate>;
    pub type Transformer<'transformer, Input, Output, Failure> = Arc<Mutex<dyn FnMut(&mut Registry, Form<'transformer, Input, Output, Failure>) -> Result<Form<'transformer, Input, Output, Failure>, Failure> + 'transformer>>;
}
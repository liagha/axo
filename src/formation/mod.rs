use std::sync::atomic::{AtomicUsize, Ordering};
use crate::data::Identity;

pub mod classifier;
pub mod form;
pub mod former;
pub mod order;
mod traits;

pub static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub(super) fn next_identity() -> Identity {
    crate::resolver::COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub mod helper {
    use {
        super::{classifier::Classifier, order::Order},
        crate::{
            data::memory::Rc,
            format::Show,
            internal::hash::Hash,
            tracker::{Peekable, Spanned},
        },
    };

    pub trait Formable<'a>:
    Clone + Eq + Hash + PartialEq + Show<'a> + Spanned<'a> + 'a
    {
    }

    impl<'a, T> Formable<'a> for T where
        T: Clone + Eq + Hash + PartialEq + Show<'a> + Spanned<'a> + 'a
    {
    }

    pub trait Source<'a, Input>: Peekable<'a, Input>
    where
        Input: Formable<'a>,
    {
    }

    impl<'a, Target, Input> Source<'a, Input> for Target
    where
        Target: Peekable<'a, Input>,
        Input: Formable<'a>,
    {
    }

    pub type Emitter<'a, Input, Output, Failure> =
    Rc<dyn Fn(Classifier<'a, Input, Output, Failure>) -> Failure + 'a>;

    pub type Evaluator<'a, Input, Output, Failure> =
    Rc<dyn Fn() -> Classifier<'a, Input, Output, Failure> + 'a>;

    pub type Inspector<'a, Input, Output, Failure> =
    Rc<dyn Fn(Classifier<'a, Input, Output, Failure>) -> Rc<dyn Order<'a, Input, Output, Failure>> + 'a>;

    pub type Performer = Rc<dyn Fn()>;

    pub type Predicate<'a, Input> = Rc<dyn Fn(&Input) -> bool + 'a>;

    pub type Transformer<'a, Input, Output, Failure> = &'a dyn Fn(
        &mut Classifier<'a, Input, Output, Failure>,
    ) -> Result<(), Failure>;
}

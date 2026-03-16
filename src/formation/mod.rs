pub mod classifier;
pub mod form;
pub mod former;
pub mod order;
mod traits;

pub mod helper {
    use {
        super::{classifier::Classifier, order::Order},
        crate::{
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
    &'a dyn Fn(Classifier<'a, Input, Output, Failure>) -> Failure;

    pub type Evaluator<'a, Input, Output, Failure> =
    &'a dyn Fn() -> Classifier<'a, Input, Output, Failure>;

    pub type Inspector<'a, Input, Output, Failure> =
    &'a dyn Fn(Classifier<'a, Input, Output, Failure>) -> &'a dyn Order<'a, Input, Output, Failure>;

    pub type Performer<'a> = &'a dyn Fn();

    pub type Predicate<'a, Input> = &'a dyn Fn(&Input) -> bool;

    pub type Transformer<'a, Input, Output, Failure> = &'a dyn Fn(
        &mut Classifier<'a, Input, Output, Failure>,
    ) -> Result<(), Failure>;
}

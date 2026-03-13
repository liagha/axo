pub mod classifier;
pub mod form;
pub mod former;
pub mod order;
mod traits;

pub mod helper {
    use {
        super::{classifier::Classifier, order::Order},
        crate::{
            data::sync::{Rc},
            internal::hash::Hash,
            tracker::Peekable,
            format::Show,
        },
    };
    use crate::tracker::Spanned;

    pub trait Formable<'formable>: Clone + Eq + Hash + PartialEq + Show<'formable, Verbosity = u8> + Spanned<'formable> + 'formable {}

    impl<'formable, T> Formable<'formable> for T where
        T: Clone + Eq + Hash + PartialEq + Show<'formable, Verbosity = u8> + Spanned<'formable> + 'formable
    {
    }

    pub trait Source<'source, Input>: Peekable<'source, Input>
    where
        Input: Formable<'source>,
    {
    }

    impl<'source, Target, Input> Source<'source, Input> for Target
    where
        Target: Peekable<'source, Input>,
        Input: Formable<'source>,
    {
    }

    pub type Emitter<'emitter, Input, Output, Failure> =
    Rc<dyn Fn(Classifier<'emitter, Input, Output, Failure>) -> Failure + 'emitter>;
    pub type Evaluator<'evaluator, Input, Output, Failure> =
    Rc<dyn Fn() -> Classifier<'evaluator, Input, Output, Failure> + 'evaluator>;
    pub type Inspector<'inspector, Input, Output, Failure> = Rc<
        dyn Fn(
            Classifier<'inspector, Input, Output, Failure>,
        ) -> Rc<dyn Order<'inspector, Input, Output, Failure> + 'inspector>
        + 'inspector,
    >;
    pub type Performer<'performer> = Rc<dyn Fn() -> () + 'performer>;
    pub type Predicate<'predicate, Input> = Rc<dyn Fn(&Input) -> bool + 'predicate>;

    pub type Transformer<'transformer, Input, Output, Failure> = Rc<
        dyn Fn(
            &mut Classifier<'transformer, Input, Output, Failure>,
        ) -> Result<(), Failure>
        + 'transformer,
    >;
}

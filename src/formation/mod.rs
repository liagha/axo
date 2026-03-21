use crate::{
    data::{
        Identity,
        sync::{AtomicUsize, Ordering},
    },
};

mod classifier;
mod form;
mod former;
mod order;
mod traits;

pub use {
    classifier::*,
    former::*,
    form::*,
    order::*,
};

pub static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub(super) fn next_identity() -> Identity {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub(super) mod helper {
    use {
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
}

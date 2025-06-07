use std::hash::Hash;
use {
    super::{
        pattern::{Emitter, Transformer},
    },

    crate::{
        format::Debug,
        thread::Arc,
        axo_span::Span,
    }
};

#[derive(Clone)]
pub enum Action<Input, Output, Failure>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    Map(Transformer<Input, Output, Failure>),
    Ignore,
    Trigger {
        found: Box<Action<Input, Output, Failure>>,
        missing: Box<Action<Input, Output, Failure>>,
    },
    Capture {
        identifier: usize,
    },
    Failure(Emitter<Failure>),
}

impl<Input, Output, Failure> Action<Input, Output, Failure>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub fn map(f: impl Into<Transformer<Input, Output, Failure>>) -> Self {
        Self::Map(f.into())
    }

    pub fn error_with<F>(f: F) -> Self
    where
        F: Fn(Span) -> Failure + 'static,
    {
        Self::Failure(Arc::new(f))
    }

    pub fn require_or_error(function: Emitter<Failure>) -> Self {
        Self::Trigger {
            found: Box::new(Self::Ignore),
            missing: Box::new(Self::Failure(function)),
        }
    }
}

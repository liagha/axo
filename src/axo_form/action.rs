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
pub enum Action<Input, Output, Error>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Error: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    Map(Transformer<Input, Output, Error>),
    Ignore,
    Trigger {
        found: Box<Action<Input, Output, Error>>,
        missing: Box<Action<Input, Output, Error>>,
    },
    Error(Emitter<Error>),
}

impl<Input, Output, Error> Action<Input, Output, Error>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Error: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub fn map(f: impl Into<Transformer<Input, Output, Error>>) -> Self {
        Self::Map(f.into())
    }

    pub fn error_with<F>(f: F) -> Self
    where
        F: Fn(Span) -> Error + 'static,
    {
        Self::Error(Arc::new(f))
    }

    pub fn require_or_error(function: Emitter<Error>) -> Self {
        Self::Trigger {
            found: Box::new(Self::Ignore),
            missing: Box::new(Self::Error(function)),
        }
    }
}

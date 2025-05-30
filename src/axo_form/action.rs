use crate::format::Debug;
use crate::thread::Arc;
use crate::axo_form::Form;
use crate::axo_form::pattern::{Emitter, Transformer};
use crate::axo_span::Span;

#[derive(Clone)]
pub enum Action<Input, Output, Error>
where
    Input: Clone + PartialEq + Debug,
    Output: Clone + PartialEq + Debug,
    Error: Clone + PartialEq + Debug,
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
    Input: Clone + PartialEq + Debug,
    Output: Clone + PartialEq + Debug,
    Error: Clone + PartialEq + Debug,
{
    pub fn map<F>(f: F) -> Self
    where
        F: Fn(Vec<Form<Input, Output, Error>>, Span) -> Result<Output, Error> + Send + Sync + 'static,
    {
        Self::Map(Arc::new(f))
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

    pub fn transform_if_found(transform: Transformer<Input, Output, Error>) -> Self {
        Self::Trigger {
            found: Box::new(Self::Map(transform)),
            missing: Box::new(Self::Ignore),
        }
    }
}

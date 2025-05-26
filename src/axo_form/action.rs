use crate::format::Debug;
use crate::thread::Arc;
use crate::axo_form::{ErrorFunction, Form, TransformFunction};
use crate::axo_span::Span;

#[derive(Clone)]
pub enum Action<Input, Output, Error>
where
    Input: Clone + PartialEq + Debug,
    Output: Clone + PartialEq + Debug,
    Error: Clone + PartialEq + Debug,
{
    Transform(TransformFunction<Input, Output, Error>),
    Ignore,
    Error(ErrorFunction<Error>),
    Conditional {
        found: Box<Action<Input, Output, Error>>,
        missing: Box<Action<Input, Output, Error>>,
    },
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
        Self::Transform(Arc::new(f))
    }

    pub fn error_with<F>(f: F) -> Self
    where
        F: Fn(Span) -> Error + 'static,
    {
        Self::Error(Arc::new(f))
    }

    pub fn require_or_error(function: ErrorFunction<Error>) -> Self {
        Self::Conditional {
            found: Box::new(Self::Ignore),
            missing: Box::new(Self::Error(function)),
        }
    }

    pub fn transform_if_found(transform: TransformFunction<Input, Output, Error>) -> Self {
        Self::Conditional {
            found: Box::new(Self::Transform(transform)),
            missing: Box::new(Self::Ignore),
        }
    }
}

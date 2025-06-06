use std::hash::Hash;
use {
    super::{
        former::Form,
        action::Action,
    },

    crate::{
        format::Debug,
        thread::Arc,
        axo_span::Span,
        Peekable,
    }
};
use crate::compiler::Context;

pub type Transformer<Input, Output, Error> = Arc<dyn Fn(&mut Context, Form<Input, Output, Error>) -> Result<Output, Error> + Send + Sync>;
pub type Predicate<Input> = Arc<dyn Fn(&Input) -> bool + Send + Sync>;
pub type Emitter<Error> = Arc<dyn Fn(Span) -> Error>;
pub type Evaluator<Input, Output, Error> = Arc<dyn Fn() -> Pattern<Input, Output, Error> + Send + Sync>;

#[derive(Clone)]
pub enum PatternKind<Input, Output, Error>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Error: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    Literal(Input),
    Alternative(Vec<Pattern<Input, Output, Error>>),
    Guard {
        predicate: Arc<dyn Fn(&dyn Peekable<Input>) -> bool + Send + Sync>,
        pattern: Box<Pattern<Input, Output, Error>>,
    },
    Required {
        pattern: Box<Pattern<Input, Output, Error>>,
        action: Action<Input, Output, Error>,
    },
    Sequence(Vec<Pattern<Input, Output, Error>>),
    Repetition {
        pattern: Box<Pattern<Input, Output, Error>>,
        minimum: usize,
        maximum: Option<usize>,
    },
    Optional(Box<Pattern<Input, Output, Error>>),
    Condition(Predicate<Input>),
    Negation(Box<Pattern<Input, Output, Error>>),
    Deferred(Evaluator<Input, Output, Error>),
    Capture {
        identifier: usize,
        pattern: Box<Pattern<Input, Output, Error>>,
    },
    WildCard,
}

#[derive(Clone, Debug)]
pub struct Pattern<Input, Output, Error>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Error: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub kind: PatternKind<Input, Output, Error>,
    pub action: Option<Action<Input, Output, Error>>,
}

impl<Input, Output, Error> Pattern<Input, Output, Error>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Error: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub fn exact(value: Input) -> Self {
        Self {
            kind: PatternKind::Literal(value),
            action: None,
        }
    }

    pub fn guard(
        predicate: Arc<dyn Fn(&dyn Peekable<Input>) -> bool + Send + Sync>,
        pattern: Pattern<Input, Output, Error>
    ) -> Self {
        Self {
            kind: PatternKind::Guard {
                predicate,
                pattern: Box::new(pattern),
            },
            action: None,
        }
    }

    pub fn alternative(patterns: impl Into<Vec<Pattern<Input, Output, Error>>>) -> Self {
        Self {
            kind: PatternKind::Alternative(patterns.into()),
            action: None,
        }
    }

    pub fn sequence(patterns: impl Into<Vec<Pattern<Input, Output, Error>>>) -> Self {
        Self {
            kind: PatternKind::Sequence(patterns.into()),
            action: None,
        }
    }

    pub fn capture(
        identifier: usize,
        pattern: impl Into<Box<Pattern<Input, Output, Error>>>,
    ) -> Self {
        Self {
            kind: PatternKind::Capture {
                identifier,
                pattern: pattern.into(),
            },
            action: None,
        }
    }

    pub fn as_capture(&self, identifier: usize) -> Self {
        Self::capture(identifier, Box::new(self.clone()))
    }

    pub fn repeat(
        pattern: impl Into<Box<Pattern<Input, Output, Error>>>,
        minimum: usize,
        maximum: Option<usize>,
    ) -> Self {
        Self {
            kind: PatternKind::Repetition {
                pattern: pattern.into(),
                minimum,
                maximum,
            },
            action: None,
        }
    }

    pub fn optional(pattern: impl Into<Box<Pattern<Input, Output, Error>>>) -> Self {
        Self {
            kind: PatternKind::Optional(pattern.into()),
            action: None,
        }
    }

    pub fn predicate(predicate: Predicate<Input>) -> Self {
        Self {
            kind: PatternKind::Condition(predicate),
            action: None,
        }
    }

    pub fn negate(pattern: impl Into<Box<Pattern<Input, Output, Error>>>) -> Self {
        Self {
            kind: PatternKind::Negation(pattern.into()),
            action: None,
        }
    }

    pub fn anything() -> Self {
        Self {
            kind: PatternKind::WildCard,
            action: None,
        }
    }

    pub fn required(
        pattern: impl Into<Box<Pattern<Input, Output, Error>>>,
        action: Action<Input, Output, Error>,
    ) -> Self {
        Self {
            kind: PatternKind::Required {
                pattern: pattern.into(),
                action,
            },
            action: None,
        }
    }

    pub fn lazy<F>(factory: F) -> Self
    where
        F: Fn() -> Pattern<Input, Output, Error> + Send + Sync + 'static,
    {
        Self {
            kind: PatternKind::Deferred(Arc::new(factory)),
            action: None,
        }
    }

    pub fn resolve_lazy(&self) -> Pattern<Input, Output, Error> {
        match &self.kind {
            PatternKind::Deferred(factory) => {
                factory()
            }
            _ => self.clone(),
        }
    }

    pub fn transform(
        pattern: impl Into<Box<Pattern<Input, Output, Error>>>,
        transform: Transformer<Input, Output, Error>,
    ) -> Self {
        Self {
            kind: PatternKind::Sequence(vec![*pattern.into()]),
            action: Some(Action::Map(transform)),
        }
    }

    pub fn ignore(pattern: impl Into<Box<Pattern<Input, Output, Error>>>) -> Self {
        Self {
            kind: PatternKind::Sequence(vec![*pattern.into()]),
            action: Some(Action::Ignore),
        }
    }

    pub fn error(
        pattern: impl Into<Box<Pattern<Input, Output, Error>>>,
        function: Emitter<Error>,
    ) -> Self {
        Self {
            kind: PatternKind::Sequence(vec![*pattern.into()]),
            action: Some(Action::Error(function)),
        }
    }

    pub fn conditional(
        pattern: impl Into<Box<Pattern<Input, Output, Error>>>,
        found: Action<Input, Output, Error>,
        missing: Action<Input, Output, Error>,
    ) -> Self {
        Self {
            kind: PatternKind::Sequence(vec![*pattern.into()]),
            action: Some(Action::Trigger {
                found: Box::new(found),
                missing: Box::new(missing),
            }),
        }
    }

    pub fn with_action(mut self, action: Action<Input, Output, Error>) -> Self {
        self.action = Some(action);
        self
    }

    pub fn with_ignore(mut self) -> Self {
        self.action = Some(Action::Ignore);
        self
    }

    pub fn with_error(mut self, function: Emitter<Error>) -> Self {
        self.action = Some(Action::Error(function));
        self
    }

    pub fn with_conditional(
        mut self,
        found: Action<Input, Output, Error>,
        missing: Action<Input, Output, Error>,
    ) -> Self {
        self.action = Some(Action::Trigger {
            found: Box::new(found),
            missing: Box::new(missing),
        });
        self
    }

    pub fn with_transform(mut self, transform: Transformer<Input, Output, Error>) -> Self {
        self.action = Some(Action::Map(transform));
        self
    }

    pub fn any_of(patterns: impl Into<Vec<Pattern<Input, Output, Error>>>) -> Self {
        Self::alternative(patterns)
    }

    pub fn all_of(patterns: impl Into<Vec<Pattern<Input, Output, Error>>>) -> Self {
        Self::sequence(patterns)
    }

    pub fn maybe(pattern: impl Into<Box<Pattern<Input, Output, Error>>>) -> Self {
        Self::optional(pattern)
    }

    pub fn not(pattern: impl Into<Box<Pattern<Input, Output, Error>>>) -> Self {
        Self::negate(pattern)
    }

    pub fn anything_except(patterns: impl Into<Vec<Pattern<Input, Output, Error>>>) -> Self {
        Self::negate(Box::new(Self::alternative(patterns)))
    }

    pub fn delimited(
        open: Pattern<Input, Output, Error>,
        content: Pattern<Input, Output, Error>,
        close: Pattern<Input, Output, Error>,
    ) -> Self {
        Self::sequence(vec![
            open.with_ignore(),
            content,
            close.with_ignore(),
        ])
    }

    pub fn when<F>(predicate: F) -> Self
    where
        F: Fn(&Input) -> bool + Send + Sync + 'static,
    {
        Self::predicate(Arc::new(predicate))
    }

    pub fn map(pattern: impl Into<Box<Pattern<Input, Output, Error>>>, f: impl Into<Transformer<Input, Output, Error>>) -> Self {
        Self::transform(pattern, f.into())
    }

    pub fn empty() -> Self {
        Self::optional(Box::new(Self::negate(Box::new(Self::anything()))))
    }

    pub fn then(self, other: Pattern<Input, Output, Error>) -> Self {
        Self::sequence(vec![self, other])
    }

    pub fn or(self, other: Pattern<Input, Output, Error>) -> Self {
        Self::alternative(vec![self, other])
    }

    pub fn optional_self(self) -> Self {
        Self::optional(Box::new(self))
    }

    pub fn repeat_self(self, min: usize, max: Option<usize>) -> Self {
        Self::repeat(Box::new(self), min, max)
    }
}
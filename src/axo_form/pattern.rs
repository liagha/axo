use crate::axo_form::action::Emitter;
use std::sync::Mutex;
use {
    super::{action::Action, form::Form},
    crate::{compiler::Context, format::Debug, hash::Hash, thread::Arc, Peekable},
};

pub type Predicate<Input> = Arc<Mutex<dyn FnMut(&Input) -> bool + Send + Sync>>;
pub type Evaluator<Input, Output, Failure> =
    Arc<Mutex<dyn FnMut() -> Pattern<Input, Output, Failure> + Send + Sync>>;

#[derive(Clone)]
pub enum PatternKind<Input, Output, Failure>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    Alternative(Vec<Pattern<Input, Output, Failure>>),
    Condition(Predicate<Input>),
    Deferred(Evaluator<Input, Output, Failure>),
    Guard {
        predicate: Arc<Mutex<dyn FnMut(&dyn Peekable<Input>) -> bool + Send + Sync>>,
        pattern: Box<Pattern<Input, Output, Failure>>,
    },
    Literal(Input),
    Negation(Box<Pattern<Input, Output, Failure>>),
    Optional(Box<Pattern<Input, Output, Failure>>),
    Repetition {
        pattern: Box<Pattern<Input, Output, Failure>>,
        minimum: usize,
        maximum: Option<usize>,
    },
    Sequence(Vec<Pattern<Input, Output, Failure>>),
    WildCard,
    Wrap(Box<Pattern<Input, Output, Failure>>),
}

#[derive(Clone, Debug)]
pub struct Pattern<Input, Output, Failure>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub kind: PatternKind<Input, Output, Failure>,
    pub action: Option<Action<Input, Output, Failure>>,
}

impl<Input, Output, Failure> Pattern<Input, Output, Failure>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub fn exact(value: Input) -> Self {
        Self {
            kind: PatternKind::Literal(value),
            action: None,
        }
    }

    pub fn guard<F>(predicate: F, pattern: Pattern<Input, Output, Failure>) -> Self
    where
        F: FnMut(&dyn Peekable<Input>) -> bool + Send + Sync + 'static,
    {
        Self {
            kind: PatternKind::Guard {
                predicate: Arc::new(Mutex::new(predicate)),
                pattern: Box::new(pattern),
            },
            action: None,
        }
    }

    pub fn alternative(patterns: impl Into<Vec<Pattern<Input, Output, Failure>>>) -> Self {
        Self {
            kind: PatternKind::Alternative(patterns.into()),
            action: None,
        }
    }

    pub fn sequence(patterns: impl Into<Vec<Pattern<Input, Output, Failure>>>) -> Self {
        Self {
            kind: PatternKind::Sequence(patterns.into()),
            action: None,
        }
    }

    pub fn capture(
        identifier: usize,
        pattern: impl Into<Box<Pattern<Input, Output, Failure>>>,
    ) -> Self {
        Self {
            kind: pattern.into().kind,
            action: Some(Action::Capture { identifier }),
        }
    }

    pub fn as_capture(&self, identifier: usize) -> Self {
        Self::capture(identifier, Box::new(self.clone()))
    }

    pub fn repeat(
        pattern: impl Into<Box<Pattern<Input, Output, Failure>>>,
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

    pub fn optional(pattern: impl Into<Box<Pattern<Input, Output, Failure>>>) -> Self {
        Self {
            kind: PatternKind::Optional(pattern.into()),
            action: None,
        }
    }

    pub fn predicate<F>(predicate: F) -> Self
    where
        F: FnMut(&Input) -> bool + Send + Sync + 'static,
    {
        Self {
            kind: PatternKind::Condition(Arc::new(Mutex::new(predicate))),
            action: None,
        }
    }

    pub fn negate(pattern: impl Into<Box<Pattern<Input, Output, Failure>>>) -> Self {
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
        pattern: impl Into<Box<Pattern<Input, Output, Failure>>>,
        action: Action<Input, Output, Failure>,
    ) -> Self {
        Self {
            kind: PatternKind::Wrap(pattern.into()),
            action: Some(Action::Trigger {
                found: Action::execute(|| {}).into(),
                missing: action.into(),
            }),
        }
    }

    pub fn lazy<F>(factory: F) -> Self
    where
        F: FnMut() -> Pattern<Input, Output, Failure> + Send + Sync + 'static,
    {
        Self {
            kind: PatternKind::Deferred(Arc::new(Mutex::new(factory))),
            action: None,
        }
    }

    pub fn resolve_lazy(&self) -> Pattern<Input, Output, Failure> {
        match &self.kind {
            PatternKind::Deferred(factory) => {
                let mut guard = factory.lock().unwrap();

                guard()
            }
            _ => self.clone(),
        }
    }

    pub fn transform<T>(
        pattern: impl Into<Box<Pattern<Input, Output, Failure>>>,
        transform: T,
    ) -> Self
    where
        T: FnMut(&mut Context, Form<Input, Output, Failure>) -> Result<Output, Failure>
            + Send
            + Sync
            + 'static,
    {
        Self {
            kind: PatternKind::Sequence(vec![*pattern.into()]),
            action: Some(Action::Map(Arc::new(Mutex::new(transform)))),
        }
    }

    pub fn ignore(pattern: impl Into<Box<Pattern<Input, Output, Failure>>>) -> Self {
        Self {
            kind: PatternKind::Sequence(vec![*pattern.into()]),
            action: Some(Action::Ignore),
        }
    }

    pub fn error(
        pattern: impl Into<Box<Pattern<Input, Output, Failure>>>,
        function: Emitter<Failure>,
    ) -> Self {
        Self {
            kind: PatternKind::Sequence(vec![*pattern.into()]),
            action: Some(Action::Failure(function)),
        }
    }

    pub fn conditional(
        pattern: impl Into<Box<Pattern<Input, Output, Failure>>>,
        found: Action<Input, Output, Failure>,
        missing: Action<Input, Output, Failure>,
    ) -> Self {
        Self {
            kind: PatternKind::Sequence(vec![*pattern.into()]),
            action: Some(Action::Trigger {
                found: Box::new(found),
                missing: Box::new(missing),
            }),
        }
    }

    pub fn action(
        pattern: impl Into<Box<Pattern<Input, Output, Failure>>>,
        action: Action<Input, Output, Failure>,
    ) -> Self {
        Self {
            kind: PatternKind::Sequence(vec![*pattern.into()]),
            action: Some(action),
        }
    }

    pub fn with_action(mut self, action: Action<Input, Output, Failure>) -> Self {
        self.action = Some(action);
        self
    }

    pub fn with_ignore(mut self) -> Self {
        self.action = Some(Action::Ignore);
        self
    }

    pub fn with_error(mut self, function: Emitter<Failure>) -> Self {
        self.action = Some(Action::Failure(function));
        self
    }

    pub fn with_conditional(
        mut self,
        found: Action<Input, Output, Failure>,
        missing: Action<Input, Output, Failure>,
    ) -> Self {
        self.action = Some(Action::Trigger {
            found: Box::new(found),
            missing: Box::new(missing),
        });
        self
    }

    pub fn with_transform<T>(mut self, transform: T) -> Self
    where
        T: FnMut(&mut Context, Form<Input, Output, Failure>) -> Result<Output, Failure>
            + Send
            + Sync
            + 'static,
    {
        self.action = Some(Action::Map(Arc::new(Mutex::new(transform))));
        self
    }

    pub fn any_of(patterns: impl Into<Vec<Pattern<Input, Output, Failure>>>) -> Self {
        Self::alternative(patterns)
    }

    pub fn all_of(patterns: impl Into<Vec<Pattern<Input, Output, Failure>>>) -> Self {
        Self::sequence(patterns)
    }

    pub fn maybe(pattern: impl Into<Box<Pattern<Input, Output, Failure>>>) -> Self {
        Self::optional(pattern)
    }

    pub fn not(pattern: impl Into<Box<Pattern<Input, Output, Failure>>>) -> Self {
        Self::negate(pattern)
    }

    pub fn anything_except(patterns: impl Into<Vec<Pattern<Input, Output, Failure>>>) -> Self {
        Self::negate(Box::new(Self::alternative(patterns)))
    }

    pub fn delimited(
        open: Pattern<Input, Output, Failure>,
        content: Pattern<Input, Output, Failure>,
        close: Pattern<Input, Output, Failure>,
    ) -> Self {
        Self::sequence(vec![open.with_ignore(), content, close.with_ignore()])
    }

    pub fn empty() -> Self {
        Self::optional(Box::new(Self::negate(Box::new(Self::anything()))))
    }

    pub fn then(self, other: Pattern<Input, Output, Failure>) -> Self {
        Self::sequence(vec![self, other])
    }

    pub fn or(self, other: Pattern<Input, Output, Failure>) -> Self {
        Self::alternative(vec![self, other])
    }

    pub fn optional_self(self) -> Self {
        Self::optional(Box::new(self))
    }

    pub fn repeat_self(self, min: usize, max: Option<usize>) -> Self {
        Self::repeat(Box::new(self), min, max)
    }
}

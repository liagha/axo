use {
    super::{
        form::Form,
        action::Action,
    },

    crate::{
        hash::Hash,
        compiler::Context,
        format::Debug,
        thread::Arc,
        axo_span::Span,
        Peekable,
    }
};

pub type Transformer<Input, Output, Failure> = Arc<dyn Fn(&mut Context, Form<Input, Output, Failure>) -> Result<Output, Failure> + Send + Sync>;
pub type Predicate<Input> = Arc<dyn Fn(&Input) -> bool + Send + Sync>;
pub type Emitter<Failure> = Arc<dyn Fn(Span) -> Failure>;
pub type Evaluator<Input, Output, Failure> = Arc<dyn Fn() -> Pattern<Input, Output, Failure> + Send + Sync>;

#[derive(Clone)]
pub enum PatternKind<Input, Output, Failure>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    Literal(Input),
    Alternative(Vec<Pattern<Input, Output, Failure>>),
    Guard {
        predicate: Arc<dyn Fn(&dyn Peekable<Input>) -> bool + Send + Sync>,
        pattern: Box<Pattern<Input, Output, Failure>>,
    },
    Required {
        pattern: Box<Pattern<Input, Output, Failure>>,
        action: Action<Input, Output, Failure>,
    },
    Sequence(Vec<Pattern<Input, Output, Failure>>),
    Repetition {
        pattern: Box<Pattern<Input, Output, Failure>>,
        minimum: usize,
        maximum: Option<usize>,
    },
    Optional(Box<Pattern<Input, Output, Failure>>),
    Condition(Predicate<Input>),
    Negation(Box<Pattern<Input, Output, Failure>>),
    Deferred(Evaluator<Input, Output, Failure>),
    WildCard,
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

    pub fn guard(
        predicate: Arc<dyn Fn(&dyn Peekable<Input>) -> bool + Send + Sync>,
        pattern: Pattern<Input, Output, Failure>
    ) -> Self {
        Self {
            kind: PatternKind::Guard {
                predicate,
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

    pub fn predicate(predicate: Predicate<Input>) -> Self {
        Self {
            kind: PatternKind::Condition(predicate),
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
            kind: PatternKind::Required {
                pattern: pattern.into(),
                action,
            },
            action: None,
        }
    }

    pub fn lazy<F>(factory: F) -> Self
    where
        F: Fn() -> Pattern<Input, Output, Failure> + Send + Sync + 'static,
    {
        Self {
            kind: PatternKind::Deferred(Arc::new(factory)),
            action: None,
        }
    }

    pub fn resolve_lazy(&self) -> Pattern<Input, Output, Failure> {
        match &self.kind {
            PatternKind::Deferred(factory) => {
                factory()
            }
            _ => self.clone(),
        }
    }

    pub fn transform(
        pattern: impl Into<Box<Pattern<Input, Output, Failure>>>,
        transform: Transformer<Input, Output, Failure>,
    ) -> Self {
        Self {
            kind: PatternKind::Sequence(vec![*pattern.into()]),
            action: Some(Action::Map(transform)),
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

    pub fn with_transform(mut self, transform: Transformer<Input, Output, Failure>) -> Self {
        self.action = Some(Action::Map(transform));
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

    pub fn map(pattern: impl Into<Box<Pattern<Input, Output, Failure>>>, f: impl Into<Transformer<Input, Output, Failure>>) -> Self {
        Self::transform(pattern, f.into())
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

#[macro_export]
macro_rules! pattern {
    // Empty pattern
    () => {
        Pattern::empty()
    };

    // Wildcard
    (_) => {
        Pattern::anything()
    };

    // Lazy evaluation (@ factory)
    (@ $factory:expr) => {
        Pattern::lazy($factory)
    };

    // Delimited pattern (open <> content <> close)
    ($open:tt <> $content:tt <> $close:tt) => {
        Pattern::delimited(
            pattern!($open),
            pattern!($content),
            pattern!($close)
        )
    };

    // Alternative (|) - handles multiple alternatives with proper recursion
    ($first:tt | $($rest:tt)|+) => {
        Pattern::alternative(vec![
            pattern!($first),
            $(pattern!($rest),)+
        ])
    };

    // Actions - these need to come BEFORE sequence handling for proper precedence

    // Transform with inline closure (pattern => |args| body)
    ($pattern:tt => |$($args:tt)*| $body:expr) => {
        pattern!($pattern).with_transform(Arc::new(|$($args)*| $body))
    };

    // Transform (pattern => transform)
    ($pattern:tt => $transform:expr) => {
        pattern!($pattern).with_transform($transform)
    };

    // Guard (pattern :: guard)
    ($pattern:tt :: $guard:expr) => {
        Pattern::guard($guard, pattern!($pattern))
    };

    // Ignore action (pattern >> ignore)
    ($pattern:tt >> ignore) => {
        pattern!($pattern).with_ignore()
    };

    // Error action (pattern >> error(emitter))
    ($pattern:tt >> error($emitter:expr)) => {
        pattern!($pattern).with_error($emitter)
    };

    // Capture action (pattern >> capture(id))
    ($pattern:tt >> capture($id:expr)) => {
        pattern!($pattern).as_capture($id)
    };

    // Conditional action (pattern >> if(found, missing))
    ($pattern:tt >> if($found:expr, $missing:expr)) => {
        pattern!($pattern).with_conditional($found, $missing)
    };

    // Repetition patterns with recursion

    // Optional (pattern?)
    ($pattern:tt ?) => {
        Pattern::optional(Box::new(pattern!($pattern)))
    };

    // Zero or more (pattern*)
    ($pattern:tt *) => {
        Pattern::repeat(Box::new(pattern!($pattern)), 0, None)
    };

    // One or more (pattern+)
    ($pattern:tt +) => {
        Pattern::repeat(Box::new(pattern!($pattern)), 1, None)
    };

    // Repetition with exact count {n}
    ($pattern:tt { $exact:literal }) => {
        Pattern::repeat(Box::new(pattern!($pattern)), $exact, Some($exact))
    };

    // Repetition with range {min, max}
    ($pattern:tt { $min:literal , $max:literal }) => {
        Pattern::repeat(Box::new(pattern!($pattern)), $min, Some($max))
    };

    // Repetition with minimum only {min,}
    ($pattern:tt { $min:literal , }) => {
        Pattern::repeat(Box::new(pattern!($pattern)), $min, None)
    };

    // Negation (!pattern)
    (!$pattern:tt) => {
        Pattern::negate(Box::new(pattern!($pattern)))
    };

    // Closure/predicate - this needs to come before sequence to avoid conflicts
    (|$($args:tt)*| $body:expr) => {
        Pattern::when(|$($args)*| $body)
    };

    // Parenthesized expressions - handle grouping, process contents recursively
    (($($inner:tt)*)) => {
        pattern!($($inner)*)
    };

    // Sequence handling - this comes AFTER actions to ensure proper precedence
    // Multiple comma-separated patterns
    ($first:tt , $($rest:tt),+ $(,)?) => {
        Pattern::sequence(vec![
            pattern!($first),
            $(pattern!($rest),)+
        ])
    };

    // Two patterns separated by comma (base case)
    ($first:tt , $second:tt) => {
        Pattern::sequence(vec![
            pattern!($first),
            pattern!($second)
        ])
    };

    // Literal values (characters, strings, etc.)
    ($literal:literal) => {
        Pattern::exact($literal)
    };

    // Raw expression passthrough for complex patterns - this should be last
    ($expr:expr) => {
        $expr
    };
}

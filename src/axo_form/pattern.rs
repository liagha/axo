use {
    super::{action::Action, form::Form},
    crate::{
        hash::Hash,
        format::Debug,
        compiler::Context,
        thread::{
            Arc, Mutex
        },
        axo_cursor::{
            Spanned,  
        },
        axo_form::{
            action::Emitter,
        },
    },
};

/// A predicate function that examines input and returns whether it matches some condition.
/// Used in conditional patterns to test input values.
pub type Predicate<Input> = Arc<Mutex<dyn FnMut(&Input) -> bool + Send + Sync>>;

/// An evaluator function that lazily creates patterns when needed.
/// Used for recursive or context-dependent pattern construction.
pub type Evaluator<Input, Output, Failure> =
    Arc<Mutex<dyn FnMut() -> Pattern<Input, Output, Failure> + Send + Sync>>;

/// The core matching behaviors that patterns can exhibit.
/// Each kind defines how a pattern attempts to match against input.
#[derive(Clone)]
pub enum PatternKind<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    /// Matches if any of the contained patterns match (logical OR).
    /// Tries patterns in order and succeeds on the first match.
    Alternative {
        patterns: Vec<Pattern<Input, Output, Failure>>
    },

    /// Matches input that satisfies the given predicate function.
    /// The predicate receives the input value and returns true/false.
    Predicate { 
        function: Predicate<Input>, 
    },

    /// Lazily evaluates to create a pattern when needed.
    /// Useful for recursive patterns or context-dependent matching.
    Deferred { 
        function: Evaluator<Input, Output, Failure> 
    },

    /// Matches exactly the specified input value.
    /// Uses equality comparison to determine matches.
    Literal {
        value: Input
    },
    
    /// Matches the input value using the PartialEq trait.
    /// Allows for different types than Input to be used.
    Twin {
        value: Arc<dyn PartialEq<Input>>,  
    },

    /// Matches input that does NOT match the inner pattern (logical NOT).
    /// Succeeds when the inner pattern fails, and vice versa.
    Negation {
        pattern: Box<Pattern<Input, Output, Failure>>
    },

    /// Optionally matches the inner pattern.
    /// Always succeeds, whether the inner pattern matches or not.
    Optional { 
        pattern: Box<Pattern<Input, Output, Failure>> 
    },

    /// Matches the inner pattern a specified number of times.
    /// Must match at least `minimum` times, up to `maximum` times (if specified).
    Repetition {
        pattern: Box<Pattern<Input, Output, Failure>>,
        minimum: usize,
        maximum: Option<usize>,
    },

    /// Matches all contained patterns in order (logical AND).
    /// All patterns must succeed for the sequence to succeed.
    Sequence { 
        patterns: Vec<Pattern<Input, Output, Failure>>, 
    },

    /// Wraps another pattern without changing its behavior.
    /// Used for applying actions to existing patterns.
    Wrapper { 
        pattern: Box<Pattern<Input, Output, Failure>> 
    },
}

/// A pattern defines how to match input and what action to take on successful matches.
/// Patterns are the building blocks of the parsing system, combining matching logic with transformation actions.
#[derive(Clone, Debug)]
pub struct Pattern<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    /// The matching behavior of this pattern
    pub kind: PatternKind<Input, Output, Failure>,
    /// Optional action to execute when the pattern matches
    pub action: Option<Action<Input, Output, Failure>>,
}

impl<Input, Output, Failure> Pattern<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    #[inline]
    pub fn literal(value: Input) -> Self {
        Self {
            kind: PatternKind::Literal { 
                value 
            },
            action: None,
        }
    }
    
    pub fn exact(value: impl PartialEq<Input> + 'static) -> Self {
        Self {
            kind: PatternKind::Twin {
                value: Arc::new(value),
            },
            action: None,
        }
    }

    #[inline]
    pub fn alternative(patterns: impl Into<Vec<Pattern<Input, Output, Failure>>>) -> Self {
        Self {
            kind: PatternKind::Alternative {
                patterns: patterns.into()
            },
            action: None,
        }
    }

    #[inline]
    pub fn sequence(patterns: impl Into<Vec<Pattern<Input, Output, Failure>>>) -> Self {
        Self {
            kind: PatternKind::Sequence { 
                patterns: patterns.into() 
            },
            action: None,
        }
    }

    #[inline]
    pub fn capture(
        identifier: usize,
        pattern: impl Into<Box<Pattern<Input, Output, Failure>>>,
    ) -> Self {
        Self {
            kind: pattern.into().kind,
            action: Some(Action::Capture { identifier }),
        }
    }

    #[inline]
    pub fn as_capture(&self, identifier: usize) -> Self {
        Self::capture(identifier, Box::new(self.clone()))
    }

    #[inline]
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

    #[inline]
    pub fn optional(pattern: impl Into<Box<Pattern<Input, Output, Failure>>>) -> Self {
        Self {
            kind: PatternKind::Optional { 
                pattern: pattern.into() 
            },
            action: None,
        }
    }

    #[inline]
    pub fn predicate<F>(predicate: F) -> Self
    where
        F: FnMut(&Input) -> bool + Send + Sync + 'static,
    {
        Self {
            kind: PatternKind::Predicate { 
                function: Arc::new(Mutex::new(predicate)) 
            },
            action: None,
        }
    }

    #[inline]
    pub fn negate(pattern: impl Into<Box<Pattern<Input, Output, Failure>>>) -> Self {
        Self {
            kind: PatternKind::Negation { 
                pattern: pattern.into() 
            },
            action: None,
        }
    }

    #[inline]
    pub fn anything() -> Self {
        Self::predicate(|_| true)
    }

    #[inline]
    pub fn nothing() -> Self {
        Self::predicate(|_| false)
    }

    #[inline]
    pub fn required(
        pattern: impl Into<Box<Pattern<Input, Output, Failure>>>,
        action: Action<Input, Output, Failure>,
    ) -> Self {
        Self {
            kind: PatternKind::Wrapper { 
                pattern: pattern.into() 
            },
            action: Some(Action::Trigger {
                found: Action::perform(|| {}).into(),
                missing: action.into(),
            }),
        }
    }

    #[inline]
    pub fn lazy<F>(factory: F) -> Self
    where
        F: FnMut() -> Pattern<Input, Output, Failure> + Send + Sync + 'static,
    {
        Self {
            kind: PatternKind::Deferred { 
                function: Arc::new(Mutex::new(factory)) 
            },
            action: None,
        }
    }

    #[inline]
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
            kind: PatternKind::Wrapper { 
                pattern: pattern.into() 
            },
            action: Some(Action::Map(Arc::new(Mutex::new(transform)))),
        }
    }

    #[inline]
    pub fn ignore(pattern: impl Into<Box<Pattern<Input, Output, Failure>>>) -> Self {
        Self {
            kind: PatternKind::Wrapper { 
                pattern: pattern.into() 
            },
            action: Some(Action::Ignore),
        }
    }

    #[inline]
    pub fn skip(pattern: impl Into<Box<Pattern<Input, Output, Failure>>>) -> Self {
        Self {
            kind: PatternKind::Wrapper { 
                pattern: pattern.into() 
            },
            action: Some(Action::Skip),
        }
    }

    #[inline]
    pub fn error(
        pattern: impl Into<Box<Pattern<Input, Output, Failure>>>,
        function: Emitter<Input, Output, Failure>,
    ) -> Self {
        Self {
            kind: PatternKind::Wrapper { 
                pattern: pattern.into() 
            },
            action: Some(Action::Failure(function)),
        }
    }

    #[inline]
    pub fn conditional(
        pattern: impl Into<Box<Pattern<Input, Output, Failure>>>,
        found: Action<Input, Output, Failure>,
        missing: Action<Input, Output, Failure>,
    ) -> Self {
        Self {
            kind: PatternKind::Wrapper { 
                pattern: pattern.into() 
            },
            action: Some(Action::Trigger {
                found: Box::new(found),
                missing: Box::new(missing),
            }),
        }
    }

    #[inline]
    pub fn action(
        pattern: impl Into<Box<Pattern<Input, Output, Failure>>>,
        action: Action<Input, Output, Failure>,
    ) -> Self {
        Self {
            kind: PatternKind::Wrapper { 
                pattern: pattern.into() 
            },
            action: Some(action),
        }
    }

    #[inline]
    pub fn with_action(mut self, action: Action<Input, Output, Failure>) -> Self {
        self.action = Some(action);
        self
    }

    #[inline]
    pub fn with_ignore(mut self) -> Self {
        self.action = Some(Action::Ignore);
        self
    }

    #[inline]
    pub fn with_error(mut self, function: Emitter<Input, Output, Failure>) -> Self {
        self.action = Some(Action::Failure(function));
        self
    }

    #[inline]
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

    #[inline]
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

    #[inline]
    pub fn as_optional(&self) -> Self {
        Self::optional(Box::new(self.clone()))
    }

    #[inline]
    pub fn as_repeat(&self, min: usize, max: Option<usize>) -> Self {
        Self::repeat(Box::new(self.clone()), min, max)
    }
}

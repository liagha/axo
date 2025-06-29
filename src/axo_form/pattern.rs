use {
    super::{
        order::Order, 
        form::Form,
        functions::{
            Emitter, Evaluator,
            Predicate,
        },
    },
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
    },
};


#[derive(Clone)]
pub enum PatternKind<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    Alternative {
        patterns: Vec<Pattern<Input, Output, Failure>>
    },

    Predicate {
        function: Predicate<Input>, 
    },

    Deferred {
        function: Evaluator<Input, Output, Failure> 
    },

    Identical {
        value: Arc<dyn PartialEq<Input>>,  
    },

    Reject {
        pattern: Box<Pattern<Input, Output, Failure>>
    },

    Optional {
        pattern: Box<Pattern<Input, Output, Failure>> 
    },

    Repetition {
        pattern: Box<Pattern<Input, Output, Failure>>,
        minimum: usize,
        maximum: Option<usize>,
    },

    Sequence {
        patterns: Vec<Pattern<Input, Output, Failure>>,
    },

    Wrapper {
        pattern: Box<Pattern<Input, Output, Failure>> 
    },
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct Pattern<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub kind: PatternKind<Input, Output, Failure>,
    pub order: Option<Order<Input, Output, Failure>>,
}

impl<Input, Output, Failure> Pattern<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    #[inline]
    pub fn literal(value: impl PartialEq<Input> + 'static) -> Self {
        Self {
            kind: PatternKind::Identical {
                value: Arc::new(value),
            },
            order: None,
        }
    }

    #[inline]
    pub fn alternative(patterns: impl Into<Vec<Pattern<Input, Output, Failure>>>) -> Self {
        Self {
            kind: PatternKind::Alternative {
                patterns: patterns.into()
            },
            order: None,
        }
    }

    #[inline]
    pub fn sequence(patterns: impl Into<Vec<Pattern<Input, Output, Failure>>>) -> Self {
        Self {
            kind: PatternKind::Sequence { 
                patterns: patterns.into() 
            },
            order: None,
        }
    }

    #[inline]
    pub fn capture(
        identifier: usize,
        pattern: impl Into<Box<Pattern<Input, Output, Failure>>>,
    ) -> Self {
        Self {
            kind: pattern.into().kind,
            order: Some(Order::Capture { identifier }),
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
            order: None,
        }
    }

    #[inline]
    pub fn optional(pattern: impl Into<Box<Pattern<Input, Output, Failure>>>) -> Self {
        Self {
            kind: PatternKind::Optional { 
                pattern: pattern.into() 
            },
            order: None,
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
            order: None,
        }
    }

    #[inline]
    pub fn negate(pattern: impl Into<Box<Pattern<Input, Output, Failure>>>) -> Self {
        Self {
            kind: PatternKind::Reject {
                pattern: pattern.into()
            },
            order: None,
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
        order: Order<Input, Output, Failure>,
    ) -> Self {
        Self {
            kind: PatternKind::Wrapper { 
                pattern: pattern.into() 
            },
            order: Some(Order::Trigger {
                found: Order::perform(|| {}).into(),
                missing: order.into(),
            }),
        }
    }

    #[inline]
    pub fn lazy<F>(factory: F) -> Self
    where
        F: Fn() -> Pattern<Input, Output, Failure> + Send + Sync + 'static,
    {
        Self {
            kind: PatternKind::Deferred { 
                function: Arc::new(factory)
            },
            order: None,
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
            order: Some(Order::Convert(Arc::new(Mutex::new(transform)))),
        }
    }

    #[inline]
    pub fn ignore(pattern: impl Into<Box<Pattern<Input, Output, Failure>>>) -> Self {
        Self {
            kind: PatternKind::Wrapper { 
                pattern: pattern.into() 
            },
            order: Some(Order::Ignore),
        }
    }

    #[inline]
    pub fn skip(pattern: impl Into<Box<Pattern<Input, Output, Failure>>>) -> Self {
        Self {
            kind: PatternKind::Wrapper { 
                pattern: pattern.into() 
            },
            order: Some(Order::Skip),
        }
    }

    #[inline]
    pub fn conditional(
        pattern: impl Into<Box<Pattern<Input, Output, Failure>>>,
        found: Order<Input, Output, Failure>,
        missing: Order<Input, Output, Failure>,
    ) -> Self {
        Self {
            kind: PatternKind::Wrapper { 
                pattern: pattern.into() 
            },
            order: Some(Order::Trigger {
                found: Box::new(found),
                missing: Box::new(missing),
            }),
        }
    }

    #[inline]
    pub fn order(
        pattern: impl Into<Box<Pattern<Input, Output, Failure>>>,
        order: Order<Input, Output, Failure>,
    ) -> Self {
        Self {
            kind: PatternKind::Wrapper { 
                pattern: pattern.into() 
            },
            order: Some(order),
        }
    }

    #[inline]
    pub fn with_action(mut self, order: Order<Input, Output, Failure>) -> Self {
        self.order = Some(order);
        self
    }

    #[inline]
    pub fn with_ignore(mut self) -> Self {
        self.order = Some(Order::Ignore);
        self
    }

    #[inline]
    pub fn with_error(mut self, function: Emitter<Input, Output, Failure>) -> Self {
        self.order = Some(Order::Failure(function));
        self
    }

    #[inline]
    pub fn with_conditional(
        mut self,
        found: Order<Input, Output, Failure>,
        missing: Order<Input, Output, Failure>,
    ) -> Self {
        self.order = Some(Order::Trigger {
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
        self.order = Some(Order::Convert(Arc::new(Mutex::new(transform))));
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

use {
    super::{
        order::Order,
        form::Form,
        former::Record,
        helper::{
            Evaluator,
            Predicate,
        },
    },
    crate::{
        hash::Hash,
        format::Debug,
        artifact::Artifact,
        compiler::Context,
        thread::{
            Arc,
        },
        axo_cursor::{
            Spanned,
        },
    },
};
use crate::axo_form::order::Pulse;

#[derive(Clone)]
pub enum PatternKind<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    Alternative {
        patterns: Vec<Pattern<Input, Output, Failure>>,
        order: Order<Input, Output, Failure>,
        finish: Order<Input, Output, Failure>,
    },

    Predicate {
        function: Predicate<Input>,
        align: Order<Input, Output, Failure>,
        miss: Order<Input, Output, Failure>,
    },

    Deferred {
        function: Evaluator<Input, Output, Failure>,
        order: Order<Input, Output, Failure>,
    },

    Reject {
        pattern: Box<Pattern<Input, Output, Failure>>,
        align: Order<Input, Output, Failure>,
        miss: Order<Input, Output, Failure>,
    },

    Identical {
        value: Arc<dyn PartialEq<Input>>,
        align: Order<Input, Output, Failure>,
        miss: Order<Input, Output, Failure>,
    },

    Repetition {
        pattern: Box<Pattern<Input, Output, Failure>>,
        minimum: usize,
        maximum: Option<usize>,
        order: Order<Input, Output, Failure>,
        lack: Order<Input, Output, Failure>,
        exceed: Order<Input, Output, Failure>,
        finish: Order<Input, Output, Failure>,
    },

    Sequence {
        patterns: Vec<Pattern<Input, Output, Failure>>,
        order: Order<Input, Output, Failure>,
        finish: Order<Input, Output, Failure>,
    },

    Wrapper {
        pattern: Box<Pattern<Input, Output, Failure>>,
        order: Order<Input, Output, Failure>,
    },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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
                align: Order::multiple([
                    Order::Pulse(Pulse::Feast),
                    Order::Pulse(Pulse::Imitate),
                    Order::Pulse(Pulse::Align)
                ]),
                miss: Order::Pulse(Pulse::Pardon),
            },
            order: None,
        }
    }

    #[inline]
    pub fn negate(pattern: impl Into<Box<Pattern<Input, Output, Failure>>>) -> Self {
        Self {
            kind: PatternKind::Reject {
                pattern: pattern.into(),
                align: Order::multiple([
                    Order::Pulse(Pulse::Feast),
                    Order::Pulse(Pulse::Imitate),
                    Order::Pulse(Pulse::Align),
                ]),
                miss: Order::Pulse(Pulse::Pardon),
            },
            order: None,
        }
    }

    #[inline]
    pub fn predicate<F>(predicate: F) -> Self
    where
        F: Fn(&Input) -> bool + Send + Sync + 'static,
    {
        Self {
            kind: PatternKind::Predicate {
                function: Arc::new(predicate),
                align: Order::multiple([
                    Order::Pulse(Pulse::Feast),
                    Order::Pulse(Pulse::Imitate),
                    Order::Pulse(Pulse::Align),
                ]),
                miss: Order::Pulse(Pulse::Pardon),
            },
            order: None,
        }
    }

    #[inline]
    pub fn alternative(patterns: impl Into<Vec<Pattern<Input, Output, Failure>>>) -> Self {
        Self {
            kind: PatternKind::Alternative {
                patterns: patterns.into(),
                order: Order::inspect(|draft| {
                    match draft.record {
                        Record::Aligned => Order::multiple([
                            Order::Pulse(Pulse::Feast),
                            Order::Pulse(Pulse::Imitate),
                            Order::Pulse(Pulse::Align),
                            Order::Pulse(Pulse::Terminate),
                        ]),
                        Record::Failed => Order::Pulse(Pulse::Inject),
                        Record::Blank => Order::Pulse(Pulse::Proceed),
                    }
                }),
                finish: Order::inspect(|draft| {
                    if !draft.stack.is_empty() {
                        Order::multiple([
                            Order::Pulse(Pulse::Feast),
                            Order::Pulse(Pulse::Imitate),
                            Order::Pulse(Pulse::Fail),
                        ])
                    } else {
                        Order::Pulse(Pulse::Pardon)
                    }
                })
            },
            order: None,
        }
    }

    #[inline]
    pub fn lazy<F>(factory: F) -> Self
    where
        F: Fn() -> Pattern<Input, Output, Failure> + Send + Sync + 'static,
    {
        Self {
            kind: PatternKind::Deferred {
                function: Arc::new(factory),
                order: Order::Pulse(Pulse::Imitate),
            },
            order: None,
        }
    }

    #[inline]
    pub fn optional(pattern: impl Into<Box<Pattern<Input, Output, Failure>>>) -> Self {
        Self {
            kind: PatternKind::Wrapper {
                pattern: pattern.into(),
                order: Order::multiple([
                    Order::inspect(|draft| {
                        match draft.record {
                            Record::Aligned | Record::Failed => Order::Pulse(Pulse::Imitate),
                            Record::Blank => Order::Yawn,
                        }
                    }),
                    Order::Pulse(Pulse::Align)
                ]),
            },
            order: None,
        }
    }

    #[inline]
    pub fn sequence(patterns: impl Into<Vec<Pattern<Input, Output, Failure>>>) -> Self {
        Self {
            kind: PatternKind::Sequence {
                patterns: patterns.into(),
                order: Order::inspect(|draft| {
                    match draft.record {
                        Record::Aligned => Order::multiple([
                            Order::Pulse(Pulse::Feast),
                            Order::Pulse(Pulse::Inject),
                            Order::Pulse(Pulse::Align),
                            Order::Pulse(Pulse::Proceed),
                        ]),
                        Record::Failed => Order::multiple([
                            Order::Pulse(Pulse::Feast),
                            Order::Pulse(Pulse::Inject),
                            Order::Pulse(Pulse::Fail),
                            Order::Pulse(Pulse::Terminate),
                        ]),
                        Record::Blank => Order::multiple([
                            Order::Pulse(Pulse::Pardon),
                            Order::Pulse(Pulse::Terminate),
                        ]),
                    }
                }),
                finish: Order::multiple([
                    Order::Pulse(Pulse::Forge),
                ]),
            },
            order: None,
        }
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
                order: Order::inspect(|draft| {
                    match draft.record {
                        Record::Aligned | Record::Failed => Order::multiple([
                            Order::Pulse(Pulse::Feast),
                            Order::Pulse(Pulse::Inject),
                        ]),
                        Record::Blank => Order::Pulse(Pulse::Terminate),
                    }
                }),
                lack: Order::Pulse(Pulse::Pardon),
                exceed: Order::Pulse(Pulse::Terminate),
                finish: Order::multiple([
                    Order::Pulse(Pulse::Feast),
                    Order::Pulse(Pulse::Forge),
                    Order::Pulse(Pulse::Align),
                ]),
            },
            order: None,
        }
    }

    #[inline]
    pub fn order(
        pattern: impl Into<Box<Pattern<Input, Output, Failure>>>,
        order: Order<Input, Output, Failure>,
    ) -> Self {
        Self {
            kind: PatternKind::Wrapper {
                pattern: pattern.into(),
                order: Order::Pulse(Pulse::Imitate),
            },
            order: Some(order),
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
    pub fn capture(
        identifier: Artifact,
        pattern: impl Into<Box<Pattern<Input, Output, Failure>>>,
    ) -> Self {
        Self {
            kind: pattern.into().kind,
            order: Some(Order::Capture(identifier)),
        }
    }

    #[inline]
    pub fn required(
        pattern: impl Into<Box<Pattern<Input, Output, Failure>>>,
        error_order: Order<Input, Output, Failure>,
    ) -> Self {
        Self {
            kind: PatternKind::Wrapper {
                pattern: pattern.into(),
                order: Order::Pulse(Pulse::Imitate),
            },
            order: Some(Order::trigger(
                Order::Yawn,
                error_order,
            )),
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
                pattern: pattern.into(),
                order: Order::Pulse(Pulse::Imitate),
            },
            order: Some(Order::map(transform)),
        }
    }

    #[inline]
    pub fn ignore(pattern: impl Into<Box<Pattern<Input, Output, Failure>>>) -> Self {
        Self {
            kind: PatternKind::Wrapper {
                pattern: pattern.into(),
                order: Order::Pulse(Pulse::Imitate),
            },
            order: Some(Order::ignore()),
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
                pattern: pattern.into(),
                order: Order::Pulse(Pulse::Imitate),
            },
            order: Some(Order::trigger(found, missing)),
        }
    }

    #[inline]
    pub fn with_order(
        pattern: impl Into<Box<Pattern<Input, Output, Failure>>>,
        order: Order<Input, Output, Failure>,
    ) -> Self {
        Self {
            kind: PatternKind::Wrapper {
                pattern: pattern.into(),
                order: Order::Pulse(Pulse::Imitate),
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
        self.order = Some(Order::ignore());
        self
    }

    #[inline]
    pub fn with_error<F>(mut self, function: F) -> Self 
    where 
        F: Fn(&mut Context, Form<Input, Output, Failure>) -> Failure + Send + Sync + 'static
    {
        self.order = Some(Order::failure(function));
        self
    }

    #[inline]
    pub fn with_conditional(
        mut self,
        found: Order<Input, Output, Failure>,
        missing: Order<Input, Output, Failure>,
    ) -> Self {
        self.order = Some(Order::trigger(found, missing));
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
        self.order = Some(Order::map(transform));
        self
    }

    #[inline]
    pub fn with_perform<T>(mut self, executor: T) -> Self
    where
        T: FnMut() + Send + Sync + 'static,
    {
        self.order = Some(Order::perform(executor));
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

    #[inline]
    pub fn expect(self, error_message: &'static str) -> Self
    where
        Failure: From<&'static str>,
    {
        self.with_conditional(
            Order::Pulse(Pulse::Ignore),
            Order::failure(move |_ctx, _form| Failure::from(error_message))
        )
    }
}
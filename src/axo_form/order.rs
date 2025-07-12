use {
    super::{
        form::{Form, FormKind},
        former::Draft,
        helper::{
            Emitter, Executor,
            Transformer, Tweaker,
        },
    },
    crate::{
        artifact::Artifact,
        axo_cursor::{
            Peekable,
            Spanned,
        },
        axo_schema::{
            Formation
        },
        axo_parser::{Symbol, SymbolKind},
        compiler::{Context, Marked},
        format::Debug,
        hash::Hash,
        thread::{Arc, Mutex},
    }
};

#[derive(Clone)]
pub enum Order<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    Capture(Artifact),
    Convert(Transformer<Input, Output, Failure>),
    Failure(Emitter<Input, Output, Failure>),
    Ignore,
    Multiple(Vec<Order<Input, Output, Failure>>),
    Pardon,
    Perform(Executor),
    Remove,
    Skip,
    Trigger {
        found: Box<Order<Input, Output, Failure>>,
        missing: Box<Order<Input, Output, Failure>>,
    },
    Tweak(Tweaker<Input, Output, Failure>),
}

impl<Input, Output, Failure> Order<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    #[inline]
    pub fn execute<Source>(&self, source: &mut Source, draft: &mut Draft<Input, Output, Failure>)
    where
        Source: Peekable<Input> + Marked + ?Sized,
    {
        match self {
            Order::Convert(transform) => {
                if draft.is_aligned() {
                    let result = if let Ok(mut guard) = transform.lock() {
                        let result = guard(source.context_mut(), draft.form.clone());
                        drop(guard);
                        result
                    } else {
                        return;
                    };

                    let span = draft.form.span.clone();

                    match result {
                        Ok(mapped) => {
                            draft.form = mapped;
                        }
                        Err(error) => {
                            draft.form = Form::new(FormKind::Failure(error), span);
                            draft.fail();
                        }
                    }
                }
            }

            Order::Multiple(actions) => {
                for order in actions.iter() {
                    order.execute(source, draft);
                }
            }

            Order::Capture(identifier) => {
                if draft.is_aligned() {
                    let resolver = &mut source.context_mut().resolver;

                    let artifact = draft.form.clone().map(
                        |input| Artifact::new(input),
                        |output| Artifact::new(output),
                        |error| Artifact::new(error),
                    );

                    let symbol = Symbol::new(
                        SymbolKind::Formation(Formation::new(identifier.clone(), artifact)),
                        draft.form.span.clone(),
                    );

                    resolver.insert(symbol);
                }
            }

            Order::Ignore => {
                if draft.is_aligned() {
                    let span = draft.form.span.clone();
                    draft.ignore();
                    draft.form = Form::new(FormKind::<Input, Output, Failure>::Blank, span);
                }
            }

            Order::Skip => {
                if draft.is_aligned() {
                    let span = draft.form.span.clone();

                    draft.empty();
                    draft.form = Form::new(FormKind::<Input, Output, Failure>::Blank, span);
                }
            }

            Order::Perform(executor) => {
                if draft.is_aligned() {
                    if let Ok(mut guard) = executor.lock() {
                        guard();
                        drop(guard);
                    }
                }
            }

            Order::Failure(function) => {
                let span = draft.form.span.clone();

                let failure = function(source.context_mut(), draft.form.clone());

                let form = Form::new(FormKind::Failure(failure), span);
                draft.fail();
                draft.form = form;
            }

            Order::Trigger { found, missing } => {
                let chosen = if draft.is_aligned() {
                    found
                } else {
                    missing
                };

                draft.classifier.order = Some(*chosen.clone());

                chosen.execute(source, draft);
            },
            Order::Tweak(tweaker) => {
                tweaker(draft);
            }
            Order::Remove => {
                source.remove(draft.marker);
            }
            Order::Pardon => {
                draft.empty();
            }
        }
    }

    #[inline]
    pub fn failure<T>(transform: T) -> Self
    where
        T: Fn(&mut Context, Form<Input, Output, Failure>) -> Failure + Send + Sync + 'static,
    {
        Self::Failure(Arc::new(transform))
    }

    #[inline]
    pub fn map<T>(transformer: T) -> Self
    where
        T: FnMut(&mut Context, Form<Input, Output, Failure>) -> Result<Form<Input, Output, Failure>, Failure>
        + Send
        + Sync
        + 'static,
    {
        Self::Convert(Arc::new(Mutex::new(transformer)))
    }

    #[inline]
    pub fn perform<T>(executor: T) -> Self
    where
        T: FnMut() + Send + Sync + 'static,
    {
        Self::Perform(Arc::new(Mutex::new(executor)))
    }

    #[inline]
    pub fn capture(identifier: Artifact) -> Self {
        Self::Capture(identifier)
    }

    #[inline]
    pub fn ignore() -> Self {
        Self::Ignore
    }

    #[inline]
    pub fn skip() -> Self {
        Self::Skip
    }

    #[inline]
    pub fn multiple(actions: Vec<Self>) -> Self {
        Self::Multiple(actions)
    }

    #[inline]
    pub fn trigger(found: Self, missing: Self) -> Self {
        Self::Trigger {
            found: Box::new(found),
            missing: Box::new(missing),
        }
    }

    pub fn then(self, next: Self) -> Self {
        Self::multiple(vec![self, next])
    }

    #[inline]
    pub fn with_capture(self, identifier: Artifact) -> Self {
        self.then(Self::capture(identifier))
    }

    #[inline]
    pub fn with_ignore(self) -> Self {
        self.then(Self::ignore())
    }
}
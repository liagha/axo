use crate::{
    artifact::Artifact,
    axo_cursor::Peekable,
    axo_form::{
        form::{Form, FormKind},
        former::{Draft, Record},
    },
    axo_parser::{Item, ItemKind},
    compiler::{Context, Marked},
    format::Debug,
    hash::Hash,
    thread::{Arc, Mutex},
};

/// A transformer function that processes a form and returns either a successful output or a failure.
/// Takes a mutable context and a form, returning a Result containing the transformed output or an error.
pub type Transformer<Input, Output, Failure> = Arc<
    Mutex<
        dyn FnMut(&mut Context, Form<Input, Output, Failure>) -> Result<Output, Failure>
            + Send
            + Sync,
    >,
>;

/// An emitter function that generates a failure from a span location.
/// Used to create error messages or failure states at specific positions in the input.
pub type Emitter<Input, Output, Failure> =
    Arc<Mutex<dyn FnMut(&mut Context, Form<Input, Output, Failure>) -> Failure + Send + Sync>>;

/// An executor function that performs side effects without returning a value.
/// Used for logging, debugging, or other operations that don't transform the form.
pub type Executor = Arc<Mutex<dyn FnMut() -> () + Send + Sync>>;

/// An inspector function that examines a form and returns an action to be performed.
/// Used for dynamic action selection based on form content.
pub type Inspector<Input, Output, Failure> = Arc<
    Mutex<dyn FnMut(Form<Input, Output, Failure>) -> Action<Input, Output, Failure> + Send + Sync>,
>;

/// Actions define what happens when patterns match during form processing.
/// Each action can transform, execute side effects, or control the flow of processing.
#[derive(Clone)]
pub enum Action<Input, Output, Failure>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    /// Transform the input form into an output form using the provided transformer function.
    /// If transformation fails, the form becomes a failure form.
    Map(Transformer<Input, Output, Failure>),

    /// Execute a side effect without modifying the form.
    /// The form passes through unchanged after execution.
    Perform(Executor),

    /// Inspect the form and dynamically choose an action based on its content.
    /// The inspector function examines the form and returns the action to perform.
    Inspect(Inspector<Input, Output, Failure>),

    /// Execute multiple actions in sequence.
    /// Each action is applied to the result of the previous action.
    Multiple(Vec<Action<Input, Output, Failure>>),

    /// Conditional execution based on whether the form has content.
    /// Executes `found` if the form contains input/output data, `missing` if it's empty/failed.
    Trigger {
        found: Box<Action<Input, Output, Failure>>,
        missing: Box<Action<Input, Output, Failure>>,
    },

    /// Capture the current form state and store it in the resolver with the given identifier.
    /// The form is converted to an artifact and stored for later retrieval.
    Capture {
        identifier: usize,
    },

    /// Ignore the current form and replace it with an empty form.
    /// Used to discard unwanted matches while continuing processing.
    Ignore,

    Skip,

    /// Generate a failure form using the provided emitter function.
    /// The emitter receives the current span and produces a failure value.
    Failure(Emitter<Input, Output, Failure>),
}

impl<Input, Output, Failure> Action<Input, Output, Failure>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub fn apply<Source>(&self, source: &mut Source, draft: &mut Draft<Input, Output, Failure>)
    where
        Source: Peekable<Input> + Marked,
    {
        let result = match self {
            Action::Inspect(inspector) => {
                let mut guard = inspector.lock().unwrap();
                let action = guard(draft.form.clone());
                drop(guard);

                draft.pattern.action = Some(action.clone());
            }

            Action::Multiple(actions) => {
                for action in actions {
                    action.apply(source, draft);
                }
            }

            Action::Trigger { found, missing } => {
                let chosen = if draft.record == Record::Aligned {
                    found
                } else {
                    missing
                };

                draft.pattern.action = Some(*chosen.clone());
            }

            Action::Ignore => {}

            Action::Skip => {
                draft.record = Record::Skipped;
            }

            Action::Failure(_) => {
                draft.record = Record::Failed;
            }

            _ => {}
        };

        result
    }

    pub fn execute<Source>(&self, source: &mut Source, draft: &mut Draft<Input, Output, Failure>)
    where
        Source: Peekable<Input> + Marked,
    {
        let result = match self {
            Action::Map(transform) => {
                if draft.record != Record::Aligned {
                    return;
                } else {
                }

                let mut guard = transform.lock().unwrap();
                let transformed = guard(source.context_mut(), draft.form.clone());
                drop(guard);

                let span = draft.form.span.clone();

                match transformed {
                    Ok(output) => {
                        let mapped = Form::new(FormKind::Output(output), span);

                        draft.form = mapped;
                    }
                    Err(error) => {
                        draft.form = Form::new(FormKind::Failure(error), span);
                    }
                }
            }

            Action::Inspect(inspector) => {
                let mut guard = inspector.lock().unwrap();
                let action = guard(draft.form.clone());
                drop(guard);

                draft.pattern.action = Some(action.clone());

                action.execute(source, draft);
            }

            Action::Multiple(actions) => {
                for action in actions.iter() {
                    action.execute(source, draft);
                }
            }

            Action::Trigger { found, missing } => {
                let chosen = if draft.record == Record::Aligned {
                    found
                } else {
                    missing
                };

                draft.pattern.action = Some(*chosen.clone());

                chosen.execute(source, draft);
            }

            Action::Capture { identifier } => {
                let resolver = &mut source.context_mut().resolver;

                let artifact = draft.form.clone().map(
                    |input| Artifact::new(input),
                    |output| Artifact::new(output),
                    |error| Artifact::new(error),
                );

                let item = Item::new(
                    ItemKind::Formed {
                        identifier: *identifier,
                        form: artifact,
                    },
                    draft.form.span.clone(),
                );

                resolver.insert(item);
            }

            Action::Ignore => {
                let span = draft.form.span.clone();

                draft.form = Form::new(FormKind::<Input, Output, Failure>::Blank, span);
            }

            Action::Skip => {
                let span = draft.form.span.clone();

                draft.form = Form::new(FormKind::<Input, Output, Failure>::Blank, span);
            }

            Action::Perform(executor) => {
                let mut guard = executor.lock().unwrap();
                guard();
                drop(guard);
            }

            Action::Failure(function) => {
                let span = draft.form.span.clone();

                let mut guard = function.lock().unwrap();
                let form = Form::new(
                    FormKind::Failure(guard(source.context_mut(), draft.form.clone())),
                    span.clone(),
                );

                draft.form = form.clone();
                draft.form = Form::new(FormKind::<Input, Output, Failure>::Blank, span);
            }
        };

        result
    }

    pub fn failure<T>(transform: T) -> Self
    where
        T: FnMut(&mut Context, Form<Input, Output, Failure>) -> Failure + Send + Sync + 'static,
    {
        Self::Failure(Arc::new(Mutex::new(transform)))
    }

    pub fn map<T>(transformer: T) -> Self
    where
        T: FnMut(&mut Context, Form<Input, Output, Failure>) -> Result<Output, Failure>
            + Send
            + Sync
            + 'static,
    {
        Self::Map(Arc::new(Mutex::new(transformer)))
    }

    pub fn inspect<T>(inspector: T) -> Self
    where
        T: FnMut(Form<Input, Output, Failure>) -> Action<Input, Output, Failure>
            + Send
            + Sync
            + 'static,
    {
        Self::Inspect(Arc::new(Mutex::new(inspector)))
    }

    pub fn perform<T>(executor: T) -> Self
    where
        T: FnMut() + Send + Sync + 'static,
    {
        Self::Perform(Arc::new(Mutex::new(executor)))
    }

    pub fn capture(identifier: usize) -> Self {
        Self::Capture { identifier }
    }

    pub fn ignore() -> Self {
        Self::Ignore
    }

    pub fn multiple(actions: Vec<Self>) -> Self {
        Self::Multiple(actions)
    }

    pub fn chain<I>(actions: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        Self::Multiple(actions.into_iter().collect())
    }

    pub fn trigger(found: Self, missing: Self) -> Self {
        Self::Trigger {
            found: Box::new(found),
            missing: Box::new(missing),
        }
    }

    pub fn when_found(action: Self) -> Self {
        Self::trigger(action, Self::ignore())
    }

    pub fn when_missing(action: Self) -> Self {
        Self::trigger(Self::ignore(), action)
    }

    pub fn then(self, next: Self) -> Self {
        Self::multiple(vec![self, next])
    }

    pub fn with_capture(self, identifier: usize) -> Self {
        self.then(Self::capture(identifier))
    }

    pub fn with_ignore(self) -> Self {
        self.then(Self::ignore())
    }
}

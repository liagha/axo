use {
    crate::{
        hash::Hash,
        format::Debug,
        compiler::Context,
        thread::{Arc, Mutex},

        axo_span::Span,
        axo_form::form::Form,
    }
};

pub type Transformer<Input, Output, Failure> = Arc<Mutex<dyn FnMut(&mut Context, Form<Input, Output, Failure>) -> Result<Output, Failure> + Send + Sync>>;
pub type Emitter<Failure> = Arc<Mutex<dyn FnMut(Span) -> Failure + Send + Sync>>;
pub type Executor = Arc<Mutex<dyn FnMut() -> () + Send + Sync>>;
pub type Inspector<Input, Output, Failure> = Arc<Mutex<dyn FnMut(Form<Input, Output, Failure>) -> Action<Input, Output, Failure> + Send + Sync>>;

#[derive(Clone)]
pub enum Action<Input, Output, Failure>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    Map(Transformer<Input, Output, Failure>),
    Execute(Executor),
    Inspect(Inspector<Input, Output, Failure>),
    Multiple(Vec<Action<Input, Output, Failure>>),
    Trigger {
        found: Box<Action<Input, Output, Failure>>,
        missing: Box<Action<Input, Output, Failure>>,
    },
    Capture {
        identifier: usize,
    },
    Ignore,
    Failure(Emitter<Failure>),
}

impl<Input, Output, Failure> Action<Input, Output, Failure>
where
    Input: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    pub fn failure<T>(transform: T) -> Self
    where T: FnMut(Span) -> Failure + Send + Sync + 'static,
    {
        Self::Failure(Arc::new(Mutex::new(transform)))
    }

    pub fn map<T>(transformer: T) -> Self
    where T: FnMut(&mut Context, Form<Input, Output, Failure>) -> Result<Output, Failure> + Send + Sync + 'static,
    {
        Self::Map(Arc::new(Mutex::new(transformer)))
    }

    pub fn inspect<T>(inspector: T) -> Self
    where T: FnMut(Form<Input, Output, Failure>) -> Action<Input, Output, Failure> + Send + Sync + 'static,
    {
        Self::Inspect(Arc::new(Mutex::new(inspector)))
    }
    
    pub fn map_simple<T>(mut transformer: T) -> Self
    where
        T: FnMut(Form<Input, Output, Failure>) -> Output + Send + Sync + 'static,
        Failure: Default,
    {
        Self::map(move |_ctx, form| Ok(transformer(form)))
    }

    pub fn extract_input<T>(mut extractor: T) -> Self
    where
        T: FnMut(&Input) -> Output + Send + Sync + 'static,
        Failure: Default,
    {
        Self::map(move |_ctx, form| {
            match form.kind {
                crate::axo_form::form::FormKind::Input(ref input) => Ok(extractor(input)),
                _ => Err(Failure::default()),
            }
        })
    }

    pub fn execute<T>(executor: T) -> Self
    where T: FnMut() + Send + Sync + 'static,
    {
        Self::Execute(Arc::new(Mutex::new(executor)))
    }

    pub fn log<S: Into<String>>(message: S) -> Self {
        let msg = message.into();
        Self::execute(move || {
            log::info!("{}", msg);
        })
    }

    pub fn debug_form() -> Self {
        Self::execute(|| {
            log::debug!("Form processed");
        })
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
    where I: IntoIterator<Item = Self>,
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

    pub fn with_log<S: Into<String>>(self, message: S) -> Self {
        self.then(Self::log(message))
    }

    pub fn with_ignore(self) -> Self {
        self.then(Self::ignore())
    }
}
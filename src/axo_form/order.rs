use {
    super::{
        form::Form,
        former::Draft,
        helper::{
            Emitter, Executor,
            Inspector, Transformer,
        },
    },
    crate::{
        axo_cursor::{
            Peekable,
        },
        format::Debug,
        hash::Hash,
        thread::{Arc, Mutex},
    }
};
use crate::axo_internal::compiler::{Context, Marked};

#[derive(Clone)]
pub enum Order<Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    Align,
    Branch {
        found: Box<Order<Input, Output, Failure>>,
        missing: Box<Order<Input, Output, Failure>>,
    },
    Fail(Emitter<Input, Output, Failure>),
    Ignore,
    Inspect(Inspector<Input, Output, Failure>),
    Multiple(Vec<Order<Input, Output, Failure>>),
    Panic(Emitter<Input, Output, Failure>),
    Pardon,
    Perform(Executor),
    Skip,
    Transform(Transformer<Input, Output, Failure>),
}

impl<Input, Output, Failure> Order<Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    #[inline]
    pub fn execute<Source>(&self, source: &mut Source, draft: &mut Draft<Input, Output, Failure>)
    where
        Source: Peekable<Input> + Marked + ?Sized,
    {
        match self {
            Order::Align => {
                draft.align();
            }

            Order::Branch { found, missing } => {
                let chosen = if draft.is_aligned() {
                    found
                } else {
                    missing
                };

                draft.classifier.order = Some(*chosen.clone());

                chosen.execute(source, draft);
            },

            Order::Fail(function) => {
                let failure = function(source.context_mut(), draft.form.clone());

                draft.fail();
                draft.form = Form::Failure(failure);
            }

            Order::Ignore => {
                if draft.is_aligned() {
                    draft.ignore();
                    draft.form = Form::<Input, Output, Failure>::Blank;
                }
            }
            
            Order::Inspect(inspector) => {
                let order = inspector(draft.to_owned());
                
                order.execute(source, draft);
            }

            Order::Multiple(actions) => {
                for order in actions.iter() {
                    order.execute(source, draft);
                }
            }

            Order::Panic(function) => {
                let failure = function(source.context_mut(), draft.form.clone());

                let form = Form::Failure(failure);
                draft.panic();
                draft.form = form;
            }

            Order::Pardon => {
                draft.empty();
            }

            Order::Perform(executor) => {
                if draft.is_aligned() {
                    if let Ok(mut guard) = executor.lock() {
                        guard();
                        drop(guard);
                    }
                }
            }

            Order::Skip => {
                if draft.is_aligned() {
                    draft.empty();
                    draft.form = Form::<Input, Output, Failure>::Blank;
                }
            }

            Order::Transform(transform) => {
                if draft.is_aligned() {
                    let result = if let Ok(mut guard) = transform.lock() {
                        let result = guard(source.context_mut(), draft.form.clone());
                        drop(guard);
                        result
                    } else {
                        return;
                    };

                    match result {
                        Ok(mapped) => {
                            draft.form = mapped;
                        }
                        Err(error) => {
                            draft.fail();
                            draft.form = Form::Failure(error);
                        }
                    }
                }
            }
        }
    }

    pub fn convert<T>(transformer: T) -> Self
    where
        T: FnMut(&mut Context, Form<Input, Output, Failure>) -> Result<Form<Input, Output, Failure>, Failure> + Send + Sync + 'static
    {
        Self::Transform(Arc::new(Mutex::new(transformer)))
    }

    pub fn fail<T>(emitter: T) -> Self
    where
        T: Fn(&mut Context, Form<Input, Output, Failure>) -> Failure + Send + Sync + 'static,
    {
        Self::Fail(Arc::new(emitter))
    }

    pub fn panic<T>(emitter: T) -> Self
    where
        T: Fn(&mut Context, Form<Input, Output, Failure>) -> Failure + Send + Sync + 'static,
    {
        Self::Panic(Arc::new(emitter))
    }

    pub fn ignore() -> Self {
        Self::Ignore
    }
    
    pub fn inspect<T>(inspector: T) -> Self 
    where 
        T: Fn(Draft<Input, Output, Failure>) -> Order<Input, Output, Failure> + Send + Sync + 'static
    {
        Self::Inspect(Arc::new(inspector))
    }

    pub fn chain(orders: Vec<Self>) -> Self {
        Self::Multiple(orders)
    }

    pub fn pardon() -> Self {
        Self::Pardon
    }

    pub fn perform<T>(executor: T) -> Self
    where
        T: FnMut() + Send + Sync + 'static,
    {
        Self::Perform(Arc::new(Mutex::new(executor)))
    }

    pub fn skip() -> Self {
        Self::Skip
    }

    pub fn branch(found: Self, missing: Self) -> Self {
        Self::Branch {
            found: Box::new(found),
            missing: Box::new(missing),
        }
    }

    pub fn then(self, next: Self) -> Self {
        Self::chain(vec![self, next])
    }

    pub fn with_ignore(self) -> Self {
        self.then(Self::ignore())
    }

    pub fn with_skip(self) -> Self {
        self.then(Self::skip())
    }

    pub fn with_pardon(self) -> Self {
        self.then(Self::pardon())
    }
}
use {
    super::{
        Formable,
        form::Form,
        classifier::{
            Classifier,
        },
        former::{
            Composer, Draft
        },
        helper::{
            Emitter, Executor,
            Inspector, Transformer,
        },
    },
    crate::{
        axo_internal::{
            compiler::{
                Registry
            },
        },
        thread::{Arc, Mutex},
    }
};

pub trait Order<Input: Formable, Output: Formable, Failure: Formable> {
    fn order(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>);
}

pub struct Align;

impl<Input: Formable, Output: Formable, Failure: Formable> Order<Input, Output, Failure> for Align {
    #[inline]
    fn order(&self, _composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        draft.align();
    }
}

pub struct Branch<Input: Formable, Output: Formable, Failure: Formable> {
    pub found: Arc<dyn Order<Input, Output, Failure>>,
    pub missing: Arc<dyn Order<Input, Output, Failure>>,
}

impl<Input: Formable, Output: Formable, Failure: Formable> Order<Input, Output, Failure> for Branch<Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        let chosen = if draft.is_aligned() {
            &self.found
        } else {
            &self.missing
        };

        chosen.order(composer, draft);
    }
}

pub struct Fail<Input: Formable, Output: Formable, Failure: Formable> {
    pub emitter: Emitter<Input, Output, Failure>,
}

impl<Input: Formable, Output: Formable, Failure: Formable> Order<Input, Output, Failure> for Fail<Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        let failure = (self.emitter)(composer.source.registry_mut(), draft.form.clone());

        draft.fail();
        draft.form = Form::Failure(failure);
    }
}

pub struct Ignore;

impl<Input: Formable, Output: Formable, Failure: Formable> Order<Input, Output, Failure> for Ignore {
    #[inline]
    fn order(&self, _composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        if draft.is_aligned() {
            draft.ignore();
            draft.form = Form::<Input, Output, Failure>::Blank;
        }
    }
}

pub struct Inspect<Input: Formable, Output: Formable, Failure: Formable> {
    pub inspector: Inspector<Input, Output, Failure>,
}

impl<Input: Formable, Output: Formable, Failure: Formable> Order<Input, Output, Failure> for Inspect<Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        let order = (self.inspector)(draft.to_owned());

        order.order(composer, draft);
    }
}

pub struct Multiple<Input: Formable, Output: Formable, Failure: Formable> {
    pub orders: Vec<Arc<dyn Order<Input, Output, Failure>>>
}

impl<Input: Formable, Output: Formable, Failure: Formable> Order<Input, Output, Failure> for Multiple<Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        for order in self.orders.iter() {
            order.order(composer, draft);
        }
    }
}

pub struct Panic<Input: Formable, Output: Formable, Failure: Formable> {
    pub emitter: Emitter<Input, Output, Failure>,
}

impl<Input: Formable, Output: Formable, Failure: Formable> Order<Input, Output, Failure> for Panic<Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        let failure = (self.emitter)(composer.source.registry_mut(), draft.form.clone());

        let form = Form::Failure(failure);
        draft.panic();
        draft.form = form;
    }
}

pub struct Pardon;

impl<Input: Formable, Output: Formable, Failure: Formable> Order<Input, Output, Failure> for Pardon {
    #[inline]
    fn order(&self, _composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        draft.empty();
    }
}

pub struct Perform {
    pub performer: Executor,
}

impl<Input: Formable, Output: Formable, Failure: Formable> Order<Input, Output, Failure> for Perform {
    #[inline]
    fn order(&self, _composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        if draft.is_aligned() {
            if let Ok(mut guard) = self.performer.lock() {
                guard();
                drop(guard);
            }
        }
    }
}

pub struct Skip;

impl<Input: Formable, Output: Formable, Failure: Formable> Order<Input, Output, Failure> for Skip {
    #[inline]
    fn order(&self, _composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        if draft.is_aligned() {
            draft.empty();
            draft.form = Form::<Input, Output, Failure>::Blank;
        }
    }
}

pub struct Transform<Input: Formable, Output: Formable, Failure: Formable> {
    pub transformer: Transformer<Input, Output, Failure>,
}

impl<Input: Formable, Output: Formable, Failure: Formable> Order<Input, Output, Failure> for Transform<Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        if draft.is_aligned() {
            let result = if let Ok(mut guard) = self.transformer.lock() {
                let result = guard(composer.source.registry_mut(), draft.form.clone());
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

impl<Input: Formable, Output: Formable, Failure: Formable> Classifier<Input, Output, Failure> {
    #[inline]
    pub fn transform<T>(transformer: T) -> Arc<dyn Order<Input, Output, Failure>>
    where
        T: FnMut(&mut Registry, Form<Input, Output, Failure>) -> Result<Form<Input, Output, Failure>, Failure> + Send + Sync + 'static
    {
        Arc::new(Transform { transformer: Arc::new(Mutex::new(transformer))})
    }

    #[inline]
    pub fn fail<T>(emitter: T) -> Arc<dyn Order<Input, Output, Failure>>
    where
        T: Fn(&mut Registry, Form<Input, Output, Failure>) -> Failure + Send + Sync + 'static,
    {
        Arc::new(Fail { emitter: Arc::new(emitter) })
    }

    #[inline]
    pub fn panic<T>(emitter: T) -> Arc<dyn Order<Input, Output, Failure>>
    where
        T: Fn(&mut Registry, Form<Input, Output, Failure>) -> Failure + Send + Sync + 'static,
    {
        Arc::new(Panic { emitter: Arc::new(emitter) })
    }

    #[inline]
    pub fn ignore() -> Arc<dyn Order<Input, Output, Failure>> {
        Arc::new(Ignore)
    }

    #[inline]
    pub fn inspect<T>(inspector: T) -> Arc<dyn Order<Input, Output, Failure>>
    where
        T: Fn(Draft<Input, Output, Failure>) -> Arc<dyn Order<Input, Output, Failure>> + Send + Sync + 'static
    {
        Arc::new(Inspect { inspector: Arc::new(inspector) })
    }

    #[inline]
    pub fn multiple(orders: Vec<Arc<dyn Order<Input, Output, Failure>>>) -> Arc<dyn Order<Input, Output, Failure>> {
        Arc::new(Multiple { orders })
    }

    #[inline]
    pub fn pardon() -> Arc<dyn Order<Input, Output, Failure>> {
        Arc::new(Pardon)
    }

    #[inline]
    pub fn perform<T>(executor: T) -> Arc<dyn Order<Input, Output, Failure>>
    where
        T: FnMut() + Send + Sync + 'static,
    {
        Arc::new(Perform { performer: Arc::new(Mutex::new(executor))})
    }

    #[inline]
    pub fn skip() -> Arc<dyn Order<Input, Output, Failure>> {
        Arc::new(Skip)
    }

    #[inline]
    pub fn branch(found: Arc<dyn Order<Input, Output, Failure>>, missing: Arc<dyn Order<Input, Output, Failure>>) -> Arc<dyn Order<Input, Output, Failure>> {
        Arc::new(Branch { found, missing })
    }
}
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

pub trait Order<'order, Input: Formable, Output: Formable, Failure: Formable> {
    fn order(&self, composer: &mut Composer<'order, Input, Output, Failure>, draft: &mut Draft<'order, Input, Output, Failure>);
}

pub struct Align;

impl<'align, Input: Formable, Output: Formable, Failure: Formable> Order<'align, Input, Output, Failure> for Align {
    #[inline]
    fn order(&self, _composer: &mut Composer<'align, Input, Output, Failure>, draft: &mut Draft<'align, Input, Output, Failure>) {
        draft.set_align();
    }
}

pub struct Branch<'branch, Input: Formable, Output: Formable, Failure: Formable> {
    pub found: Arc<dyn Order<'branch, Input, Output, Failure>>,
    pub missing: Arc<dyn Order<'branch, Input, Output, Failure>>,
}

impl<'branch, Input: Formable, Output: Formable, Failure: Formable> Order<'branch, Input, Output, Failure> for Branch<'branch, Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Composer<'branch, Input, Output, Failure>, draft: &mut Draft<'branch, Input, Output, Failure>) {
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

impl<'fail, Input: Formable, Output: Formable, Failure: Formable> Order<'fail, Input, Output, Failure> for Fail<Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        let failure = (self.emitter)(composer.source.registry_mut(), draft.form.clone());

        draft.set_fail();
        draft.form = Form::Failure(failure);
    }
}

pub struct Ignore;

impl<'ignore, Input: Formable, Output: Formable, Failure: Formable> Order<'ignore, Input, Output, Failure> for Ignore {
    #[inline]
    fn order(&self, _composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        if draft.is_aligned() {
            draft.set_ignore();
            draft.form = Form::<Input, Output, Failure>::Blank;
        }
    }
}

pub struct Inspect<'inspector, Input: Formable, Output: Formable, Failure: Formable> {
    pub inspector: Inspector<'inspector, Input, Output, Failure>,
}

impl<'inspector, Input: Formable, Output: Formable, Failure: Formable> Order<'inspector, Input, Output, Failure> for Inspect<'inspector, Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        let order = (self.inspector)(draft.to_owned());

        order.order(composer, draft);
    }
}

pub struct Multiple<'multiple, Input: Formable, Output: Formable, Failure: Formable> {
    pub orders: Vec<Arc<dyn Order<'multiple, Input, Output, Failure>>>
}

impl<'multiple, Input: Formable, Output: Formable, Failure: Formable> Order<'multiple, Input, Output, Failure> for Multiple<'multiple, Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Composer<'multiple, Input, Output, Failure>, draft: &mut Draft<'multiple, Input, Output, Failure>) {
        for order in self.orders.iter() {
            order.order(composer, draft);
        }
    }
}

pub struct Panic<Input: Formable, Output: Formable, Failure: Formable> {
    pub emitter: Emitter<Input, Output, Failure>,
}

impl<'panic, Input: Formable, Output: Formable, Failure: Formable> Order<'panic, Input, Output, Failure> for Panic<Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        let failure = (self.emitter)(composer.source.registry_mut(), draft.form.clone());

        let form = Form::Failure(failure);
        draft.set_panic();
        draft.form = form;
    }
}

pub struct Pardon;

impl<'pardon, Input: Formable, Output: Formable, Failure: Formable> Order<'pardon, Input, Output, Failure> for Pardon {
    #[inline]
    fn order(&self, _composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        draft.set_empty();
    }
}

pub struct Perform {
    pub performer: Executor,
}

impl<'perform, Input: Formable, Output: Formable, Failure: Formable> Order<'perform, Input, Output, Failure> for Perform {
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

impl<'skip, Input: Formable, Output: Formable, Failure: Formable> Order<'skip, Input, Output, Failure> for Skip {
    #[inline]
    fn order(&self, _composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        if draft.is_aligned() {
            draft.set_empty();
            draft.form = Form::<Input, Output, Failure>::Blank;
        }
    }
}

pub struct Transform<Input: Formable, Output: Formable, Failure: Formable> {
    pub transformer: Transformer<Input, Output, Failure>,
}

impl<'transform, Input: Formable, Output: Formable, Failure: Formable> Order<'transform, Input, Output, Failure> for Transform<Input, Output, Failure> {
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
                    draft.set_fail();
                    draft.form = Form::Failure(error);
                }
            }
        }
    }
}

impl<'classifier, Input: Formable, Output: Formable, Failure: Formable> Classifier<'classifier, Input, Output, Failure> {
    #[inline]
    pub fn transform<T>(transformer: T) -> Arc<dyn Order<'classifier, Input, Output, Failure>>
    where
        T: FnMut(&mut Registry, Form<Input, Output, Failure>) -> Result<Form<Input, Output, Failure>, Failure> + Send + Sync + 'static,
    {
        Arc::new(Transform { transformer: Arc::new(Mutex::new(transformer))})
    }

    #[inline]
    pub fn fail<T>(emitter: T) -> Arc<dyn Order<'classifier, Input, Output, Failure>>
    where
        T: Fn(&mut Registry, Form<Input, Output, Failure>) -> Failure + Send + Sync + 'static,
    {
        Arc::new(Fail { emitter: Arc::new(emitter) })
    }

    #[inline]
    pub fn panic<T>(emitter: T) -> Arc<dyn Order<'classifier, Input, Output, Failure>>
    where
        T: Fn(&mut Registry, Form<Input, Output, Failure>) -> Failure + Send + Sync + 'static,
    {
        Arc::new(Panic { emitter: Arc::new(emitter) })
    }

    #[inline]
    pub fn ignore() -> Arc<dyn Order<'classifier, Input, Output, Failure>> {
        Arc::new(Ignore)
    }

    #[inline]
    pub fn inspect<T>(inspector: T) -> Arc<dyn Order<'classifier, Input, Output, Failure>>
    where
        T: Fn(Draft<Input, Output, Failure>) -> Arc<dyn Order<Input, Output, Failure>> + Send + Sync + 'static
    {
        Arc::new(Inspect { inspector: Arc::new(inspector) })
    }

    #[inline]
    pub fn multiple(orders: Vec<Arc<dyn Order<'classifier, Input, Output, Failure> + 'static>>) -> Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier> {
        Arc::new(Multiple { orders })
    }

    #[inline]
    pub fn pardon() -> Arc<dyn Order<'classifier, Input, Output, Failure>> {
        Arc::new(Pardon)
    }

    #[inline]
    pub fn perform<T>(executor: T) -> Arc<dyn Order<'classifier, Input, Output, Failure>>
    where
        T: FnMut() + Send + Sync + 'static,
    {
        Arc::new(Perform { performer: Arc::new(Mutex::new(executor))})
    }

    #[inline]
    pub fn skip() -> Arc<dyn Order<'classifier, Input, Output, Failure>> {
        Arc::new(Skip)
    }

    #[inline]
    pub fn branch(found: Arc<dyn Order<'classifier, Input, Output, Failure> + 'static>, missing: Arc<dyn Order<'classifier, Input, Output, Failure> + 'static>) -> Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier> {
        Arc::new(Branch { found, missing })
    }
}
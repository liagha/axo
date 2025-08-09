use {
    super::{
        classifier::Classifier,
        form::Form,
        former::{Composer, Draft},
        helper::{Formable, Emitter, Performer, Inspector, Transformer},
    },
    crate::{
        data::{
            thread::{Arc, Mutex},
        },
        internal::{
            compiler::{
                Registry
            },
        },
    }
};

pub trait Order<'order, Input: Formable<'order>, Output: Formable<'order>, Failure: Formable<'order>> {
    fn order(&self, composer: &mut Composer<'_, 'order, Input, Output, Failure>, draft: &mut Draft<'order, Input, Output, Failure>);
}

pub struct Align;

impl<'align, Input: Formable<'align>, Output: Formable<'align>, Failure: Formable<'align>> Order<'align, Input, Output, Failure> for Align {
    #[inline]
    fn order(&self, _composer: &mut Composer<'_, 'align, Input, Output, Failure>, draft: &mut Draft<'align, Input, Output, Failure>) {
        draft.set_align();
    }
}

pub struct Branch<'branch, Input: Formable<'branch>, Output: Formable<'branch>, Failure: Formable<'branch>> {
    pub found: Arc<dyn Order<'branch, Input, Output, Failure> + 'branch>,
    pub missing: Arc<dyn Order<'branch, Input, Output, Failure> + 'branch>,
}

impl<'branch, Input: Formable<'branch>, Output: Formable<'branch>, Failure: Formable<'branch>> Order<'branch, Input, Output, Failure> for Branch<'branch, Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Composer<'_, 'branch, Input, Output, Failure>, draft: &mut Draft<'branch, Input, Output, Failure>) {
        let chosen = if draft.is_aligned() {
            &self.found
        } else {
            &self.missing
        };

        chosen.order(composer, draft);
    }
}

pub struct Fail<'fail, Input: Formable<'fail>, Output: Formable<'fail>, Failure: Formable<'fail>> {
    pub emitter: Emitter<'fail, Input, Output, Failure>,
}

impl<'fail, Input: Formable<'fail>, Output: Formable<'fail>, Failure: Formable<'fail>> Order<'fail, Input, Output, Failure> for Fail<'fail, Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Composer<'_, 'fail, Input, Output, Failure>, draft: &mut Draft<'fail, Input, Output, Failure>) {
        // Todo: Actual Registry Handling
        let failure = (self.emitter)(&mut Registry::new(), draft.form.clone());

        draft.set_fail();
        draft.form = Form::Failure(failure);
    }
}

pub struct Ignore;

impl<'ignore, Input: Formable<'ignore>, Output: Formable<'ignore>, Failure: Formable<'ignore>> Order<'ignore, Input, Output, Failure> for Ignore {
    #[inline]
    fn order(&self, _composer: &mut Composer<'_, 'ignore, Input, Output, Failure>, draft: &mut Draft<'ignore, Input, Output, Failure>) {
        if draft.is_aligned() {
            draft.set_ignore();
            draft.form = Form::<Input, Output, Failure>::Blank;
        }
    }
}

pub struct Inspect<'inspector, Input: Formable<'inspector>, Output: Formable<'inspector>, Failure: Formable<'inspector>> {
    pub inspector: Inspector<'inspector, Input, Output, Failure>,
}

impl<'inspector, Input: Formable<'inspector>, Output: Formable<'inspector>, Failure: Formable<'inspector>> Order<'inspector, Input, Output, Failure> for Inspect<'inspector, Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Composer<'_, 'inspector, Input, Output, Failure>, draft: &mut Draft<'inspector, Input, Output, Failure>) {
        let draft_clone = Draft {
            marker: draft.marker,
            position: draft.position,
            consumed: draft.consumed.clone(),
            record: draft.record,
            classifier: draft.classifier.clone(),
            form: draft.form.clone(),
        };

        let order = (self.inspector)(draft_clone);
        order.order(composer, draft);
    }
}

pub struct Multiple<'multiple, Input: Formable<'multiple>, Output: Formable<'multiple>, Failure: Formable<'multiple>> {
    pub orders: Vec<Arc<dyn Order<'multiple, Input, Output, Failure> + 'multiple>>
}

impl<'multiple, Input: Formable<'multiple>, Output: Formable<'multiple>, Failure: Formable<'multiple>> Order<'multiple, Input, Output, Failure> for Multiple<'multiple, Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Composer<'_, 'multiple, Input, Output, Failure>, draft: &mut Draft<'multiple, Input, Output, Failure>) {
        for order in self.orders.iter() {
            order.order(composer, draft);
        }
    }
}

pub struct Panic<'panic, Input: Formable<'panic>, Output: Formable<'panic>, Failure: Formable<'panic>> {
    pub emitter: Emitter<'panic, Input, Output, Failure>,
}

impl<'panic, Input: Formable<'panic>, Output: Formable<'panic>, Failure: Formable<'panic>> Order<'panic, Input, Output, Failure> for Panic<'panic, Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Composer<'_, 'panic, Input, Output, Failure>, draft: &mut Draft<'panic, Input, Output, Failure>) {
        let failure = (self.emitter)(composer.source.registry_mut(), draft.form.clone());

        let form = Form::Failure(failure);
        draft.set_panic();
        draft.form = form;
    }
}

pub struct Pardon;

impl<'pardon, Input: Formable<'pardon>, Output: Formable<'pardon>, Failure: Formable<'pardon>> Order<'pardon, Input, Output, Failure> for Pardon {
    #[inline]
    fn order(&self, _composer: &mut Composer<'_, 'pardon, Input, Output, Failure>, draft: &mut Draft<'pardon, Input, Output, Failure>) {
        draft.set_empty();
    }
}

pub struct Perform<'perform> {
    pub performer: Performer<'perform>,
}

impl<'perform, Input: Formable<'perform>, Output: Formable<'perform>, Failure: Formable<'perform>> Order<'perform, Input, Output, Failure> for Perform<'perform> {
    #[inline]
    fn order(&self, _composer: &mut Composer<'_, 'perform, Input, Output, Failure>, draft: &mut Draft<'perform, Input, Output, Failure>) {
        if draft.is_aligned() {
            if let Ok(mut guard) = self.performer.lock() {
                guard();
                drop(guard);
            }
        }
    }
}

pub struct Skip;

impl<'skip, Input: Formable<'skip>, Output: Formable<'skip>, Failure: Formable<'skip>> Order<'skip, Input, Output, Failure> for Skip {
    #[inline]
    fn order(&self, _composer: &mut Composer<'_, 'skip, Input, Output, Failure>, draft: &mut Draft<'skip, Input, Output, Failure>) {
        if draft.is_aligned() {
            draft.set_empty();
            draft.form = Form::<Input, Output, Failure>::Blank;
        }
    }
}

pub struct Transform<'transform, Input: Formable<'transform>, Output: Formable<'transform>, Failure: Formable<'transform>> {
    pub transformer: Transformer<'transform,  Input, Output, Failure>,
}

impl<'transform, Input: Formable<'transform>, Output: Formable<'transform>, Failure: Formable<'transform>> Order<'transform, Input, Output, Failure> for Transform<'transform, Input, Output, Failure> {
    #[inline]
    fn order(&self, composer: &mut Composer<'_, 'transform, Input, Output, Failure>, draft: &mut Draft<'transform, Input, Output, Failure>) {
        if draft.is_aligned() {
            let result = if let Ok(mut guard) = self.transformer.lock() {
                // Todo: Actual Registry Handling
                let result = guard(&mut Registry::new(), draft.form.clone());
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

impl<'classifier, Input: Formable<'classifier>, Output: Formable<'classifier>, Failure: Formable<'classifier>> Classifier<'classifier, Input, Output, Failure> {
    #[inline]
    pub fn transform<T>(transformer: T) -> Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>
    where
        T: FnMut(&mut Registry, Form<'classifier, Input, Output, Failure>) -> Result<Form<'classifier, Input, Output, Failure>, Failure> + 'classifier,
    {
        Arc::new(Transform { transformer: Arc::new(Mutex::new(transformer))})
    }

    #[inline]
    pub fn fail<T>(emitter: T) -> Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>
    where
        T: Fn(&mut Registry, Form<'classifier, Input, Output, Failure>) -> Failure + 'classifier,
    {
        Arc::new(Fail { emitter: Arc::new(emitter) })
    }

    #[inline]
    pub fn panic<T>(emitter: T) -> Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>
    where
        T: Fn(&mut Registry, Form<'classifier, Input, Output, Failure>) -> Failure + 'classifier,
    {
        Arc::new(Panic { emitter: Arc::new(emitter) })
    }

    #[inline]
    pub fn ignore() -> Arc<dyn Order<'classifier, Input, Output, Failure>> {
        Arc::new(Ignore)
    }

    #[inline]
    pub fn inspect<T>(inspector: T) -> Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>
    where
        T: Fn(Draft<'classifier, Input, Output, Failure>) -> Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier> + 'classifier
    {
        Arc::new(Inspect { inspector: Arc::new(inspector) })
    }

    #[inline]
    pub fn multiple(orders: Vec<Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>>) -> Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier> {
        Arc::new(Multiple { orders })
    }

    #[inline]
    pub fn pardon() -> Arc<dyn Order<'classifier, Input, Output, Failure>> {
        Arc::new(Pardon)
    }

    #[inline]
    pub fn perform<T>(executor: T) -> Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>
    where
        T: FnMut() + 'classifier,
    {
        Arc::new(Perform { performer: Arc::new(Mutex::new(executor))})
    }

    #[inline]
    pub fn skip() -> Arc<dyn Order<'classifier, Input, Output, Failure>> {
        Arc::new(Skip)
    }

    #[inline]
    pub fn branch(found: Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>, missing: Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier>) -> Arc<dyn Order<'classifier, Input, Output, Failure> + 'classifier> {
        Arc::new(Branch { found, missing })
    }
}
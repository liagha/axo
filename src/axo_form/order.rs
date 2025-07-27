use {
    super::{
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
        format::Debug,
        hash::Hash,
    }
};

pub trait Order<Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    fn order(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>);
}

pub struct Align;

impl<Input, Output, Failure> Order<Input, Output, Failure> for Align
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    #[inline]
    fn order(&self, _composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        draft.align();
    }
}

pub struct Branch<Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    pub found: Arc<dyn Order<Input, Output, Failure>>,
    pub missing: Arc<dyn Order<Input, Output, Failure>>,
}

impl<Input, Output, Failure> Order<Input, Output, Failure> for Branch<Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
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

pub struct Fail<Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    pub emitter: Emitter<Input, Output, Failure>,
}

impl<Input, Output, Failure> Order<Input, Output, Failure> for Fail<Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    #[inline]
    fn order(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        let failure = (self.emitter)(composer.source.registry_mut(), draft.form.clone());

        draft.fail();
        draft.form = Form::Failure(failure);
    }
}

pub struct Ignore;

impl<Input, Output, Failure> Order<Input, Output, Failure> for Ignore
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    #[inline]
    fn order(&self, _composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        if draft.is_aligned() {
            draft.ignore();
            draft.form = Form::<Input, Output, Failure>::Blank;
        }
    }
}

pub struct Inspect<Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    pub inspector: Inspector<Input, Output, Failure>,
}

impl<Input, Output, Failure> Order<Input, Output, Failure> for Inspect<Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    #[inline]
    fn order(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        let order = (self.inspector)(draft.to_owned());

        order.order(composer, draft);
    }
}

pub struct Multiple<Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    pub orders: Vec<Arc<dyn Order<Input, Output, Failure>>>
}

impl<Input, Output, Failure> Order<Input, Output, Failure> for Multiple<Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    #[inline]
    fn order(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        for order in self.orders.iter() {
            order.order(composer, draft);
        }
    }
}

pub struct Panic<Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    pub emitter: Emitter<Input, Output, Failure>,
}

impl<Input, Output, Failure> Order<Input, Output, Failure> for Panic<Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    #[inline]
    fn order(&self, composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        let failure = (self.emitter)(composer.source.registry_mut(), draft.form.clone());

        let form = Form::Failure(failure);
        draft.panic();
        draft.form = form;
    }
}

pub struct Pardon;

impl<Input, Output, Failure> Order<Input, Output, Failure> for Pardon
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    #[inline]
    fn order(&self, _composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        draft.empty();
    }
}

pub struct Perform {
    pub performer: Executor,
}

impl<Input, Output, Failure> Order<Input, Output, Failure> for Perform
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
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

impl<Input, Output, Failure> Order<Input, Output, Failure> for Skip
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    #[inline]
    fn order(&self, _composer: &mut Composer<Input, Output, Failure>, draft: &mut Draft<Input, Output, Failure>) {
        if draft.is_aligned() {
            draft.empty();
            draft.form = Form::<Input, Output, Failure>::Blank;
        }
    }
}

pub struct Transform<Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    pub transformer: Transformer<Input, Output, Failure>,
}

impl<Input, Output, Failure> Order<Input, Output, Failure> for Transform<Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
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

impl<Input, Output, Failure> Classifier<Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
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
use {
    super::{
        classifier::Classifier,
        form::Form,
        former::Former,
        helper::{Formable, Inspector, Performer},
    },
    crate::data::sync::Rc,
};

pub trait Order<
    'order,
    Input: Formable<'order>,
    Output: Formable<'order>,
    Failure: Formable<'order>,
>
{
    fn order(
        &self,
        former: &mut Former<'_, 'order, Input, Output, Failure>,
        classifier: &mut Classifier<'order, Input, Output, Failure>,
    );
}

pub struct Align;

impl<'align, Input: Formable<'align>, Output: Formable<'align>, Failure: Formable<'align>>
Order<'align, Input, Output, Failure> for Align
{
    #[inline]
    fn order(
        &self,
        _former: &mut Former<'_, 'align, Input, Output, Failure>,
        classifier: &mut Classifier<'align, Input, Output, Failure>,
    ) {
        classifier.set_align();
    }
}

pub struct Branch<
    'branch,
    Input: Formable<'branch>,
    Output: Formable<'branch>,
    Failure: Formable<'branch>,
> {
    pub found: Rc<dyn Order<'branch, Input, Output, Failure> + 'branch>,
    pub missing: Rc<dyn Order<'branch, Input, Output, Failure> + 'branch>,
}

impl<'branch, Input: Formable<'branch>, Output: Formable<'branch>, Failure: Formable<'branch>>
Order<'branch, Input, Output, Failure> for Branch<'branch, Input, Output, Failure>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'branch, Input, Output, Failure>,
        classifier: &mut Classifier<'branch, Input, Output, Failure>,
    ) {
        let chosen = if classifier.is_aligned() {
            &self.found
        } else {
            &self.missing
        };

        chosen.order(former, classifier);
    }
}

pub struct Fail<'fail, Input: Formable<'fail>, Output: Formable<'fail>, Failure: Formable<'fail>> {
    pub emitter: Rc<dyn Fn(&mut Former<'_, 'fail, Input, Output, Failure>, Classifier<'fail, Input, Output, Failure>) -> Failure + 'fail>,
}

impl<'fail, Input: Formable<'fail>, Output: Formable<'fail>, Failure: Formable<'fail>>
Order<'fail, Input, Output, Failure> for Fail<'fail, Input, Output, Failure>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'fail, Input, Output, Failure>,
        classifier: &mut Classifier<'fail, Input, Output, Failure>,
    ) {
        if !classifier.is_aligned() {
            let failure = (self.emitter)(former, classifier.clone());

            let form_id = former.forms.len();
            former.forms.push(Form::Failure(failure));

            classifier.set_fail();
            classifier.form = form_id;
        }
    }
}

pub struct Ignore;

impl<'ignore, Input: Formable<'ignore>, Output: Formable<'ignore>, Failure: Formable<'ignore>>
Order<'ignore, Input, Output, Failure> for Ignore
{
    #[inline]
    fn order(
        &self,
        _former: &mut Former<'_, 'ignore, Input, Output, Failure>,
        classifier: &mut Classifier<'ignore, Input, Output, Failure>,
    ) {
        if classifier.is_aligned() {
            classifier.set_ignore();
            classifier.form = 0;
        }
    }
}

pub struct Inspect<
    'inspector,
    Input: Formable<'inspector>,
    Output: Formable<'inspector>,
    Failure: Formable<'inspector>,
> {
    pub inspector: Inspector<'inspector, Input, Output, Failure>,
}

impl<
    'inspector,
    Input: Formable<'inspector>,
    Output: Formable<'inspector>,
    Failure: Formable<'inspector>,
> Order<'inspector, Input, Output, Failure> for Inspect<'inspector, Input, Output, Failure>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'inspector, Input, Output, Failure>,
        classifier: &mut Classifier<'inspector, Input, Output, Failure>,
    ) {
        let order = (self.inspector)(classifier.clone());
        order.order(former, classifier);
    }
}

pub struct Multiple<
    'multiple,
    Input: Formable<'multiple>,
    Output: Formable<'multiple>,
    Failure: Formable<'multiple>,
> {
    pub orders: Vec<Rc<dyn Order<'multiple, Input, Output, Failure> + 'multiple>>,
}

impl<
    'multiple,
    Input: Formable<'multiple>,
    Output: Formable<'multiple>,
    Failure: Formable<'multiple>,
> Order<'multiple, Input, Output, Failure> for Multiple<'multiple, Input, Output, Failure>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'multiple, Input, Output, Failure>,
        classifier: &mut Classifier<'multiple, Input, Output, Failure>,
    ) {
        for order in self.orders.iter() {
            order.order(former, classifier);
        }
    }
}

pub struct Panic<
    'panic,
    Input: Formable<'panic>,
    Output: Formable<'panic>,
    Failure: Formable<'panic>,
> {
    pub emitter: Rc<dyn Fn(&mut Former<'_, 'panic, Input, Output, Failure>, Classifier<'panic, Input, Output, Failure>) -> Failure + 'panic>,
}

impl<'panic, Input: Formable<'panic>, Output: Formable<'panic>, Failure: Formable<'panic>>
Order<'panic, Input, Output, Failure> for Panic<'panic, Input, Output, Failure>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'panic, Input, Output, Failure>,
        classifier: &mut Classifier<'panic, Input, Output, Failure>,
    ) {
        if !classifier.is_aligned() {
            let failure = (self.emitter)(former, classifier.clone());

            let form_id = former.forms.len();
            former.forms.push(Form::Failure(failure));

            classifier.set_panic();
            classifier.form = form_id;
        }
    }
}

pub struct Pardon;

impl<'pardon, Input: Formable<'pardon>, Output: Formable<'pardon>, Failure: Formable<'pardon>>
Order<'pardon, Input, Output, Failure> for Pardon
{
    #[inline]
    fn order(
        &self,
        _former: &mut Former<'_, 'pardon, Input, Output, Failure>,
        classifier: &mut Classifier<'pardon, Input, Output, Failure>,
    ) {
        classifier.set_empty();
    }
}

pub struct Perform<'perform> {
    pub performer: Performer<'perform>,
}

impl<
    'perform,
    Input: Formable<'perform>,
    Output: Formable<'perform>,
    Failure: Formable<'perform>,
> Order<'perform, Input, Output, Failure> for Perform<'perform>
{
    #[inline]
    fn order(
        &self,
        _former: &mut Former<'_, 'perform, Input, Output, Failure>,
        classifier: &mut Classifier<'perform, Input, Output, Failure>,
    ) {
        if classifier.is_aligned() {
            self.performer.clone()();
        }
    }
}

pub struct Skip;

impl<'skip, Input: Formable<'skip>, Output: Formable<'skip>, Failure: Formable<'skip>>
Order<'skip, Input, Output, Failure> for Skip
{
    #[inline]
    fn order(
        &self,
        _former: &mut Former<'_, 'skip, Input, Output, Failure>,
        classifier: &mut Classifier<'skip, Input, Output, Failure>,
    ) {
        if classifier.is_aligned() {
            classifier.set_empty();
            classifier.form = 0;
        }
    }
}

pub struct Transform<
    'transform,
    Input: Formable<'transform>,
    Output: Formable<'transform>,
    Failure: Formable<'transform>,
> {
    pub transformer: Rc<dyn Fn(&mut Former<'_, 'transform, Input, Output, Failure>, &mut Classifier<'transform, Input, Output, Failure>) -> Result<(), Failure> + 'transform>,
}

impl<
    'transform,
    Input: Formable<'transform>,
    Output: Formable<'transform>,
    Failure: Formable<'transform>,
> Order<'transform, Input, Output, Failure> for Transform<'transform, Input, Output, Failure>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'transform, Input, Output, Failure>,
        classifier: &mut Classifier<'transform, Input, Output, Failure>,
    ) {
        if classifier.is_aligned() {
            let result = (self.transformer)(former, classifier);

            match result {
                Ok(_) => {
                }
                Err(error) => {
                    let form_id = former.forms.len();
                    former.forms.push(Form::Failure(error));

                    classifier.set_fail();
                    classifier.form = form_id;
                }
            }
        }
    }
}

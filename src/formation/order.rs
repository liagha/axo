// src/formation/order.rs
use super::{
    classifier::Classifier,
    form::Form,
    former::Former,
    helper::Formable,
};

pub trait Order<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    fn order(
        &self,
        former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
    );
}

pub struct Align;

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Order<'a, Input, Output, Failure> for Align
{
    #[inline]
    fn order(
        &self,
        _former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
    ) {
        classifier.set_align();
    }
}

pub struct Branch<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub found: &'a dyn Order<'a, Input, Output, Failure>,
    pub missing: &'a dyn Order<'a, Input, Output, Failure>,
}

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Order<'a, Input, Output, Failure> for Branch<'a, Input, Output, Failure>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
    ) {
        let chosen = if classifier.is_aligned() {
            self.found
        } else {
            self.missing
        };

        chosen.order(former, classifier);
    }
}

pub struct Fail<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub emitter: &'a dyn Fn(
        &mut Former<'_, 'a, Input, Output, Failure>,
        Classifier<'a, Input, Output, Failure>,
    ) -> Failure,
}

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Order<'a, Input, Output, Failure> for Fail<'a, Input, Output, Failure>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
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

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Order<'a, Input, Output, Failure> for Ignore
{
    #[inline]
    fn order(
        &self,
        _former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
    ) {
        if classifier.is_aligned() {
            classifier.set_ignore();
            classifier.form = 0;
        }
    }
}

pub struct Inspect<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub inspector: &'a dyn Fn(
        Classifier<'a, Input, Output, Failure>,
    ) -> &'a dyn Order<'a, Input, Output, Failure>,
}

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Order<'a, Input, Output, Failure> for Inspect<'a, Input, Output, Failure>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
    ) {
        let target = (self.inspector)(classifier.clone());
        target.order(former, classifier);
    }
}

pub struct Multiple<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub orders: Vec<&'a dyn Order<'a, Input, Output, Failure>>,
}

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Order<'a, Input, Output, Failure> for Multiple<'a, Input, Output, Failure>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
    ) {
        for order in self.orders.iter() {
            order.order(former, classifier);
        }
    }
}

pub struct Panic<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub emitter: &'a dyn Fn(
        &mut Former<'_, 'a, Input, Output, Failure>,
        Classifier<'a, Input, Output, Failure>,
    ) -> Failure,
}

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Order<'a, Input, Output, Failure> for Panic<'a, Input, Output, Failure>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
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

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Order<'a, Input, Output, Failure> for Pardon
{
    #[inline]
    fn order(
        &self,
        _former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
    ) {
        classifier.set_empty();
    }
}

pub struct Perform<'a> {
    pub performer: &'a dyn Fn(),
}

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Order<'a, Input, Output, Failure> for Perform<'a>
{
    #[inline]
    fn order(
        &self,
        _former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
    ) {
        if classifier.is_aligned() {
            (self.performer)();
        }
    }
}

pub struct Skip;

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Order<'a, Input, Output, Failure> for Skip
{
    #[inline]
    fn order(
        &self,
        _former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
    ) {
        if classifier.is_aligned() {
            classifier.set_empty();
            classifier.form = 0;
        }
    }
}

pub struct Transform<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub transformer: &'a dyn Fn(
        &mut Former<'_, 'a, Input, Output, Failure>,
        &mut Classifier<'a, Input, Output, Failure>,
    ) -> Result<(), Failure>,
}

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Order<'a, Input, Output, Failure> for Transform<'a, Input, Output, Failure>
{
    #[inline]
    fn order(
        &self,
        former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
    ) {
        if classifier.is_aligned() {
            if let Err(error) = (self.transformer)(former, classifier) {
                let form_id = former.forms.len();
                former.forms.push(Form::Failure(error));

                classifier.set_fail();
                classifier.form = form_id;
            }
        }
    }
}

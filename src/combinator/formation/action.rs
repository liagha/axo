use {
    crate::{
        combinator::{
            Action, Multiple, Ignore, Skip, Transform, Fail, Panic,
            Classifier, Form, Former, Formable,
        },
    },
};

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Action<'a, Input, Output, Failure> for Multiple<'a, Input, Output, Failure>
{
    #[inline]
    fn action(
        &self,
        former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
    ) {
        for action in self.actions.iter() {
            action.action(former, classifier);
        }
    }
}

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Action<'a, Input, Output, Failure> for Ignore
{
    #[inline]
    fn action(
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

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Action<'a, Input, Output, Failure> for Skip
{
    #[inline]
    fn action(
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

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Action<'a, Input, Output, Failure> for Transform<'a, Input, Output, Failure>
{
    #[inline]
    fn action(
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
impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Action<'a, Input, Output, Failure> for Fail<'a, Input, Output, Failure>
{
    #[inline]
    fn action(
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

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Action<'a, Input, Output, Failure> for Panic<'a, Input, Output, Failure>
{
    #[inline]
    fn action(
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
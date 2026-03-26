use {
    crate::{
        data::memory::Rc,
        combinator::{
            Classifier,
            Form,
            Former,
            helper::Formable,
        },
    },
};


pub trait Action<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    fn action(
        &self,
        former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
    );
}

pub struct Align;

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Action<'a, Input, Output, Failure> for Align
{
    #[inline]
    fn action(
        &self,
        _former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
    ) {
        classifier.set_align();
    }
}

pub struct Branch<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub found: Rc<dyn Action<'a, Input, Output, Failure> + 'a>,
    pub missing: Rc<dyn Action<'a, Input, Output, Failure> + 'a>,
}

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Action<'a, Input, Output, Failure> for Branch<'a, Input, Output, Failure>
{
    #[inline]
    fn action(
        &self,
        former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
    ) {
        let chosen = if classifier.is_aligned() {
            &self.found
        } else {
            &self.missing
        };

        chosen.action(former, classifier);
    }
}

pub struct Fail<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub emitter: Rc<dyn Fn(
        &mut Former<'_, 'a, Input, Output, Failure>,
        Classifier<'a, Input, Output, Failure>,
    ) -> Failure + 'a>,
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

pub struct Ignore;

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

pub struct Inspect<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub inspector: Rc<dyn Fn(
        Classifier<'a, Input, Output, Failure>,
    ) -> &'a (dyn Action<'a, Input, Output, Failure> + 'a) + 'a>,
}

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Action<'a, Input, Output, Failure> for Inspect<'a, Input, Output, Failure>
{
    #[inline]
    fn action(
        &self,
        former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
    ) {
        let target = (self.inspector)(classifier.clone());
        target.action(former, classifier);
    }
}

pub struct Multiple<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub actions: Vec<Rc<dyn Action<'a, Input, Output, Failure> + 'a>>,
}

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

pub struct Pair<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub first: Rc<dyn Action<'a, Input, Output, Failure> + 'a>,
    pub second: Rc<dyn Action<'a, Input, Output, Failure> + 'a>,
}

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Action<'a, Input, Output, Failure> for Pair<'a, Input, Output, Failure>
{
    #[inline]
    fn action(
        &self,
        former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
    ) {
        self.first.action(former, classifier);
        self.second.action(former, classifier);
    }
}

pub struct Panic<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub emitter: Rc<dyn Fn(
        &mut Former<'_, 'a, Input, Output, Failure>,
        Classifier<'a, Input, Output, Failure>,
    ) -> Failure + 'a>,
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

pub struct Pardon;

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Action<'a, Input, Output, Failure> for Pardon
{
    #[inline]
    fn action(
        &self,
        _former: &mut Former<'_, 'a, Input, Output, Failure>,
        classifier: &mut Classifier<'a, Input, Output, Failure>,
    ) {
        classifier.set_empty();
    }
}

pub struct Perform<'a> {
    pub performer: Rc<dyn Fn() + 'a>,
}

impl<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>>
Action<'a, Input, Output, Failure> for Perform<'a>
{
    #[inline]
    fn action(
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

pub struct Transform<'a, Input: Formable<'a>, Output: Formable<'a>, Failure: Formable<'a>> {
    pub transformer: Rc<dyn Fn(
        &mut Former<'_, 'a, Input, Output, Failure>,
        &mut Classifier<'a, Input, Output, Failure>,
    ) -> Result<(), Failure> + 'a>,
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

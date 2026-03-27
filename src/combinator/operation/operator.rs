use {
    crate::combinator::{Formable, Operation},
    std::marker::PhantomData,
};

pub struct Operator<'a, Input, Output, Failure>
where
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    pub inputs: Input,
    pub outputs: Output,
    pub failures: Failure,
    pub _marker: PhantomData<&'a ()>,
}

impl<'a, Input, Output, Failure> Operator<'a, Input, Output, Failure>
where
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    pub const fn new(inputs: Input, outputs: Output, failures: Failure) -> Self {
        Self {
            inputs,
            outputs,
            failures,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn build<'source>(
        &mut self,
        operation: &mut Operation<'a, 'source, Input, Output, Failure>,
    ) {
        let action = operation.action.clone();
        action.action(self, operation);
    }
}

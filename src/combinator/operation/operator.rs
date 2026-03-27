use {
    crate::combinator::{Formable, Operation, Status},
    std::{marker::PhantomData, thread, time::Duration},
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
    pub phantom: PhantomData<&'a ()>,
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
            phantom: PhantomData,
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

    #[inline]
    pub fn execute<'source>(
        &mut self,
        operation: &mut Operation<'a, 'source, Input, Output, Failure>,
    ) -> Status {
        loop {
            self.build(operation);

            match operation.status {
                Status::Pending => {
                    thread::sleep(Duration::from_millis(10));
                }
                Status::Resolved | Status::Rejected => break operation.status,
            }
        }
    }
}

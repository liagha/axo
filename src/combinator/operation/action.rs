use {
    crate::{
        combinator::{
            Action, Command, Multiple, Sequence, Alternative, Operator, Processor, Formable, Status
        },
        data::{memory::take, Scale},
    }
};

impl<'a, 'source, Data, Output, Failure>
Action<'a, Operator<'a, Data, Output, Failure>, Processor<'a, 'source, Data, Output, Failure>>
for Command<'a, 'source, Operator<'a, Data, Output, Failure>, Processor<'a, 'source, Data, Output, Failure>>
where
    Data: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn action(
        &self,
        operator: &mut Operator<'a, Data, Output, Failure>,
        processor: &mut Processor<'a, 'source, Data, Output, Failure>,
    ) {
        processor.status = Status::Active;
        (self.runner)(operator, processor);
    }
}

impl<'a, 'source, Data, Output, Failure>
Action<'a, Operator<'a, Data, Output, Failure>, Processor<'a, 'source, Data, Output, Failure>>
for Multiple<'a, 'source, Operator<'a, Data, Output, Failure>, Processor<'a, 'source, Data, Output, Failure>>
where
    Data: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn action(
        &self,
        operator: &mut Operator<'a, Data, Output, Failure>,
        processor: &mut Processor<'a, 'source, Data, Output, Failure>,
    ) {
        for step in self.actions.iter() {
            step.action(operator, processor);
        }
    }
}

impl<'a, 'source, Data, Output, Failure, const SIZE: Scale>
Action<'a, Operator<'a, Data, Output, Failure>, Processor<'a, 'source, Data, Output, Failure>>
for Sequence<Processor<'a, 'source, Data, Output, Failure>, SIZE>
where
    Data: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn action(
        &self,
        operator: &mut Operator<'a, Data, Output, Failure>,
        processor: &mut Processor<'a, 'source, Data, Output, Failure>,
    ) {
        let mut stack = take(&mut processor.stack);
        processor.status = Status::Active;

        for node in &self.states {
            let mut child = Processor::create(
                node.action.clone(),
                Status::Pending,
                processor.depth + 1,
                stack,
            );

            operator.build(&mut child);
            stack = child.stack;

            if child.status == Status::Rejected {
                processor.reject();
                processor.stack = stack;
                return;
            }
        }

        processor.resolve();
        processor.stack = stack;
    }
}

impl<'a, 'source, Data, Output, Failure, const SIZE: Scale>
Action<'a, Operator<'a, Data, Output, Failure>, Processor<'a, 'source, Data, Output, Failure>>
for Alternative<Processor<'a, 'source, Data, Output, Failure>, SIZE>
where
    Data: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn action(
        &self,
        operator: &mut Operator<'a, Data, Output, Failure>,
        processor: &mut Processor<'a, 'source, Data, Output, Failure>,
    ) {
        let stack = take(&mut processor.stack);
        processor.status = Status::Active;

        for node in &self.states {
            let mut child = Processor::create(
                node.action.clone(),
                Status::Pending,
                processor.depth + 1,
                stack.clone(),
            );

            operator.build(&mut child);

            if child.status == Status::Resolved {
                processor.resolve();
                processor.stack = child.stack;
                return;
            }
        }

        processor.reject();
        processor.stack = stack;
    }
}

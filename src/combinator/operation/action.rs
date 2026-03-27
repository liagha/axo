use {
    crate::{
        combinator::{
            Action, Alternative, Command, Condition, Formable, Multiple, Operation, Operator,
            Repetition, Sequence, Status, Trigger,
        },
        data::{memory::take, Identity, Scale},
    },
    std::{process::Command as Terminal, time::SystemTime},
};

impl<'a, 'source, Input, Output, Failure>
Action<'a, Operator<'a, Input, Output, Failure>, Operation<'a, 'source, Input, Output, Failure>>
for Command
where
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn action(
        &self,
        _operator: &mut Operator<'a, Input, Output, Failure>,
        operation: &mut Operation<'a, 'source, Input, Output, Failure>,
    ) {
        let mut terminal = Terminal::new(&self.program);
        terminal.args(&self.arguments);
        if let Some(dir) = self.dir.as_deref() {
            terminal.current_dir(dir);
        }

        match terminal.output() {
            Ok(outcome) if outcome.status.success() => operation.set_resolve(),
            _ => operation.set_reject(),
        }
    }
}

impl<'a, 'source, Input, Output, Failure>
Action<'a, Operator<'a, Input, Output, Failure>, Operation<'a, 'source, Input, Output, Failure>>
for Trigger<'a, 'source, Input, Output, Failure>
where
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn action(
        &self,
        operator: &mut Operator<'a, Input, Output, Failure>,
        operation: &mut Operation<'a, 'source, Input, Output, Failure>,
    ) {
        match self.condition {
            Condition::Always => {}
            Condition::Time(time) => {
                if SystemTime::now() < time {
                    operation.set_pending();
                    return;
                }
            }
            Condition::Evaluate(function) => {
                if !function() {
                    operation.set_pending();
                    return;
                }
            }
        }

        self.action.action(operator, operation);
    }
}

impl<'a, 'source, Input, Output, Failure>
Action<'a, Operator<'a, Input, Output, Failure>, Operation<'a, 'source, Input, Output, Failure>>
for Multiple<
    'a,
    'source,
    Operator<'a, Input, Output, Failure>,
    Operation<'a, 'source, Input, Output, Failure>,
>
where
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn action(
        &self,
        operator: &mut Operator<'a, Input, Output, Failure>,
        operation: &mut Operation<'a, 'source, Input, Output, Failure>,
    ) {
        for step in self.actions.iter() {
            step.action(operator, operation);
        }
    }
}

impl<'a, 'source, Input, Output, Failure, const SIZE: Scale>
Action<'a, Operator<'a, Input, Output, Failure>, Operation<'a, 'source, Input, Output, Failure>>
for Sequence<Operation<'a, 'source, Input, Output, Failure>, SIZE>
where
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn action(
        &self,
        operator: &mut Operator<'a, Input, Output, Failure>,
        operation: &mut Operation<'a, 'source, Input, Output, Failure>,
    ) {
        let mut current_stack = take(&mut operation.stack);
        let base_stack = current_stack.len();
        let mut broke = false;

        for pattern in &self.states {
            let mut child = Operation::create(
                pattern.action.clone(),
                Status::Pending,
                operation.depth + 1,
                current_stack,
            );

            operator.build(&mut child);

            let halted = (self.halt)(&child);

            current_stack = take(&mut child.stack);

            if halted {
                operation.status = child.status;
                broke = true;
                break;
            }

            operation.status = child.status;
        }

        operation.stack = current_stack;

        if broke {
            operation.stack.truncate(base_stack);
        }
    }
}

impl<'a, 'source, Input, Output, Failure, const SIZE: Scale>
Action<'a, Operator<'a, Input, Output, Failure>, Operation<'a, 'source, Input, Output, Failure>>
for Alternative<Operation<'a, 'source, Input, Output, Failure>, SIZE>
where
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn action(
        &self,
        operator: &mut Operator<'a, Input, Output, Failure>,
        operation: &mut Operation<'a, 'source, Input, Output, Failure>,
    ) {
        let mut best: Option<Operation<'a, 'source, Input, Output, Failure>> = None;
        let current_stack = take(&mut operation.stack);

        for pattern in &self.states {
            let mut child = Operation::create(
                pattern.action.clone(),
                Status::Pending,
                operation.depth + 1,
                current_stack.clone(),
            );

            operator.build(&mut child);

            if child.is_pending() {
                best = Some(child);
                break;
            }

            let better = match &best {
                Some(champion) => (self.compare)(&child, champion),
                None => true,
            };

            if better {
                best = Some(child);
            }

            if let Some(ref champion) = best {
                if (self.halt)(champion) {
                    break;
                }
            }
        }

        match best {
            Some(mut champion) => {
                operation.status = champion.status;
                operation.stack = take(&mut champion.stack);
            }
            None => {
                operation.set_reject();
                operation.stack = current_stack;
            }
        }
    }
}

impl<'a, 'source, Input, Output, Failure>
Action<'a, Operator<'a, Input, Output, Failure>, Operation<'a, 'source, Input, Output, Failure>>
for Repetition<Operation<'a, 'source, Input, Output, Failure>>
where
    Input: Formable<'a>,
    Output: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    fn action(
        &self,
        operator: &mut Operator<'a, Input, Output, Failure>,
        operation: &mut Operation<'a, 'source, Input, Output, Failure>,
    ) {
        let mut current_stack = take(&mut operation.stack);
        let base_stack = current_stack.len();
        let mut count: Identity = 0;

        loop {
            let step_stack = current_stack.len();

            let mut child = Operation::create(
                self.state.action.clone(),
                Status::Pending,
                operation.depth + 1,
                current_stack,
            );

            operator.build(&mut child);

            let halted = (self.halt)(&child);
            let kept = (self.keep)(&child);

            current_stack = take(&mut child.stack);

            if halted {
                if child.is_pending() {
                    operation.status = child.status;
                    current_stack.truncate(step_stack);
                    operation.stack = current_stack;
                    return;
                }
                if kept {
                    count += 1;
                } else {
                    current_stack.truncate(step_stack);
                }
                break;
            }

            if kept {
                count += 1;
            } else {
                current_stack.truncate(step_stack);
            }

            if let Some(max) = self.maximum {
                if count >= max as Identity {
                    break;
                }
            }
        }

        operation.stack = current_stack;

        if count >= self.minimum as Identity {
            operation.set_resolve();
        } else {
            operation.stack.truncate(base_stack);
            operation.set_reject();
        }
    }
}
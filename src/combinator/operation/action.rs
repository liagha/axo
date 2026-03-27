use {
    crate::{
        combinator::{
            Action, Alternative, Formable, Multiple, Operator,
            Processor, Sequence, Status, Task, Workflow,
        },
        data::{memory::take, Scale},
    },
    std::{collections::HashMap, time::SystemTime},
};

impl<'a, 'source, Data, Output, Failure>
    Action<'a, Operator<'a, Data, Output, Failure>, Processor<'a, 'source, Data, Output, Failure>>
    for Task<'a, 'source, Data, Output, Failure>
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

        let run = self.ready(SystemTime::now());
        if !run {
            processor.status = Status::Pending;
            return;
        }

        let result = self.execute(operator, processor);
        if result.succeeded() {
            processor.resolve();
        } else {
            processor.reject();
        }
    }
}

impl<'a, 'source, Data, Output, Failure>
    Action<'a, Operator<'a, Data, Output, Failure>, Processor<'a, 'source, Data, Output, Failure>>
    for Workflow<'a, 'source, Data, Output, Failure>
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

        if self.tasks.is_empty() {
            processor.resolve();
            return;
        }

        let mut index_by_id = HashMap::with_capacity(self.tasks.len());

        for (index, task) in self.tasks.iter().enumerate() {
            if !index_by_id.insert(task.id.clone(), index).is_none() {
                processor.reject();
                return;
            }
        }

        for task in &self.tasks {
            for dep in &task.depends {
                if dep == &task.id || !index_by_id.contains_key(dep) {
                    processor.reject();
                    return;
                }
            }
        }

        let mut done = vec![false; self.tasks.len()];
        let mut success = vec![false; self.tasks.len()];
        let now = SystemTime::now();

        loop {
            let mut changed = false;

            for index in 0..self.tasks.len() {
                if done[index] {
                    continue;
                }

                let task = &self.tasks[index];

                if !task.ready(now) {
                    continue;
                }

                if !task.depends.iter().all(|dep| {
                    let dep_index = index_by_id[dep];
                    success[dep_index]
                }) {
                    continue;
                }

                let result = task.execute(operator, processor);
                done[index] = true;
                success[index] = result.succeeded();
                changed = true;

                if !result.succeeded() && self.fail_fast {
                    processor.reject();
                    return;
                }
            }

            if !changed {
                break;
            }
        }

        if done.iter().all(|done| *done) {
            processor.resolve();
            return;
        }

        let has_wait = (0..self.tasks.len()).any(|index| {
            if done[index] {
                false
            } else {
                !self.tasks[index].ready(now)
            }
        });

        if has_wait {
            processor.status = Status::Pending;
            return;
        }

        processor.reject();
    }
}

impl<'a, 'source, Data, Output, Failure>
    Action<'a, Operator<'a, Data, Output, Failure>, Processor<'a, 'source, Data, Output, Failure>>
    for Multiple<
        'a,
        'source,
        Operator<'a, Data, Output, Failure>,
        Processor<'a, 'source, Data, Output, Failure>,
    >
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

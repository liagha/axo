use crate::{
    combinator::{
        Alternative, Combinator, Command, Condition, Cycle, Multiple, Operation, Operator,
        Repetition, Sequence, Status, Transform, Trigger,
    },
    data::{memory::take, Identity, Scale},
    internal::{
        platform::{metadata, Command as Terminal, Stdio, Write},
        time::SystemTime,
    },
};

impl<'source, Store: Clone + Send + Sync + 'source>
    Combinator<'static, Operator<Store>, Operation<'source, Store>> for Command
{
    #[inline]
    fn combinator(
        &self,
        _operator: &mut Operator<Store>,
        operation: &mut Operation<'source, Store>,
    ) {
        let mut terminal = Terminal::new(&self.program);
        terminal.args(&self.arguments);

        if let Some(dir) = self.directory.as_deref() {
            terminal.current_dir(dir);
        }

        if !operation.payload.is_empty() {
            terminal.stdin(Stdio::piped());
        }

        terminal.stdout(Stdio::piped());

        if let Ok(mut child) = terminal.spawn() {
            if !operation.payload.is_empty() {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(&operation.payload);
                }
            }

            if let Ok(output) = child.wait_with_output() {
                if output.status.success() {
                    operation.set_resolve(output.stdout);
                    return;
                }
            }
        }

        operation.set_reject();
    }
}

impl<'source, Store: Clone + Send + Sync + 'source>
    Combinator<'static, Operator<Store>, Operation<'source, Store>> for Trigger<'source, Store>
{
    #[inline]
    fn combinator(
        &self,
        operator: &mut Operator<Store>,
        operation: &mut Operation<'source, Store>,
    ) {
        match &self.condition {
            Condition::Always => {}
            Condition::Time(time) => {
                if SystemTime::now() < *time {
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
            Condition::Outdated(source, target) => {
                let source_meta = metadata(source).and_then(|m| m.modified());
                let target_meta = metadata(target).and_then(|m| m.modified());

                match (source_meta, target_meta) {
                    (Ok(s), Ok(t)) if s > t => {}
                    (Ok(_), Err(_)) => {}
                    (Err(_), _) => {
                        operation.set_reject();
                        return;
                    }
                    _ => {
                        operation.set_resolve(Vec::new());
                        return;
                    }
                }
            }
            Condition::Missing(path) => {
                if metadata(path).is_ok() {
                    operation.set_resolve(Vec::new());
                    return;
                }
            }
        }

        self.combinator.combinator(operator, operation);
    }
}

impl<'source, Store: Clone + Send + Sync + 'source>
    Combinator<'static, Operator<Store>, Operation<'source, Store>>
    for Multiple<'static, 'source, Operator<Store>, Operation<'source, Store>>
{
    #[inline]
    fn combinator(
        &self,
        operator: &mut Operator<Store>,
        operation: &mut Operation<'source, Store>,
    ) {
        for step in self.combinators.iter() {
            step.combinator(operator, operation);
        }
    }
}

impl<'source, Store: Clone + Send + Sync + 'source, const SIZE: Scale>
    Combinator<'static, Operator<Store>, Operation<'source, Store>>
    for Sequence<Operation<'source, Store>, SIZE>
{
    #[inline]
    fn combinator(
        &self,
        operator: &mut Operator<Store>,
        operation: &mut Operation<'source, Store>,
    ) {
        let mut current_stack = take(&mut operation.stack);
        let mut current_payload = take(&mut operation.payload);
        let base_stack = current_stack.len();
        let mut broke = false;

        for state in &self.states {
            let mut child = Operation::create(
                state.identity,
                state.combinator.clone(),
                Status::Pending,
                operation.depth + 1,
                current_stack,
                current_payload,
                state.depends.clone(),
            );

            operator.build(&mut child);

            let halted = (self.halt)(&child);

            current_stack = take(&mut child.stack);

            if let Status::Resolved(data) = &child.status {
                current_payload = data.clone();
            } else {
                current_payload = Vec::new();
            }

            if halted {
                operation.status = child.status.clone();
                broke = true;
                break;
            }

            operation.status = child.status.clone();
        }

        operation.stack = current_stack;
        operation.payload = current_payload;

        if broke {
            operation.stack.truncate(base_stack);
        }
    }
}

impl<'source, Store: Clone + Send + Sync + 'source, const SIZE: Scale>
    Combinator<'static, Operator<Store>, Operation<'source, Store>>
    for Alternative<Operation<'source, Store>, SIZE>
{
    #[inline]
    fn combinator(
        &self,
        operator: &mut Operator<Store>,
        operation: &mut Operation<'source, Store>,
    ) {
        let mut best: Option<Operation<'source, Store>> = None;
        let current_stack = take(&mut operation.stack);
        let current_payload = take(&mut operation.payload);

        for state in &self.states {
            let mut child = Operation::create(
                state.identity,
                state.combinator.clone(),
                Status::Pending,
                operation.depth + 1,
                current_stack.clone(),
                current_payload.clone(),
                state.depends.clone(),
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
                operation.status = champion.status.clone();
                operation.stack = take(&mut champion.stack);
                operation.payload = take(&mut champion.payload);
            }
            None => {
                operation.set_reject();
                operation.stack = current_stack;
                operation.payload = current_payload;
            }
        }
    }
}

impl<'source, Store: Clone + Send + Sync + 'source>
    Combinator<'static, Operator<Store>, Operation<'source, Store>>
    for Repetition<Operation<'source, Store>>
{
    #[inline]
    fn combinator(
        &self,
        operator: &mut Operator<Store>,
        operation: &mut Operation<'source, Store>,
    ) {
        let mut current_stack = take(&mut operation.stack);
        let mut current_payload = take(&mut operation.payload);
        let base_stack = current_stack.len();
        let mut count: Identity = 0;

        loop {
            let step_stack = current_stack.len();

            let mut child = Operation::create(
                crate::combinator::next_identity(),
                self.state.combinator.clone(),
                Status::Pending,
                operation.depth + 1,
                current_stack,
                current_payload.clone(),
                self.state.depends.clone(),
            );

            operator.build(&mut child);

            let halted = (self.halt)(&child);
            let kept = (self.keep)(&child);

            current_stack = take(&mut child.stack);

            if let Status::Resolved(data) = &child.status {
                current_payload = data.clone();
            }

            if halted {
                if child.is_pending() {
                    operation.status = child.status.clone();
                    current_stack.truncate(step_stack);
                    operation.stack = current_stack;
                    operation.payload = current_payload;
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
            operation.set_resolve(current_payload);
        } else {
            operation.stack.truncate(base_stack);
            operation.set_reject();
        }
    }
}

impl<'source, Store: Clone + Send + Sync + 'source>
    Combinator<'static, Operator<Store>, Operation<'source, Store>>
    for Cycle<Operation<'source, Store>>
{
    #[inline]
    fn combinator(
        &self,
        operator: &mut Operator<Store>,
        operation: &mut Operation<'source, Store>,
    ) {
        let mut current_stack = take(&mut operation.stack);
        let mut current_payload = take(&mut operation.payload);

        loop {
            let mut local = Operator::new(operator.store.clone());
            let mut child = Operation::create(
                crate::combinator::next_identity(),
                self.state.combinator.clone(),
                Status::Pending,
                operation.depth + 1,
                current_stack,
                current_payload,
                self.state.depends.clone(),
            );

            local.build(&mut child);

            current_stack = take(&mut child.stack);
            current_payload = take(&mut child.payload);
            operation.status = child.status.clone();

            if !child.is_resolved() || !(self.keep)(&child) {
                break;
            }
        }

        operation.stack = current_stack;
        operation.payload = current_payload;
    }
}

impl<'source, Store: Clone + Send + Sync + 'source, Failure>
    Combinator<'static, Operator<Store>, Operation<'source, Store>>
    for Transform<'static, 'source, Operator<Store>, Operation<'source, Store>, Failure>
{
    #[inline]
    fn combinator(
        &self,
        operator: &mut Operator<Store>,
        operation: &mut Operation<'source, Store>,
    ) {
        let _ = (self.transformer)(operator, operation);
    }
}

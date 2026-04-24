use crate::{
    combinator::{Combinator, Operation, Operator, Status},
    data::memory::take,
    internal::platform::scope,
};

pub struct Plan<'source, Store = ()> {
    pub states: Vec<Operation<'source, Store>>,
}

impl<'source, Store: Clone + Send + Sync + 'source> Combinator<'static, Operator<Store>, Operation<'source, Store>> for Plan<'source, Store> {
    #[inline]
    fn combinator(&self, operator: &mut Operator<Store>, operation: &mut Operation<'source, Store>) {
        let mut all_resolved = true;
        let mut any_rejected = false;
        let mut final_payload = take(&mut operation.payload);

        for state in &self.states {
            let mut child = Operation::create(
                state.identity,
                state.combinator.clone(),
                Status::Pending,
                operation.depth + 1,
                take(&mut operation.stack),
                final_payload.clone(),
                state.depends.clone(),
            );

            operator.build(&mut child);

            operation.stack = take(&mut child.stack);

            match child.status {
                Status::Pending => all_resolved = false,
                Status::Rejected => any_rejected = true,
                Status::Resolved(data) => final_payload = data,
            }
        }

        operation.payload = final_payload;

        if any_rejected {
            operation.set_reject();
        } else if all_resolved {
            let payload = take(&mut operation.payload);
            operation.set_resolve(payload);
        } else {
            operation.set_pending();
        }
    }
}

pub struct Parallel<'source, Store = ()> {
    pub states: Vec<Operation<'source, Store>>,
}

impl<'source, Store: Clone + Send + Sync + 'source> Combinator<'static, Operator<Store>, Operation<'source, Store>> for Parallel<'source, Store> {
    #[inline]
    fn combinator(&self, operator: &mut Operator<Store>, operation: &mut Operation<'source, Store>) {
        let mut all_resolved = true;
        let mut any_rejected = false;
        let mut final_payload = take(&mut operation.payload);
        let stack = take(&mut operation.stack);

        scope(|scope| {
            let mut handles = Vec::with_capacity(self.states.len());

            for state in &self.states {
                let mut child = Operation::create(
                    state.identity,
                    state.combinator.clone(),
                    Status::Pending,
                    operation.depth + 1,
                    stack.clone(),
                    final_payload.clone(),
                    state.depends.clone(),
                );

                let cache = operator.cache.clone();
                let store = operator.store.clone();

                handles.push(scope.spawn(move || {
                    let mut local_operator = Operator { cache, store };
                    local_operator.build(&mut child);
                    child
                }));
            }

            for handle in handles {
                if let Ok(child) = handle.join() {
                    if !child.is_pending() {
                        operator.cache.insert(child.identity, child.status.clone());
                    }

                    match child.status {
                        Status::Pending => all_resolved = false,
                        Status::Rejected => any_rejected = true,
                        Status::Resolved(data) => {
                            final_payload.extend(data);
                        }
                    }
                } else {
                    any_rejected = true;
                }
            }
        });

        operation.stack = stack;
        operation.payload = final_payload;

        if any_rejected {
            operation.set_reject();
        } else if all_resolved {
            let payload = take(&mut operation.payload);
            operation.set_resolve(payload);
        } else {
            operation.set_pending();
        }
    }
}

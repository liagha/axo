use {
    crate::{
        combinator::{Action, Operation, Operator, Status},
        data::memory::take,
    },
};

pub struct Plan<'source> {
    pub states: Vec<Operation<'source>>,
}

impl<'source> Action<'static, Operator, Operation<'source>> for Plan<'source> {
    #[inline]
    fn action(&self, operator: &mut Operator, operation: &mut Operation<'source>) {
        let mut all_resolved = true;
        let mut any_rejected = false;
        let mut final_payload = take(&mut operation.payload);

        for state in &self.states {
            let mut child = Operation::create(
                state.identity,
                state.action.clone(),
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

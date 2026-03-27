use {
    crate::combinator::{Operation, Status},
    std::{thread, time::Duration},
};

pub struct Operator;

impl Operator {
    #[inline]
    pub const fn new() -> Self {
        Self {}
    }

    #[inline]
    pub fn build<'source>(&mut self, operation: &mut Operation<'source>) {
        let action = operation.action.clone();
        action.action(self, operation);
    }

    #[inline]
    pub fn execute<'source>(&mut self, operation: &mut Operation<'source>) -> Status {
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

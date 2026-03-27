use {
    crate::{
        combinator::{Operation, Status},
        data::Identity,
        internal::{hash::Map, platform::sleep, time::Duration},
    },
};

pub struct Operator {
    pub cache: Map<Identity, Status>,
}

impl Operator {
    #[inline]
    pub fn new() -> Self {
        Self {
            cache: Map::new(),
        }
    }

    #[inline]
    pub fn build<'source>(&mut self, operation: &mut Operation<'source>) {
        if let Some(status) = self.cache.get(&operation.identity) {
            operation.status = status.clone();
            return;
        }

        for dependency in &operation.depends {
            if let Some(status) = self.cache.get(dependency) {
                if !matches!(status, Status::Resolved(_)) {
                    operation.set_reject();
                    return;
                }
            } else {
                operation.set_pending();
                return;
            }
        }

        let action = operation.action.clone();
        action.action(self, operation);

        if !operation.is_pending() {
            self.cache.insert(operation.identity, operation.status.clone());
        }
    }

    #[inline]
    pub fn execute<'source>(&mut self, operation: &mut Operation<'source>) -> Status {
        loop {
            self.build(operation);

            match &operation.status {
                Status::Pending => {
                    sleep(Duration::from_millis(10));
                }
                Status::Resolved(_) | Status::Rejected => break operation.status.clone(),
            }
        }
    }

    #[inline]
    pub fn watch<'source>(&mut self, operation: &mut Operation<'source>, paths: &[&str]) {
        let mut last: Vec<_> = paths
            .iter()
            .map(|path| std::fs::metadata(path).and_then(|m| m.modified()).ok())
            .collect();

        loop {
            self.cache.clear();
            operation.status = Status::Pending;
            self.execute(operation);

            loop {
                sleep(Duration::from_millis(500));
                
                let current: Vec<_> = paths
                    .iter()
                    .map(|path| std::fs::metadata(path).and_then(|m| m.modified()).ok())
                    .collect();

                if current != last {
                    last = current;
                    break;
                }
            }
        }
    }
}

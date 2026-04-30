use crate::{
    combinator::{Depend, Operation, Pulse, Resolve, Status},
    data::Identity,
    internal::{
        hash::Map,
        platform::{metadata, sleep},
        time::Duration,
    },
};



struct Cache;

impl Cache {
    #[inline]
    pub fn get(cache: &Map<Identity, Status>, identity: Identity) -> Option<Status> {
        cache.get(&identity).cloned()
    }

    #[inline]
    pub fn put(cache: &mut Map<Identity, Status>, identity: Identity, status: Status) {
        cache.insert(identity, status);
    }

    #[inline]
    pub fn reset(cache: &mut Map<Identity, Status>) {
        cache.clear();
    }
}


impl Resolve {
    #[inline]
    pub fn run<'source, Store: Clone + Send + Sync>(operator: &mut Operator<Store>, operation: &mut Operation<'source, Store>) {
        let combinator = operation.combinator.clone();
        combinator.combinator(operator, operation);
    }
}

impl Depend {
    #[inline]
    pub fn run<'source, Store: Clone + Send + Sync>(operator: &mut Operator<Store>, operation: &mut Operation<'source, Store>) -> bool {
        for dependency in &operation.depends {
            if let Some(status) = operator.cache.get(dependency) {
                if !matches!(status, Status::Resolved(_)) {
                    operation.set_reject();
                    return false;
                }
            } else {
                operation.set_pending();
                return false;
            }
        }
        true
    }
}

impl Pulse {
    #[inline]
    pub fn tick(&self) {
        sleep(Duration::from_millis(self.idle));
    }
}
pub struct Operator<Store = ()> {
    pub cache: Map<Identity, Status>,
    pub store: Store,
}

impl<Store: Clone + Send + Sync> Operator<Store> {
    #[inline]
    pub fn new(store: Store) -> Self {
        Self {
            cache: Map::new(),
            store,
        }
    }

    #[inline]
    pub fn build<'source>(&mut self, operation: &mut Operation<'source, Store>) {
        if let Some(status) = Cache::get(&self.cache, operation.identity) {
            operation.status = status;
            return;
        }

        if !Depend::run(self, operation) {
            return;
        }

        Resolve::run(self, operation);

        if !operation.is_pending() {
            Cache::put(&mut self.cache, operation.identity, operation.status.clone());
        }
    }

    #[inline]
    pub fn execute<'source>(&mut self, operation: &mut Operation<'source, Store>) -> Status {
        loop {
            self.build(operation);

            match &operation.status {
                Status::Pending => {
                    Pulse { idle: 10 }.tick();
                }
                Status::Resolved(_) | Status::Rejected => break operation.status.clone(),
            }
        }
    }

    #[inline]
    pub fn watch<'source>(&mut self, operation: &mut Operation<'source, Store>, paths: &[&str]) {
        let mut last: Vec<_> = paths
            .iter()
            .map(|path| metadata(path).and_then(|m| m.modified()).ok())
            .collect();

        loop {
            Cache::reset(&mut self.cache);
            operation.status = Status::Pending;
            self.execute(operation);

            loop {
                Pulse { idle: 500 }.tick();

                let current: Vec<_> = paths
                    .iter()
                    .map(|path| metadata(path).and_then(|m| m.modified()).ok())
                    .collect();

                if current != last {
                    last = current;
                    break;
                }
            }
        }
    }
}

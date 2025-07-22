use {
    super::{
        order::Order,
    },
    crate::{
        hash::{
            Hasher,
            Hash,
        },
        any::TypeId,
        format::Debug,
    }
};

fn fingerprint<T: ?Sized + 'static>(ptr: &T, state: &mut impl Hasher) {
    TypeId::of::<T>().hash(state);
    (ptr as *const T as *const () as usize).hash(state);
}

fn identicality<T: ?Sized + 'static, U: ?Sized + 'static>(ptr1: &T, ptr2: &U) -> bool {
    if TypeId::of::<T>() != TypeId::of::<U>() {
        return false;
    }
    ptr1 as *const T as *const () == ptr2 as *const U as *const ()
}

impl<Input, Output, Failure> Hash for Order<Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Order::Align => {
                0u8.hash(state);
            }
            Order::Branch { found, missing } => {
                1u8.hash(state);
                found.hash(state);
                missing.hash(state);
            }
            Order::Fail(emitter) => {
                2u8.hash(state);
                fingerprint(emitter.as_ref(), state);
            }
            Order::Ignore => {
                3u8.hash(state);
            }
            Order::Inspect(inspector) => {
                4u8.hash(state);
                fingerprint(inspector.as_ref(), state);
            }
            Order::Multiple(actions) => {
                5u8.hash(state);
                actions.hash(state);
            }
            Order::Panic(emitter) => {
                6u8.hash(state);
                fingerprint(emitter.as_ref(), state);
            }
            Order::Pardon => {
                7u8.hash(state);
            }
            Order::Perform(executor) => {
                8u8.hash(state);
                fingerprint(executor.as_ref(), state);
            }
            Order::Skip => {
                9u8.hash(state);
            }
            Order::Transform(transformer) => {
                10u8.hash(state);
                fingerprint(transformer.as_ref(), state);
            }
        }
    }
}

impl<Input, Output, Failure> PartialEq for Order<Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Order::Align, Order::Align) => true,
            (Order::Branch { found: f1, missing: m1 }, Order::Branch { found: f2, missing: m2 }) => {
                f1 == f2 && m1 == m2
            }
            (Order::Fail(f1), Order::Fail(f2)) => identicality(f1.as_ref(), f2.as_ref()),
            (Order::Ignore, Order::Ignore) => true,
            (Order::Inspect(i1), Order::Inspect(i2)) => identicality(i1.as_ref(), i2.as_ref()),
            (Order::Multiple(a1), Order::Multiple(a2)) => a1 == a2,
            (Order::Panic(f1), Order::Panic(f2)) => identicality(f1.as_ref(), f2.as_ref()),
            (Order::Pardon, Order::Pardon) => true,
            (Order::Perform(e1), Order::Perform(e2)) => identicality(e1.as_ref(), e2.as_ref()),
            (Order::Skip, Order::Skip) => true,
            (Order::Transform(t1), Order::Transform(t2)) => identicality(t1.as_ref(), t2.as_ref()),
            _ => false,
        }
    }
}

impl<Input, Output, Failure> Eq for Order<Input, Output, Failure>
where
    Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Output: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    Failure: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
{}
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
        axo_cursor::Spanned,
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
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Order::Convert(transformer) => {
                0u8.hash(state);
                fingerprint(transformer.as_ref(), state);
            }
            Order::Perform(executor) => {
                1u8.hash(state);
                fingerprint(executor.as_ref(), state);
            }
            Order::Multiple(actions) => {
                2u8.hash(state);
                actions.hash(state);
            }
            Order::Trigger { found, missing } => {
                3u8.hash(state);
                found.hash(state);
                missing.hash(state);
            }
            Order::Capture(identifier) => {
                4u8.hash(state);
                identifier.hash(state);
            }
            Order::Ignore => {
                5u8.hash(state);
            }
            Order::Skip => {
                6u8.hash(state);
            }
            Order::Pardon => {
                7u8.hash(state);
            }
            Order::Tweak(tweaker) => {
                8u8.hash(state);
                fingerprint(tweaker.as_ref(), state);
            }
            Order::Remove => {
                9u8.hash(state);
            }
            Order::Failure(emitter) => {
                10u8.hash(state);
                fingerprint(emitter.as_ref(), state);
            }
        }
    }
}

impl<Input, Output, Failure> PartialEq for Order<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Order::Convert(t1), Order::Convert(t2)) => {
                identicality(t1.as_ref(), t2.as_ref())
            }
            (Order::Perform(e1), Order::Perform(e2)) => {
                identicality(e1.as_ref(), e2.as_ref())
            }
            (Order::Multiple(a1), Order::Multiple(a2)) => a1 == a2,
            (
                Order::Trigger { found: f1, missing: m1 },
                Order::Trigger { found: f2, missing: m2 }
            ) => f1 == f2 && m1 == m2,
            (Order::Capture(id1), Order::Capture(id2)) => {
                id1 == id2
            }
            (Order::Ignore, Order::Ignore) => true,
            (Order::Skip, Order::Skip) => true,
            (Order::Pardon, Order::Pardon) => true,
            (Order::Tweak(t1), Order::Tweak(t2)) => {
                identicality(t1.as_ref(), t2.as_ref())
            }
            (Order::Remove, Order::Remove) => true,
            (Order::Failure(f1), Order::Failure(f2)) => {
                identicality(f1.as_ref(), f2.as_ref())
            }
            _ => false,
        }
    }
}

impl<Input, Output, Failure> Eq for Order<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{}
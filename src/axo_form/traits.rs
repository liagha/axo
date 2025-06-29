use {
    super::{
        order::Order,
        pattern::{PatternKind}
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

impl<Input, Output, Failure> Hash for PatternKind<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            PatternKind::Alternative { patterns } => {
                0u8.hash(state);
                patterns.hash(state);
            }
            PatternKind::Predicate { function } => {
                1u8.hash(state);
                fingerprint(function.as_ref(), state);
            }
            PatternKind::Deferred { function } => {
                2u8.hash(state);
                fingerprint(function.as_ref(), state);
            }
            PatternKind::Identical { value } => {
                3u8.hash(state);
                fingerprint(value.as_ref(), state);
            }
            PatternKind::Reject { pattern } => {
                4u8.hash(state);
                pattern.hash(state);
            }
            PatternKind::Optional { pattern } => {
                5u8.hash(state);
                pattern.hash(state);
            }
            PatternKind::Repetition { pattern, minimum, maximum } => {
                6u8.hash(state);
                pattern.hash(state);
                minimum.hash(state);
                maximum.hash(state);
            }
            PatternKind::Sequence { patterns } => {
                7u8.hash(state);
                patterns.hash(state);
            }
            PatternKind::Wrapper { pattern } => {
                8u8.hash(state);
                pattern.hash(state);
            }
        }
    }
}

impl<Input, Output, Failure> PartialEq for PatternKind<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                PatternKind::Alternative { patterns: patterns1 },
                PatternKind::Alternative { patterns: patterns2 },
            ) => patterns1 == patterns2,

            (
                PatternKind::Predicate { function: func1 },
                PatternKind::Predicate { function: func2 },
            ) => identicality(func1.as_ref(), func2.as_ref()),

            (
                PatternKind::Deferred { function: func1 },
                PatternKind::Deferred { function: func2 },
            ) => identicality(func1.as_ref(), func2.as_ref()),

            (
                PatternKind::Identical { value: value1 },
                PatternKind::Identical { value: value2 },
            ) => identicality(value1.as_ref(), value2.as_ref()),

            (
                PatternKind::Reject { pattern: pattern1 },
                PatternKind::Reject { pattern: pattern2 },
            ) => pattern1 == pattern2,

            (
                PatternKind::Optional { pattern: pattern1 },
                PatternKind::Optional { pattern: pattern2 },
            ) => pattern1 == pattern2,

            (
                PatternKind::Repetition { pattern: pattern1, minimum: min1, maximum: max1 },
                PatternKind::Repetition { pattern: pattern2, minimum: min2, maximum: max2 },
            ) => pattern1 == pattern2 && min1 == min2 && max1 == max2,

            (
                PatternKind::Sequence { patterns: patterns1 },
                PatternKind::Sequence { patterns: patterns2 },
            ) => patterns1 == patterns2,

            (
                PatternKind::Wrapper { pattern: pattern1 },
                PatternKind::Wrapper { pattern: pattern2 },
            ) => pattern1 == pattern2,

            _ => false,
        }
    }
}

impl<Input, Output, Failure> Eq for PatternKind<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{}

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
            Order::Shift(shifter) => {
                7u8.hash(state);
                fingerprint(shifter.as_ref(), state);
            }
            Order::Pardon => {
                8u8.hash(state);
            }
            Order::Tweak(tweaker) => {
                9u8.hash(state);
                fingerprint(tweaker.as_ref(), state);
            }
            Order::Remove => {
                10u8.hash(state);
            }
            Order::Failure(emitter) => {
                11u8.hash(state);
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
            (Order::Shift(s1), Order::Shift(s2)) => {
                identicality(s1.as_ref(), s2.as_ref())
            }
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
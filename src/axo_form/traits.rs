use {
    super::{
        order::{Order},
        helper::{fingerprint, identicality},
        pattern::{PatternKind}
    },

    crate::{
        hash::{
            Hasher,
            Hash,
        },
        format::{Debug},
        axo_cursor::Spanned,
    }
};

impl<Input, Output, Failure> Hash for PatternKind<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            PatternKind::Alternative { patterns, order, finish } => {
                0u8.hash(state);
                patterns.hash(state);
                order.hash(state);
                finish.hash(state);
            }
            PatternKind::Predicate { function, align, miss } => {
                1u8.hash(state);
                fingerprint(function.as_ref(), state);
                align.hash(state);
                miss.hash(state);
            }
            PatternKind::Deferred { function, order } => {
                2u8.hash(state);
                fingerprint(function.as_ref(), state);
                order.hash(state);
            }
            PatternKind::Reject { pattern, align, miss } => {
                3u8.hash(state);
                pattern.hash(state);
                align.hash(state);
                miss.hash(state);
            }
            PatternKind::Identical { value, align, miss } => {
                4u8.hash(state);
                fingerprint(value.as_ref(), state);
                align.hash(state);
                miss.hash(state);
            }
            PatternKind::Repetition { pattern, minimum, maximum, order, lack, exceed, finish } => {
                5u8.hash(state);
                pattern.hash(state);
                minimum.hash(state);
                maximum.hash(state);
                order.hash(state);
                lack.hash(state);
                exceed.hash(state);
                finish.hash(state);
            }
            PatternKind::Sequence { patterns, order, finish } => {
                6u8.hash(state);
                patterns.hash(state);
                order.hash(state);
                finish.hash(state);
            }
            PatternKind::Wrapper { pattern, order } => {
                7u8.hash(state);
                pattern.hash(state);
                order.hash(state);
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
                PatternKind::Alternative { patterns: patterns1, order: order1, finish: finish1 },
                PatternKind::Alternative { patterns: patterns2, order: order2, finish: finish2 },
            ) => patterns1 == patterns2 && order1 == order2 && finish1 == finish2,

            (
                PatternKind::Predicate { function: func1, align: align1, miss: miss1 },
                PatternKind::Predicate { function: func2, align: align2, miss: miss2 },
            ) => identicality(func1.as_ref(), func2.as_ref()) && align1 == align2 && miss1 == miss2,

            (
                PatternKind::Deferred { function: func1, order: order1 },
                PatternKind::Deferred { function: func2, order: order2 },
            ) => identicality(func1.as_ref(), func2.as_ref()) && order1 == order2,

            (
                PatternKind::Reject { pattern: pattern1, align: align1, miss: miss1 },
                PatternKind::Reject { pattern: pattern2, align: align2, miss: miss2 },
            ) => pattern1 == pattern2 && align1 == align2 && miss1 == miss2,

            (
                PatternKind::Identical { value: value1, align: align1, miss: miss1 },
                PatternKind::Identical { value: value2, align: align2, miss: miss2 },
            ) => identicality(value1.as_ref(), value2.as_ref()) && align1 == align2 && miss1 == miss2,

            (
                PatternKind::Repetition {
                    pattern: pattern1,
                    minimum: min1,
                    maximum: max1,
                    order: order1,
                    lack: lack1,
                    exceed: exceed1,
                    finish: finish1
                },
                PatternKind::Repetition {
                    pattern: pattern2,
                    minimum: min2,
                    maximum: max2,
                    order: order2,
                    lack: lack2,
                    exceed: exceed2,
                    finish: finish2
                },
            ) => pattern1 == pattern2 && min1 == min2 && max1 == max2 &&
                order1 == order2 && lack1 == lack2 && exceed1 == exceed2 && finish1 == finish2,

            (
                PatternKind::Sequence { patterns: patterns1, order: order1, finish: finish1 },
                PatternKind::Sequence { patterns: patterns2, order: order2, finish: finish2 },
            ) => patterns1 == patterns2 && order1 == order2 && finish1 == finish2,

            (
                PatternKind::Wrapper { pattern: pattern1, order: order1 },
                PatternKind::Wrapper { pattern: pattern2, order: order2 },
            ) => pattern1 == pattern2 && order1 == order2,

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
            Order::Capture(identifier) => {
                0u8.hash(state);
                identifier.hash(state);
            }
            Order::Convert(transformer) => {
                1u8.hash(state);
                fingerprint(transformer.as_ref(), state);
            }
            Order::Failure(emitter) => {
                2u8.hash(state);
                fingerprint(emitter.as_ref(), state);
            }
            Order::Perform(executor) => {
                3u8.hash(state);
                fingerprint(executor.as_ref(), state);
            }
            Order::Inspect(inspector) => {
                4u8.hash(state);
                fingerprint(inspector.as_ref(), state);
            }
            Order::Trigger { found, missing } => {
                5u8.hash(state);
                found.hash(state);
                missing.hash(state);
            }
            Order::Multiple(actions) => {
                6u8.hash(state);
                actions.hash(state);
            }
            Order::Pulse(pulse) => {
                7u8.hash(state);
                pulse.hash(state);
            }
            Order::Yawn => {
                8u8.hash(state);
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
            (Order::Capture(id1), Order::Capture(id2)) => id1 == id2,
            (Order::Convert(t1), Order::Convert(t2)) => identicality(t1.as_ref(), t2.as_ref()),
            (Order::Failure(f1), Order::Failure(f2)) => identicality(f1.as_ref(), f2.as_ref()),
            (Order::Perform(e1), Order::Perform(e2)) => identicality(e1.as_ref(), e2.as_ref()),
            (Order::Inspect(i1), Order::Inspect(i2)) => identicality(i1.as_ref(), i2.as_ref()),
            (
                Order::Trigger { found: f1, missing: m1 },
                Order::Trigger { found: f2, missing: m2 }
            ) => f1 == f2 && m1 == m2,
            (Order::Multiple(a1), Order::Multiple(a2)) => a1 == a2,
            (Order::Pulse(p1), Order::Pulse(p2)) => p1 == p2,
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
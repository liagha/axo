use std::hash::{Hash, Hasher};
use crate::{
    hash::Hash as CustomHash,
    format::Debug,
    axo_cursor::Spanned,
};

use super::{
    order::Order,
    pattern::{PatternKind}
};

fn hash_function_ptr<T: ?Sized>(ptr: &T) -> u64 {
    ptr as *const T as *const () as usize as u64
}

impl<Input, Output, Failure> Hash for PatternKind<Input, Output, Failure>
where
    Input: Spanned + Clone + CustomHash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + CustomHash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + CustomHash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            PatternKind::Alternative { patterns } => {
                0u8.hash(state);
                patterns.hash(state);
            }
            PatternKind::Predicate { function } => unsafe {
                1u8.hash(state);
                let ptr = function.as_ref() as *const _ as *const ();
                hash_function_ptr(&*ptr).hash(state);
            }
            PatternKind::Deferred { function } => unsafe {
                2u8.hash(state);
                let ptr = function.as_ref() as *const _ as *const ();
                hash_function_ptr(&*ptr).hash(state);
            }
            PatternKind::Literal { value } => {
                3u8.hash(state);
                value.hash(state);
            }
            PatternKind::Identical { value } => unsafe {
                4u8.hash(state);
                let ptr = value.as_ref() as *const _ as *const ();
                hash_function_ptr(&*ptr).hash(state);
            }
            PatternKind::Negation { pattern } => {
                5u8.hash(state);
                pattern.hash(state);
            }
            PatternKind::Optional { pattern } => {
                6u8.hash(state);
                pattern.hash(state);
            }
            PatternKind::Repetition { pattern, minimum, maximum } => {
                7u8.hash(state);
                pattern.hash(state);
                minimum.hash(state);
                maximum.hash(state);
            }
            PatternKind::Sequence { patterns } => {
                8u8.hash(state);
                patterns.hash(state);
            }
            PatternKind::Wrapper { pattern } => {
                9u8.hash(state);
                pattern.hash(state);
            }
        }
    }
}

impl<Input, Output, Failure> PartialEq for PatternKind<Input, Output, Failure>
where
    Input: Spanned + Clone + CustomHash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + CustomHash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + CustomHash + Eq + PartialEq + Debug + Send + Sync + 'static,
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
            ) => {
                let ptr1 = func1.as_ref() as *const _ as *const ();
                let ptr2 = func2.as_ref() as *const _ as *const ();
                ptr1 == ptr2
            }

            (
                PatternKind::Deferred { function: func1 },
                PatternKind::Deferred { function: func2 },
            ) => {
                let ptr1 = func1.as_ref() as *const _ as *const ();
                let ptr2 = func2.as_ref() as *const _ as *const ();
                ptr1 == ptr2
            }

            (
                PatternKind::Literal { value: value1 },
                PatternKind::Literal { value: value2 },
            ) => value1 == value2,

            (
                PatternKind::Identical { value: value1 },
                PatternKind::Identical { value: value2 },
            ) => {
                let ptr1 = value1.as_ref() as *const _ as *const ();
                let ptr2 = value2.as_ref() as *const _ as *const ();
                ptr1 == ptr2
            }

            (
                PatternKind::Negation { pattern: pattern1 },
                PatternKind::Negation { pattern: pattern2 },
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
    Input: Spanned + Clone + CustomHash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + CustomHash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + CustomHash + Eq + PartialEq + Debug + Send + Sync + 'static,
{}

#[cfg(test)]
mod tests {
    use {
        hashish::HashMap,
        super::{
            *,
            super::{
                pattern::*,
                functions::Predicate,
            },
        },
        crate::{
            thread::{Arc, Mutex},
            axo_cursor::{
                Span,
            },
        }
    };

    #[derive(Clone, Debug, Hash, PartialEq, Eq)]
    struct MockSpanned;

    impl Spanned for MockSpanned {
        fn span(&self) -> Span {
            Span::default()
        }
    }


    type TestAction = Order<MockSpanned, MockSpanned, MockSpanned>;

    #[test]
    fn test_action_equality() {
        let action1 = TestAction::ignore();
        let action2 = TestAction::ignore();
        let action3 = TestAction::skip();

        assert_eq!(action1, action2);
        assert_ne!(action1, action3);
    }

    #[test]
    fn test_action_capture_equality() {
        let action1 = TestAction::capture(42);
        let action2 = TestAction::capture(42);
        let action3 = TestAction::capture(24);

        assert_eq!(action1, action2);
        assert_ne!(action1, action3);
    }

    #[test]
    fn test_action_multiple_equality() {
        let action1 = TestAction::multiple(vec![
            TestAction::ignore(),
            TestAction::skip(),
        ]);
        let action2 = TestAction::multiple(vec![
            TestAction::ignore(),
            TestAction::skip(),
        ]);
        let action3 = TestAction::multiple(vec![
            TestAction::skip(),
            TestAction::ignore(),
        ]);

        assert_eq!(action1, action2);
        assert_ne!(action1, action3);
    }

    #[test]
    fn test_action_hash() {
        let mut map = HashMap::new();

        let action1 = TestAction::ignore();
        let action2 = TestAction::capture(42);

        map.insert(action1, "ignore_value");
        map.insert(action2, "capture_value");

        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&TestAction::ignore()), Some(&"ignore_value"));
        assert_eq!(map.get(&TestAction::capture(42)), Some(&"capture_value"));
    }

    #[test]
    fn test_function_actions_inequality() {
        let action1 = TestAction::map(|_, _| Ok(MockSpanned));
        let action2 = TestAction::map(|_, _| Ok(MockSpanned));

        assert_ne!(action1, action2);
    }


    type TestPattern = Pattern<MockSpanned, MockSpanned, MockSpanned>;
    type TestPatternKind = PatternKind<MockSpanned, MockSpanned, MockSpanned>;

    #[test]
    fn test_literal_equality() {
        let kind1 = TestPatternKind::Literal { value: MockSpanned };
        let kind2 = TestPatternKind::Literal { value: MockSpanned };
        assert_eq!(kind1, kind2);
    }

    #[test]
    fn test_alternative_equality() {
        let patterns = vec![TestPattern::literal(MockSpanned)];
        let kind1 = TestPatternKind::Alternative { patterns: patterns.clone() };
        let kind2 = TestPatternKind::Alternative { patterns };
        assert_eq!(kind1, kind2);
    }

    #[test]
    fn test_predicate_equality() {
        let pred_fn = |_: &MockSpanned| true;
        let predicate1: Predicate<MockSpanned> = Arc::new(Mutex::new(pred_fn));
        let predicate2 = predicate1.clone();

        let kind1 = TestPatternKind::Predicate { function: predicate1 };
        let kind2 = TestPatternKind::Predicate { function: predicate2 };

        assert_eq!(kind1, kind2);
    }

    #[test]
    fn test_different_variants_not_equal() {
        let kind1 = TestPatternKind::Literal { value: MockSpanned };
        let kind2 = TestPatternKind::Optional {
            pattern: Box::new(TestPattern::literal(MockSpanned))
        };
        assert_ne!(kind1, kind2);
    }

    #[test]
    fn test_hash_consistency() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let kind = TestPatternKind::Literal { value: MockSpanned };

        let mut hasher1 = DefaultHasher::new();
        kind.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        kind.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2);
    }
}

impl<Input, Output, Failure> Hash for Order<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Order::Convert(_) => {
                0u8.hash(state);
                "map".hash(state);
            }
            Order::Perform(_) => {
                1u8.hash(state);
                "perform".hash(state);
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
            Order::Capture { identifier } => {
                4u8.hash(state);
                identifier.hash(state);
            }
            Order::Ignore => {
                5u8.hash(state);
                "ignore".hash(state);
            }
            Order::Skip => {
                6u8.hash(state);
                "skip".hash(state);
            }
            Order::Shift(_) => {
                7u8.hash(state);
                "shift".hash(state);
            }
            Order::Pardon => {
                8u8.hash(state);
                "pardon".hash(state);
            }
            Order::Tweak(_) => {
                9u8.hash(state);
                "tweak".hash(state);
            }
            Order::Remove => {
                10u8.hash(state);
                "remove".hash(state);
            }
            Order::Failure(_) => {
                11u8.hash(state);
                "failure".hash(state);
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
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl<Input, Output, Failure> Eq for Order<Input, Output, Failure>
where
    Input: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Output: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
    Failure: Spanned + Clone + Hash + Eq + PartialEq + Debug + Send + Sync + 'static,
{}
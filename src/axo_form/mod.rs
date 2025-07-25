#![allow(dead_code)]

mod format;
mod traits;
pub mod pattern;
pub mod order;
pub mod form;
pub mod former;

pub mod helper {
    use {
        super::{
            form::Form,
            former::Draft,
            order::Order,
            pattern::Classifier,
        },
        crate::{
            any::TypeId,
            axo_cursor::Peekable,
            axo_internal::{
                compiler::{
                    Registry, Marked
                },
            },
            format::Debug,
            hash::{Hash, Hasher},
            thread::{Arc, Mutex},
        },
    };

    pub trait Source<Input>: Peekable<Input> + Marked
    where
        Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    {}

    impl<Target, Input> Source<Input> for Target
    where
        Target: Peekable<Input> + Marked,
        Input: Clone + Debug + Eq + Hash + PartialEq + Send + Sync + 'static,
    {}

    pub fn fingerprint<T: ?Sized + 'static>(ptr: &T, state: &mut impl Hasher) {
        TypeId::of::<T>().hash(state);
        (ptr as *const T as *const () as usize).hash(state);
    }

    pub fn identicality<T: ?Sized + 'static, U: ?Sized + 'static>(ptr1: &T, ptr2: &U) -> bool {
        if TypeId::of::<T>() != TypeId::of::<U>() {
            return false;
        }

        ptr1 as *const T as *const () == ptr2 as *const U as *const ()
    }

    pub type Emitter<Input, Output, Failure> = Arc<dyn Fn(&mut Registry, Form<Input, Output, Failure>) -> Failure + Send + Sync>;
    pub type Evaluator<Input, Output, Failure> = Arc<dyn Fn() -> Classifier<Input, Output, Failure> + Send + Sync>;
    pub type Executor = Arc<Mutex<dyn FnMut() -> () + Send + Sync>>;
    pub type Inspector<Input, Output, Failure> = Arc<dyn Fn(Draft<Input, Output, Failure>) -> Order<Input, Output, Failure> + Send + Sync>;
    pub type Predicate<Input> = Arc<dyn Fn(&Input) -> bool + Send + Sync>;
    pub type Transformer<Input, Output, Failure> = Arc<Mutex<dyn FnMut(&mut Registry, Form<Input, Output, Failure>) -> Result<Form<Input, Output, Failure>, Failure> + Send + Sync>>;
}
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
            order::Order,
            form::Form,
            former::Draft,
            pattern::Pattern,
        },
        crate::{
            any::TypeId,
            hash::{Hash, Hasher},
            thread::{Arc, Mutex},
            compiler::Context,
            axo_cursor::Position,
        },
    };

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

    pub type Emitter<Input, Output, Failure> = Arc<dyn Fn(&mut Context, Form<Input, Output, Failure>) -> Failure + Send + Sync>;
    pub type Evaluator<Input, Output, Failure> = Arc<dyn Fn() -> Pattern<Input, Output, Failure> + Send + Sync>;
    pub type Executor = Arc<Mutex<dyn FnMut() -> () + Send + Sync>>;
    pub type Inspector<Input, Output, Failure> = Arc<dyn Fn(Draft<Input, Output, Failure>) -> Order<Input, Output, Failure> + Send + Sync>;
    pub type Predicate<Input> = Arc<dyn Fn(&Input) -> bool + Send + Sync>;
    pub type Shifter = Arc<dyn Fn(&mut usize, &mut Position)>;
    pub type Transformer<Input, Output, Failure> = Arc<Mutex<dyn FnMut(&mut Context, Form<Input, Output, Failure>) -> Result<Output, Failure> + Send + Sync>>;
    pub type Tweaker<Input, Output, Failure> = Arc<dyn Fn(&mut Draft<Input, Output, Failure>) + Send + Sync>;
}
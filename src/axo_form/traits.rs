use {
    crate::{
        hash::{
            Hasher,
            Hash,
        },
        any::TypeId,
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
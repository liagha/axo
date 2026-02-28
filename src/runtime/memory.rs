use crate::internal::platform::{alloc, dealloc, realloc, Layout, NonNull};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocationError;

pub type Result<T> = std::result::Result<T, AllocationError>;

#[inline]
pub fn next_capacity(current: usize, additional: usize) -> Result<usize> {
    let required = current.checked_add(additional).ok_or(AllocationError)?;

    if required == 0 {
        return Ok(0);
    }

    let mut capacity = current.max(8);

    while capacity < required {
        capacity = capacity.checked_mul(2).ok_or(AllocationError)?;
    }

    Ok(capacity)
}

#[inline]
pub unsafe fn alloc_raw(size: usize, align: usize) -> Result<NonNull<u8>> {
    let layout = Layout::from_size_align(size, align).map_err(|_| AllocationError)?;
    let ptr = unsafe { alloc(layout) };
    NonNull::new(ptr).ok_or(AllocationError)
}

#[inline]
pub unsafe fn realloc_raw(
    ptr: NonNull<u8>,
    old_size: usize,
    new_size: usize,
    align: usize,
) -> Result<NonNull<u8>> {
    let old_layout = Layout::from_size_align(old_size, align).map_err(|_| AllocationError)?;
    let new_ptr = unsafe { realloc(ptr.as_ptr(), old_layout, new_size) };
    NonNull::new(new_ptr).ok_or(AllocationError)
}

#[inline]
pub unsafe fn free_raw(ptr: NonNull<u8>, size: usize, align: usize) -> Result<()> {
    let layout = Layout::from_size_align(size, align).map_err(|_| AllocationError)?;
    unsafe { dealloc(ptr.as_ptr(), layout) };
    Ok(())
}

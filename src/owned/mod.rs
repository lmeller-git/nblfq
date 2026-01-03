pub(crate) mod buffer;
mod queue;
pub use queue::{PooledQueue, Queue};

use core::ptr::NonNull;

use crate::core::PtrLike;

unsafe impl<T> PtrLike for alloc::boxed::Box<T> {
    type Item = T;
    fn as_ptr(zelf: Self) -> NonNull<Self::Item> {
        NonNull::new(alloc::boxed::Box::into_raw(zelf)).unwrap()
    }

    fn from_raw(raw: NonNull<Self::Item>) -> Self {
        unsafe { alloc::boxed::Box::from_raw(raw.as_ptr()) }
    }
}

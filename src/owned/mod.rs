pub(crate) mod buffer;
mod queue;
#[cfg(feature = "pool")]
pub use queue::PooledQueue;
pub use queue::Queue;

use core::ptr::NonNull;

use crate::core::PtrLike;

// SAFETY:
// Box<T> is nonnull.
// The caller must ensure its validity for the lifetime of the NonNull returned by `as_ptr`
unsafe impl<T> PtrLike for alloc::boxed::Box<T> {
    type Item = T;
    fn as_ptr(zelf: Self) -> NonNull<Self::Item> {
        NonNull::new(alloc::boxed::Box::into_raw(zelf)).unwrap()
    }

    fn from_raw(raw: NonNull<Self::Item>) -> Self {
        // SAFETY:
        // The caller must ensure that:
        // - raw was retrieved by PtrLike::as_raw
        // - the underlying alocation was not freed
        // - from_raw is called exactly once on a ptr retrieved from PtrLike::as_raw
        unsafe { alloc::boxed::Box::from_raw(raw.as_ptr()) }
    }
}

pub(crate) mod buffer;
mod queue;
pub use queue::Queue;

use crate::slot::PtrLike;

unsafe impl<T> PtrLike for alloc::boxed::Box<T> {
    type Item = T;
    fn as_ptr(zelf: Self) -> *mut Self::Item {
        alloc::boxed::Box::into_raw(zelf)
    }

    fn from_raw(raw: *mut Self::Item) -> Option<Self> {
        if raw.is_null() {
            return None;
        }
        Some(unsafe { alloc::boxed::Box::from_raw(raw as *mut T) })
    }
}

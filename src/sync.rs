#[cfg(not(loom))]
pub mod cell {
    #[derive(Debug)]
    pub(crate) struct UnsafeCell<T>(core::cell::UnsafeCell<T>);

    #[allow(dead_code)]
    impl<T> UnsafeCell<T> {
        pub(crate) fn new(data: T) -> UnsafeCell<T> {
            UnsafeCell(core::cell::UnsafeCell::new(data))
        }

        pub(crate) fn with<R>(&self, f: impl FnOnce(*const T) -> R) -> R {
            f(self.0.get())
        }

        pub(crate) fn with_mut<R>(&self, f: impl FnOnce(*mut T) -> R) -> R {
            f(self.0.get())
        }
    }

    impl<T: Default> Default for UnsafeCell<T> {
        fn default() -> Self {
            Self::new(T::default())
        }
    }
}

#[cfg(all(loom, test))]
pub use loom::sync::Arc;

#[cfg(loom)]
pub use loom::cell;

#[cfg(loom)]
pub use loom::hint;

#[cfg(not(loom))]
pub use core::hint;

#[cfg(loom)]
pub use loom::sync::atomic;

#[cfg(not(loom))]
pub use portable_atomic as atomic;

#[cfg(all(loom, test))]
pub use loom::thread;

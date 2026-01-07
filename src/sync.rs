#[cfg(any(not(test), all(not(loom), not(shuttle))))]
pub use core_::*;
#[cfg(all(loom, test))]
pub use loom_::*;
#[cfg(all(shuttle, test))]
pub use shuttle_::*;

#[cfg(all(shuttle, test))]
mod shuttle_ {
    #[allow(unused_imports)]
    pub use shuttle::hint;
    pub use shuttle::sync::atomic;
    pub use shuttle::thread;

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
}

#[cfg(all(loom, test))]
mod loom_ {
    pub use loom::cell;
    // pub use loom::hint;
    pub use loom::sync::Arc;
    pub use loom::sync::atomic;
    pub use loom::thread;
}

#[cfg(any(not(test), all(not(loom), not(shuttle))))]
mod core_ {
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
    #[cfg(not(feature = "std"))]
    pub use core::hint;
    pub use portable_atomic as atomic;

    #[cfg(feature = "std")]
    pub use std::thread;
}

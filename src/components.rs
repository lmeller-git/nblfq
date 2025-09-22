use ::core::{
    array,
    sync::atomic::{AtomicU64, Ordering},
};
use cfg_if::cfg_if;
use core::marker::PhantomData;

cfg_if! {
    if #[cfg(feature = "alloc")] {
        pub(crate) use heapless::*;
        pub(crate) use heap_based::*;
    } else {
        pub(crate) use heapless::*;
    }
}

cfg_if! {
    if #[cfg(not(feature = "no-tagged-ptr"))] {
        use tagged_ptr::*;
        pub(crate) type PtrType<T> = TaggedItemInner<T>;
    } else {
        use dword_item_portable::*;
        pub(crate) type PtrType<T> = DWordItemInner<T>;
    }
}

pub(crate) type Item<T> = GenericItem<T, PtrType<T>>;

pub(super) trait Buffer<T> {
    fn len(&self) -> usize;
    fn inner(&self) -> &[Item<T>];
}

pub(crate) trait ItemInner<T> {
    const MAX_W: u64;
    /// returns (count, ptr)
    fn components(&self) -> (u64, *const T);
    /// atomically updates count + ptr
    fn cmpxchg(
        &self,
        old_ptr: *const T,
        old_count: u64,
        new_ptr: *const T,
        new_count: u64,
    ) -> Result<(u64, *const T), (u64, *const T)>;
    fn new() -> Self;
}

mod heapless {
    use super::*;

    pub struct HeaplessBuf<const N: usize, T> {
        inner: [Item<T>; N],
    }

    impl<const N: usize, T> HeaplessBuf<N, T> {
        pub fn new() -> Self {
            Self {
                inner: array::from_fn(|_| Item::new()),
            }
        }
    }

    impl<const N: usize, T> Default for HeaplessBuf<N, T> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<const N: usize, T> Buffer<T> for HeaplessBuf<N, T> {
        fn len(&self) -> usize {
            N
        }

        fn inner(&self) -> &[Item<T>] {
            &self.inner
        }
    }
}

#[cfg(feature = "alloc")]
mod heap_based {
    use super::*;
    use alloc::boxed::Box;

    pub struct FixedBuf<T> {
        inner: Box<[Item<T>]>,
    }

    impl<T> FixedBuf<T> {
        pub fn new(size: usize) -> Self {
            Self {
                inner: (0..size).map(|_| Item::new()).collect(),
            }
        }
    }

    impl<T> Buffer<T> for FixedBuf<T> {
        fn len(&self) -> usize {
            self.inner.len()
        }

        fn inner(&self) -> &[Item<T>] {
            &self.inner
        }
    }
}

pub(crate) struct GenericItem<T, I: ItemInner<T>> {
    inner: I,
    _data: PhantomData<T>,
}

impl<T, I: ItemInner<T>> GenericItem<T, I> {
    fn new() -> Self {
        Self {
            inner: I::new(),
            _data: PhantomData,
        }
    }

    #[inline]
    pub(crate) fn components(&self) -> (u64, *const T) {
        self.inner.components()
    }

    #[inline]
    pub(crate) fn cmpxchg(
        &self,
        old_ptr: *const T,
        old_count: u64,
        new_ptr: *const T,
        new_count: u64,
    ) -> Result<(u64, *const T), (u64, *const T)> {
        self.inner.cmpxchg(old_ptr, old_count, new_ptr, new_count)
    }
}

#[cfg(not(feature = "no-tagged-ptr"))]
mod tagged_ptr {
    use super::*;
    use crate::utils::{components_as_tagged, components_from_tagged};

    pub(crate) struct TaggedItemInner<T> {
        // the pointer part takes up the first 48 bits, count the last 16
        ptr: AtomicU64,
        _data: PhantomData<*const T>,
    }

    #[allow(unused)]
    impl<T> TaggedItemInner<T> {
        pub fn from_tagged(ptr: u64) -> Self {
            Self {
                ptr: AtomicU64::new(ptr),
                _data: PhantomData,
            }
        }

        pub fn from_components(count: u64, ptr: *const T) -> Self {
            Self::from_tagged(components_as_tagged(count, ptr))
        }
    }

    impl<T> ItemInner<T> for TaggedItemInner<T> {
        const MAX_W: u64 = u16::MAX as u64 + 1;
        fn components(&self) -> (u64, *const T) {
            components_from_tagged(self.ptr.load(Ordering::Acquire))
        }

        fn new() -> Self {
            Self {
                ptr: AtomicU64::new(0),
                _data: PhantomData,
            }
        }

        fn cmpxchg(
            &self,
            old_ptr: *const T,
            old_count: u64,
            new_ptr: *const T,
            new_count: u64,
        ) -> Result<(u64, *const T), (u64, *const T)> {
            let old = components_as_tagged(old_count, old_ptr);
            let new = components_as_tagged(new_count, new_ptr);
            self.ptr
                .compare_exchange(old, new, Ordering::AcqRel, Ordering::Relaxed)
                .map(|p| components_from_tagged(p))
                .map_err(|p| components_from_tagged(p))
        }
    }
}

#[cfg(feature = "no-tagged-ptr")]
mod dword_item_portable {
    use super::*;
    use crate::utils::{components_as_dword, components_from_dword};
    use portable_atomic::AtomicU128;

    pub(crate) struct DWordItemInner<T> {
        storage: AtomicU128,
        _data: PhantomData<*const T>,
    }

    impl<T> DWordItemInner<T> {
        pub(crate) fn from_components(count: u64, ptr: *const T) -> Self {
            Self::from_dword(components_as_dword(count, ptr))
        }

        pub(crate) fn from_dword(dword: u128) -> Self {
            Self {
                storage: AtomicU128::new(dword),
                _data: PhantomData,
            }
        }
    }

    impl<T> ItemInner<T> for DWordItemInner<T> {
        const MAX_W: u64 = u64::MAX;
        fn components(&self) -> (u64, *const T) {
            components_from_dword(self.storage.load(Ordering::Acquire))
        }

        fn cmpxchg(
            &self,
            old_ptr: *const T,
            old_count: u64,
            new_ptr: *const T,
            new_count: u64,
        ) -> Result<(u64, *const T), (u64, *const T)> {
            let old = components_as_dword(old_count, old_ptr);
            let new = components_as_dword(new_count, new_ptr);
            self.storage
                .compare_exchange(old, new, Ordering::AcqRel, Ordering::Relaxed)
                .map(|dword| components_from_dword(dword))
                .map_err(|dword| components_from_dword(dword))
        }

        fn new() -> Self {
            Self::from_dword(0)
        }
    }
}

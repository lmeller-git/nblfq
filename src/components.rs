use ::core::{
    array,
    sync::atomic::{AtomicU64, Ordering},
};
#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use cfg_if::cfg_if;
use core::marker::PhantomData;

#[cfg(feature = "tagged_ptr")]
use crate::utils::{components_as_tagged, components_from_tagged};

cfg_if! {
    if #[cfg(feature = "tagged_ptr")] {
        pub(crate) type PtrType<T> = TaggedItemInner<T>;
    } else {
        pub(crate) type PtrType<T> = SplitItemInner<T>;
    }
}

pub(crate) type Item<T> = GenericItem<T, PtrType<T>>;

pub(super) trait Buffer<T> {
    fn len(&self) -> usize;
    fn inner(&self) -> &[Item<T>];
}

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

#[cfg(feature = "alloc")]
pub struct FixedBuf<T> {
    inner: Box<[Item<T>]>,
}

#[cfg(feature = "alloc")]
impl<T> FixedBuf<T> {
    pub fn new(size: usize) -> Self {
        Self {
            inner: (0..size).map(|_| Item::new()).collect(),
        }
    }
}

#[cfg(feature = "alloc")]
impl<T> Buffer<T> for FixedBuf<T> {
    fn len(&self) -> usize {
        self.inner.len()
    }

    fn inner(&self) -> &[Item<T>] {
        &self.inner
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

pub(crate) trait ItemInner<T>: Send + Sync {
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

#[cfg(feature = "tagged_ptr")]
pub(crate) struct TaggedItemInner<T> {
    // the pointer part takes up the first 48 bits, count the last 16
    ptr: AtomicU64,
    _data: PhantomData<T>,
}

#[cfg(feature = "tagged_ptr")]
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

#[cfg(feature = "tagged_ptr")]
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

#[cfg(feature = "tagged_ptr")]
unsafe impl<T> Send for TaggedItemInner<T> {}
#[cfg(feature = "tagged_ptr")]
unsafe impl<T> Sync for TaggedItemInner<T> {}

#[cfg(not(feature = "tagged_ptr"))]
pub(crate) struct SplitItemInner<T> {
    count: AtomicU64,
    data: AtomicPtr<T>,
}

#[cfg(not(feature = "tagged_ptr"))]
impl<T> ItemInner<T> for SplitItemInner<T> {
    const MAX_W: u64 = u64::MAX;
    fn components(&self) -> (u64, *const T) {
        (
            self.count.load(Ordering::Acquire),
            self.data.load(Ordering::Acquire),
        )
    }

    fn cmpxchg(
        &self,
        old_ptr: *const T,
        old_count: u64,
        new_ptr: *const T,
        new_count: u64,
    ) -> Result<(u64, *const T), (u64, *const T)> {
        todo!()
    }

    fn new() -> Self {
        use core::ptr::null_mut;

        Self {
            count: AtomicU64::new(0),
            data: AtomicPtr::new(null_mut()),
        }
    }
}

use crate::{
    MPMCQueue,
    array::buffer::ArrayBuf,
    core::{
        PtrLike,
        queue::QueueCore,
        slots::{Auto, SlotType},
    },
};

#[cfg(feature = "pool")]
pub use pooled_static::*;

/// A `MPMCQueue` with capacity `N`.
///
/// This queue only accepts items that implememt `PtrLike`.
///
/// If you need to store larger types, consider using `PooledStaticQueue` instead.
pub struct StaticQueue<T, const N: usize, S = Auto>
where
    T: PtrLike,
    S: SlotType<T>,
{
    inner: QueueCore<ArrayBuf<N, S::Slot>>,
}

impl<T, const N: usize> StaticQueue<T, N, Auto>
where
    T: PtrLike,
{
    /// Constructs a new `StaticQueue` with slot type `Auto`
    pub fn new() -> Self {
        Self::with_slot::<Auto>()
    }

    /// Constructs a new `StaticQueue` with slot type `S`
    pub fn with_slot<S>() -> StaticQueue<T, N, S>
    where
        S: SlotType<T>,
    {
        StaticQueue {
            inner: QueueCore::new_in(ArrayBuf::new()),
        }
    }
}

impl<T, const N: usize, S> MPMCQueue for StaticQueue<T, N, S>
where
    T: PtrLike,
    S: SlotType<T>,
{
    type Item = T;

    fn push(&self, item: Self::Item) -> Result<(), Self::Item> {
        self.inner.push(item)
    }

    fn pop(&self) -> Option<Self::Item> {
        self.inner.pop()
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn capacity(&self) -> usize {
        self.inner.capacity()
    }
}

impl<T, const N: usize> Default for StaticQueue<T, N, Auto>
where
    T: PtrLike,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "pool")]
mod pooled_static {
    use crate::pool::{DataStorage, IndexStorage, ItemHandle, Pooled};

    use super::*;

    /// A pooled `MPMCQueue` with capacity `N`.
    ///
    /// Unlike `StaticQueue`, this queue may store any item at the cost of higher runtime and higher memory.
    ///
    /// Only available on feature `pool`
    #[allow(private_bounds)]
    pub struct PooledStaticQueue<T, const N: usize, S = Auto>
    where
        S: SlotType<ItemHandle<T>>,
    {
        #[allow(clippy::type_complexity)]
        inner: Pooled<
            T,
            StaticQueue<ItemHandle<T>, N, S>,
            ArrayBuf<N, DataStorage<T>>,
            StaticQueue<IndexStorage, N>,
        >,
    }

    #[allow(private_bounds)]
    impl<T, const N: usize> PooledStaticQueue<T, N, Auto> {
        /// Constructs a new `PooledStaticQueue` with slot type `Auto`
        pub fn new() -> Self {
            Self::with_slot::<Auto>()
        }

        /// Constructs a new `PooledStaticQueue` with slot type `S`
        pub fn with_slot<S>() -> PooledStaticQueue<T, N, S>
        where
            S: SlotType<ItemHandle<T>>,
        {
            PooledStaticQueue {
                inner: Pooled::new_from(
                    StaticQueue::with_slot(),
                    ArrayBuf::new(),
                    StaticQueue::with_slot(),
                ),
            }
        }
    }

    impl<T, const N: usize> MPMCQueue for PooledStaticQueue<T, N> {
        type Item = T;
        fn push(&self, item: Self::Item) -> Result<(), Self::Item> {
            self.inner.push(item)
        }

        fn pop(&self) -> Option<Self::Item> {
            self.inner.pop()
        }

        fn len(&self) -> usize {
            self.inner.len()
        }

        fn capacity(&self) -> usize {
            self.inner.capacity()
        }
    }

    impl<T, const N: usize> Default for PooledStaticQueue<T, N, Auto> {
        fn default() -> Self {
            Self::new()
        }
    }
}

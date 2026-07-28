#[cfg(feature = "pool")]
pub use pooled_static::*;

use crate::{
    MPMCQueue,
    array::buffer::ArrayBuf,
    core::{
        AsPackedValue,
        queue::QueueCore,
        slots::{Auto, SlotType},
    },
};

/// A `MPMCQueue` with capacity `N`.
///
/// This queue only accepts items that implememt `PtrLike`.
///
/// If you need to store larger types, consider using `PooledStaticQueue` instead.
pub struct InlineQueue<T, const N: usize, S = Auto>
where
    T: AsPackedValue,
    S: SlotType<T>,
{
    inner: QueueCore<ArrayBuf<N, S::Slot>>,
}

impl<T, const N: usize> InlineQueue<T, N, Auto>
where
    T: AsPackedValue,
{
    /// Constructs a new `StaticQueue` with slot type `Auto`.
    /// `T` must fit into the chosen slot type
    #[track_caller]
    pub fn new() -> Self {
        Self::with_slot::<Auto>()
    }

    /// Constructs a new `StaticQueue` with slot type `S`.
    /// `T` must fit into the slot type `S`
    #[track_caller]
    pub fn with_slot<S>() -> InlineQueue<T, N, S>
    where
        S: SlotType<T>,
    {
        InlineQueue {
            inner: QueueCore::new_in(ArrayBuf::new()),
        }
    }
}

impl<T, const N: usize, S> MPMCQueue for InlineQueue<T, N, S>
where
    T: AsPackedValue,
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

impl<T, const N: usize> Default for InlineQueue<T, N, Auto>
where
    T: AsPackedValue,
{
    #[track_caller]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "pool")]
mod pooled_static {
    use super::*;
    use crate::{
        core::inline_pool_storage::InlineSlotStore,
        pool::{DataStorage, ItemHandle, Pooled},
    };

    /// A pooled `MPMCQueue` with capacity `N`.
    ///
    /// Unlike `StaticQueue`, this queue may store any item at the cost of higher runtime and higher memory.
    ///
    /// Only available on feature `pool`.
    ///
    /// For more info refer to `StaticQueue`.
    #[allow(private_bounds)]
    pub struct PooledInlineQueue<T, const N: usize, S = Auto>
    where
        S: InlineSlotStore<ItemHandle<T>, N>,
    {
        #[allow(clippy::type_complexity)]
        inner: Pooled<
            T,
            InlineQueue<ItemHandle<T>, N, S::SlotType>,
            ArrayBuf<N, DataStorage<T>>,
            S::Pool,
        >,
    }

    #[allow(private_bounds)]
    impl<T, const N: usize> PooledInlineQueue<T, N, Auto>
    where
        Auto: InlineSlotStore<ItemHandle<T>, N>,
    {
        /// Constructs a new `PooledStaticQueue` with slot type `Auto`
        #[track_caller]
        pub fn new() -> Self {
            Self::with_conf()
        }
    }

    #[allow(private_bounds)]
    impl<T, const N: usize, S> PooledInlineQueue<T, N, S>
    where
        S: InlineSlotStore<ItemHandle<T>, N>,
    {
        /// Constructs a new `PooledStaticQueue` with slot type `S::SlotType` and storage type `S::Storage`.
        #[track_caller]
        pub fn with_conf() -> PooledInlineQueue<T, N, S> {
            PooledInlineQueue {
                inner: Pooled::new_from(
                    InlineQueue::with_slot(),
                    ArrayBuf::new(),
                    S::Pool::default(),
                ),
            }
        }
    }

    impl<T, const N: usize, S> MPMCQueue for PooledInlineQueue<T, N, S>
    where
        S: InlineSlotStore<ItemHandle<T>, N>,
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

    impl<T, const N: usize> Default for PooledInlineQueue<T, N, Auto>
    where
        Auto: InlineSlotStore<ItemHandle<T>, N>,
    {
        #[track_caller]
        fn default() -> Self {
            Self::new()
        }
    }
}

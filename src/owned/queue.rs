#[cfg(all(feature = "pool", feature = "alloc"))]
pub use pooled_queue::*;

use crate::{
    MPMCQueue,
    core::{
        AsPackedValue,
        queue::QueueCore,
        slots::{Auto, SlotType},
    },
    owned::buffer::BoxedBuffer,
};

/// A `MPMCQueue` over a heap-allocated array.
///
/// This queue only accepts items that implement `PtrLike`.
///
/// If you need to store larger types, consider using `PooledQueue` instead.
///
/// only available on feature `alloc`
pub struct Queue<T, S = Auto>
where
    T: AsPackedValue,
    S: SlotType<T>,
{
    inner: QueueCore<BoxedBuffer<S::Slot>>,
}

impl<T> Queue<T, Auto>
where
    T: AsPackedValue,
{
    /// Constructs a new `Queue` with capacity `size` and slot type `Auto`.
    /// `T` must fit into the chosen slot type
    pub fn new(size: usize) -> Self {
        Self::with_slot::<Auto>(size)
    }

    /// Constructs a new `Queue` with capacity `size` and slot type `S`.
    /// `T` must fit into the slot type `S`
    pub fn with_slot<S>(size: usize) -> Queue<T, S>
    where
        S: SlotType<T>,
    {
        Queue {
            inner: QueueCore::new_in(BoxedBuffer::new(size)),
        }
    }
}

impl<T, S> MPMCQueue for Queue<T, S>
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

#[cfg(all(feature = "pool", feature = "alloc"))]
mod pooled_queue {
    use lf_slots::HeapStorage;

    use super::*;
    use crate::pool::{DataStorage, ItemHandle, Pooled};

    /// A pooled `MPMCQueue`.
    ///
    /// Unlike `Queue`, this queue may store any type, at the cost of higher runtime and higher memory.
    ///
    /// Only available on feature `alloc` + `pool`.
    ///
    /// For more info refer to `Queue`.
    #[allow(private_bounds)]
    pub struct PooledQueue<T, S = Auto>
    where
        S: SlotType<ItemHandle<T>>,
    {
        #[allow(clippy::type_complexity)]
        inner: Pooled<T, Queue<ItemHandle<T>, S>, BoxedBuffer<DataStorage<T>>, HeapStorage>,
    }

    #[allow(private_bounds)]
    impl<T> PooledQueue<T, Auto> {
        /// Constructs a new `PooledQueue` with capacity `size` and slot type `Auto`
        pub fn new(size: usize) -> Self {
            Self::with_slot::<Auto>(size)
        }

        /// Constructs a new `PooledQueue` with capacity `size` and slot type `S`
        pub fn with_slot<S>(size: usize) -> PooledQueue<T, S>
        where
            S: SlotType<ItemHandle<T>>,
        {
            PooledQueue {
                inner: Pooled::new_from(
                    Queue::with_slot(size),
                    BoxedBuffer::new(size),
                    HeapStorage::new(size),
                ),
            }
        }

        // TODO maybe export these publicly?

        #[cfg(all(test, not(loom), not(shuttle)))]
        #[allow(unused)]
        /// Constructs a new `PooledQueue` with capacity `size`, arena size `arena_size` and slot type `Auto`
        pub(crate) fn new_with_arena_size(size: usize, arena_size: usize) -> Self {
            Self::with_slot_and_arena_size::<Auto>(size, arena_size)
        }

        #[cfg(all(test, not(loom), not(shuttle)))]
        #[allow(unused)]
        /// Constructs a new `PooledQueue` with capacity `size`, arena size `arena_size` and slot type `S`
        pub(crate) fn with_slot_and_arena_size<S>(
            size: usize,
            arena_size: usize,
        ) -> PooledQueue<T, S>
        where
            S: SlotType<ItemHandle<T>>,
        {
            PooledQueue {
                inner: Pooled::new_from(
                    Queue::with_slot(size),
                    BoxedBuffer::new(arena_size),
                    HeapStorage::new(arena_size),
                ),
            }
        }
    }

    impl<T, S> MPMCQueue for PooledQueue<T, S>
    where
        S: SlotType<ItemHandle<T>>,
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
}

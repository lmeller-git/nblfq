use mpmc_resize::{BoundedCollection, Resizable};

use crate::{
    MPMCQueue,
    Resize,
    core::{
        queue::QueueCore,
        slots::{Auto, SlotType},
    },
    owned::buffer::BoxedBuffer,
    pool::{DataStorage, IndexStorage, ItemHandle, Pooled},
};

#[allow(type_alias_bounds)]
type PooledBoxed<T, S>
where
    S: SlotType<ItemHandle<T>>,
= Pooled<
    T,
    QueueCore<BoxedBuffer<S::Slot>>,
    BoxedBuffer<DataStorage<T>>,
    QueueCore<BoxedBuffer<<Auto as SlotType<IndexStorage>>::Slot>>,
>;

/// A dynamically resizeable, pooled [`MPMCQueue`].
///
/// Unlike [`crate::growable::DynamicQueue`], this queue may store any type, at the cost of higher runtime and higher memory.
///
/// Only available on feature `dynamic` + `pool`.
///
/// For more info refer to [`crate::growable::DynamicQueue`].
#[allow(private_bounds)]
pub struct PooledDynamicQueue<T, S = Auto>
where
    S: SlotType<ItemHandle<T>>,
{
    inner: Resizable<PooledBoxed<T, S>>,
}

impl<T> PooledDynamicQueue<T, Auto> {
    /// Constructs a new `PooledDynamicQueue` with capacity `size` and slot type `Auto`.
    #[track_caller]
    pub fn new(size: usize) -> Self {
        Self::with_slot::<Auto>(size)
    }

    #[allow(private_bounds)]
    /// Constructs a new `Queue` with capacity `size` and slot type `S`.
    #[track_caller]
    pub fn with_slot<S>(size: usize) -> PooledDynamicQueue<T, S>
    where
        S: SlotType<ItemHandle<T>>,
    {
        PooledDynamicQueue {
            inner: Resizable::with_capacity(size),
        }
    }
}

impl<T, S> MPMCQueue for PooledDynamicQueue<T, S>
where
    S: SlotType<ItemHandle<T>>,
{
    type Item = T;

    fn push(&self, item: Self::Item) -> Result<(), Self::Item> {
        self.inner.try_push(item)
    }

    /// This method may block on stalling pushes under concurrent resizes.
    ///
    /// For more info refer to the trait-level docs of `MPMCQueue`.
    fn pop(&self) -> Option<Self::Item> {
        self.inner.try_pop()
    }

    fn len(&self) -> usize {
        MPMCQueue::len(&self.inner)
    }

    fn capacity(&self) -> usize {
        MPMCQueue::capacity(&self.inner)
    }

    fn is_empty(&self) -> bool {
        MPMCQueue::is_empty(&self.inner)
    }

    fn is_full(&self) -> bool {
        MPMCQueue::is_full(&self.inner)
    }
}

impl<T, S> Resize for PooledDynamicQueue<T, S>
where
    S: SlotType<ItemHandle<T>>,
    Resizable<PooledBoxed<T, S>>: Resize,
{
    fn resize(&self, size: usize) -> bool {
        self.inner.resize(size)
    }
}

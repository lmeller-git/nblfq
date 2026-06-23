use crate::{
    MPMCQueue,
    Resize,
    core::{
        queue::QueueCore,
        slots::{Auto, SlotType},
    },
    growable::{GrowableQueueCore, NewSized},
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

/// A dynamically resizeable, pooled `MPMCQueue`.
///
/// Unlike `DynamicQueue`, this queue may store any type, at the cost of higher runtime and higher memory.
///
/// Only available on feature `dynamic` + `pool`
#[allow(private_bounds)]
pub struct PooledDynamicQueue<T, S = Auto>
where
    S: SlotType<ItemHandle<T>>,
{
    inner: GrowableQueueCore<T, PooledBoxed<T, S>, S>,
}

impl<T> PooledDynamicQueue<T, Auto> {
    /// Constructs a new `PooledDynamicQueue` with capacity `size` and slot type `Auto`.
    pub fn new(size: usize) -> Self {
        Self::with_slot::<Auto>(size)
    }

    #[allow(private_bounds)]
    /// Constructs a new `Queue` with capacity `size` and slot type `S`.
    pub fn with_slot<S>(size: usize) -> PooledDynamicQueue<T, S>
    where
        S: SlotType<ItemHandle<T>>,
    {
        PooledDynamicQueue {
            inner: GrowableQueueCore::with_size(size),
        }
    }
}

impl<T, S> MPMCQueue for PooledDynamicQueue<T, S>
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

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn is_full(&self) -> bool {
        self.inner.is_full()
    }
}

impl<T, S> Resize for PooledDynamicQueue<T, S>
where
    S: SlotType<ItemHandle<T>>,
    GrowableQueueCore<T, PooledBoxed<T, S>, S>: Resize,
{
    fn resize(&self, size: usize) -> bool {
        self.inner.resize(size)
    }
}

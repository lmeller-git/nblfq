use mpmc_resize::{BoundedCollection, Resizable};

use crate::{
    MPMCQueue,
    Resize,
    core::{
        AsPackedValue,
        queue::QueueCore,
        slots::{Auto, SlotType},
    },
    growable::NewSized,
    owned::buffer::BoxedBuffer,
};

impl<S> NewSized for QueueCore<BoxedBuffer<S>>
where
    S: Default,
{
    #[track_caller]
    fn with_size(size: usize) -> Self {
        Self::new_in(BoxedBuffer::new(size))
    }
}

/// A dynamically sized concurrent queue.
///
/// During an ongoing `resize` operation, the ordering of this queue degrades from strict FIFO ordering to `k-FIFO` ordering where `k` is the number of concurrent calls to pop.
/// `linearizability` is guaranteed in any case.
///
/// For more info consult `mpmc_resize::Resizable`.
pub struct DynamicQueue<T, S = Auto>
where
    S: SlotType<T>,
    T: AsPackedValue,
{
    inner: Resizable<QueueCore<BoxedBuffer<S::Slot>>>,
}

impl<T> DynamicQueue<T, Auto>
where
    T: AsPackedValue,
{
    /// Constructs a new `DynamicQueue` with capacity `size` and slot type `Auto`.
    /// `T` must fit into the chosen slot type
    #[track_caller]
    pub fn new(size: usize) -> Self {
        Self::with_slot::<Auto>(size)
    }

    /// Constructs a new `DynamicQueue` with capacity `size` and slot type `S`.
    /// `T` must fit into the slot type `S`
    #[track_caller]
    pub fn with_slot<S>(size: usize) -> DynamicQueue<T, S>
    where
        S: SlotType<T>,
    {
        DynamicQueue {
            inner: Resizable::with_capacity(size),
        }
    }
}

impl<T, S> MPMCQueue for DynamicQueue<T, S>
where
    T: AsPackedValue,
    S: SlotType<T>,
{
    type Item = T;

    fn push(&self, item: Self::Item) -> Result<(), Self::Item> {
        self.inner.try_push(item)
    }

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

impl<T, S> Resize for DynamicQueue<T, S>
where
    T: AsPackedValue,
    S: SlotType<T>,
{
    fn resize(&self, size: usize) -> bool {
        self.inner.resize(size)
    }
}

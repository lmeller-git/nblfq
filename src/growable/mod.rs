#[cfg(feature = "pool")]
mod pooled;
mod queue;

use mpmc_resize::{BoundedCollection, Resizable};
#[cfg(feature = "pool")]
pub use pooled::*;
pub use queue::*;

use crate::{
    MPMCQueue,
    Resize,
    core::{buffer::Buffer, queue::QueueCore, slot::Slot},
    owned::buffer::BoxedBuffer,
};

pub(crate) trait NewSized {
    fn with_size(size: usize) -> Self;
}

impl<T> NewSized for BoxedBuffer<T>
where
    T: Default,
{
    #[track_caller]
    fn with_size(size: usize) -> Self {
        Self::new(size)
    }
}

impl<B> BoundedCollection for QueueCore<B>
where
    B: Buffer + NewSized,
    B::Slot: Slot,
{
    type Item = <B::Slot as Slot>::Item;

    fn with_capacity(capacity: usize) -> Self {
        Self::new_in(B::with_size(capacity))
    }

    fn try_push(&self, item: Self::Item) -> Result<(), Self::Item> {
        self.push(item)
    }

    fn try_pop(&self) -> Option<Self::Item> {
        self.pop()
    }

    fn len(&self) -> usize {
        MPMCQueue::len(self)
    }

    fn capacity(&self) -> usize {
        MPMCQueue::capacity(self)
    }

    fn is_empty(&self) -> bool {
        MPMCQueue::is_empty(self)
    }

    fn is_full(&self) -> bool {
        MPMCQueue::is_full(self)
    }
}

impl<T> Resize for Resizable<T>
where
    T: BoundedCollection + MPMCQueue,
{
    fn resize(&self, size: usize) -> bool {
        Resizable::resize(self, size)
    }
}

impl<T> MPMCQueue for Resizable<T>
where
    T: BoundedCollection + MPMCQueue,
{
    type Item = <T as BoundedCollection>::Item;

    fn push(&self, item: Self::Item) -> Result<(), Self::Item> {
        self.try_push(item)
    }

    fn pop(&self) -> Option<Self::Item> {
        self.try_pop()
    }

    fn len(&self) -> usize {
        BoundedCollection::len(self)
    }

    fn capacity(&self) -> usize {
        BoundedCollection::capacity(self)
    }

    fn is_empty(&self) -> bool {
        BoundedCollection::is_empty(self)
    }

    fn is_full(&self) -> bool {
        BoundedCollection::is_full(self)
    }
}

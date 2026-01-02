use crate::{
    Auto, MPMCQueue, SlotType,
    core::slot::PtrLike,
    owned::buffer::BoxedBuffer,
    queue::{ForcePushQueue, QueueCore},
};

pub struct Queue<T, S = Auto>
where
    T: PtrLike,
    S: SlotType<T>,
{
    inner: QueueCore<BoxedBuffer<S::Slot>>,
}

impl<T, S> Queue<T, S>
where
    T: PtrLike,
    S: SlotType<T>,
{
    pub fn new(size: usize) -> Self {
        Self {
            inner: QueueCore::new_in(BoxedBuffer::new(size)),
        }
    }
}

impl<T, S> MPMCQueue for Queue<T, S>
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

impl<T, S> ForcePushQueue for Queue<T, S>
where
    T: PtrLike,
    S: SlotType<T>,
{
}

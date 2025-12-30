use crate::{
    MPMCQueue,
    core::slot::{PtrLike, Slot},
    owned::buffer::BoxedBuffer,
    queue::{ForcePushQueue, QueueCore},
};

pub struct Queue<S: Slot> {
    inner: QueueCore<BoxedBuffer<S>>,
}

impl<S: Slot> Queue<S> {
    pub fn new(size: usize) -> Self {
        Self {
            inner: QueueCore::new_in(BoxedBuffer::new(size)),
        }
    }
}

impl<S: Slot> MPMCQueue for Queue<S> {
    type Item = S::Item;

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

impl<S: Slot> ForcePushQueue for Queue<S> where S::Item: PtrLike {}

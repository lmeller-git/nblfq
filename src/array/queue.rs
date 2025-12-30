use crate::{
    MPMCQueue,
    array::buffer::ArrayBuf,
    core::slot::{PtrLike, Slot},
    queue::{ForcePushQueue, QueueCore},
};

pub struct StaticQueue<const N: usize, S: Slot> {
    inner: QueueCore<ArrayBuf<N, S>>,
}

impl<const N: usize, S: Slot> StaticQueue<N, S> {
    pub fn new() -> Self {
        Self {
            inner: QueueCore::new_in(ArrayBuf::new()),
        }
    }
}

impl<const N: usize, S: Slot> MPMCQueue for StaticQueue<N, S> {
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

impl<const N: usize, S: Slot> ForcePushQueue for StaticQueue<N, S> where S::Item: PtrLike {}

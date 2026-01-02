pub(crate) mod buffer;
mod queue;

pub use queue::StaticQueue;

use crate::{
    ForcePushQueue, MPMCQueue,
    array::buffer::ArrayBuf,
    pool::{DataStorage, IndexStorage, ItemHandle, Pooled},
};

type StaticPooledQueue_<T, const N: usize> = Pooled<
    T,
    StaticQueue<ItemHandle<T>, N>,
    ArrayBuf<N, DataStorage<T>>,
    StaticQueue<IndexStorage, N>,
>;
pub struct StaticPooledQueue<T, const N: usize> {
    inner: StaticPooledQueue_<T, N>,
}

impl<T, const N: usize> StaticPooledQueue<T, N> {
    pub fn new() -> Self {
        Self {
            inner: Pooled::new_from(StaticQueue::new(), ArrayBuf::new(), StaticQueue::new()),
        }
    }
}

impl<T, const N: usize> MPMCQueue for StaticPooledQueue<T, N> {
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

impl<T, const N: usize> ForcePushQueue for StaticPooledQueue<T, N> {}

use crate::{
    Auto, MPMCQueue, SlotType,
    array::buffer::ArrayBuf,
    core::slot::PtrLike,
    queue::{ForcePushQueue, QueueCore},
};

pub struct StaticQueue<T, const N: usize, S = Auto>
where
    T: PtrLike,
    S: SlotType<T>,
{
    inner: QueueCore<ArrayBuf<N, S::Slot>>,
}

impl<T, const N: usize, S> StaticQueue<T, N, S>
where
    T: PtrLike,
    S: SlotType<T>,
{
    pub fn new() -> Self {
        Self {
            inner: QueueCore::new_in(ArrayBuf::new()),
        }
    }
}

impl<T, const N: usize, S> MPMCQueue for StaticQueue<T, N, S>
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

impl<T, const N: usize, S> ForcePushQueue for StaticQueue<T, N, S>
where
    T: PtrLike,
    S: SlotType<T>,
{
}

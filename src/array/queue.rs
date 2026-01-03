use crate::{
    MPMCQueue,
    array::buffer::ArrayBuf,
    core::{
        PtrLike,
        queue::QueueCore,
        slots::{Auto, SlotType},
    },
};

#[cfg(feature = "pool")]
pub use pooled_static::*;

pub struct StaticQueue<T, const N: usize, S = Auto>
where
    T: PtrLike,
    S: SlotType<T>,
{
    inner: QueueCore<ArrayBuf<N, S::Slot>>,
}

impl<T, const N: usize> StaticQueue<T, N, Auto>
where
    T: PtrLike,
{
    pub fn new() -> Self {
        Self::with_slot::<Auto>()
    }

    pub fn with_slot<S>() -> StaticQueue<T, N, S>
    where
        S: SlotType<T>,
    {
        StaticQueue {
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

#[cfg(feature = "pool")]
mod pooled_static {
    use crate::pool::{DataStorage, IndexStorage, ItemHandle, Pooled};

    use super::*;

    #[allow(private_bounds)]
    pub struct StaticPooledQueue<T, const N: usize, S = Auto>
    where
        S: SlotType<ItemHandle<T>>,
    {
        inner: Pooled<
            T,
            StaticQueue<ItemHandle<T>, N, S>,
            ArrayBuf<N, DataStorage<T>>,
            StaticQueue<IndexStorage, N>,
        >,
    }

    #[allow(private_bounds)]
    impl<T, const N: usize> StaticPooledQueue<T, N, Auto> {
        pub fn new() -> Self {
            Self::with_slot::<Auto>()
        }

        pub fn with_slot<S>() -> StaticPooledQueue<T, N, S>
        where
            S: SlotType<ItemHandle<T>>,
        {
            StaticPooledQueue {
                inner: Pooled::new_from(
                    StaticQueue::with_slot(),
                    ArrayBuf::new(),
                    StaticQueue::with_slot(),
                ),
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
}

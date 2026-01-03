use crate::{
    ForcePushQueue, MPMCQueue,
    core::{queue::QueueCore, slot::PtrLike},
    owned::buffer::BoxedBuffer,
    pool::{DataStorage, IndexStorage, ItemHandle, Pooled},
    slots::{Auto, SlotType},
};

pub struct Queue<T, S = Auto>
where
    T: PtrLike,
    S: SlotType<T>,
{
    inner: QueueCore<BoxedBuffer<S::Slot>>,
}

impl<T> Queue<T, Auto>
where
    T: PtrLike,
{
    pub fn new(size: usize) -> Self {
        Self::with_slot::<Auto>(size)
    }

    pub fn with_slot<S>(size: usize) -> Queue<T, S>
    where
        S: SlotType<T>,
    {
        Queue {
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

#[allow(private_bounds)]
pub struct PooledQueue<T, S = Auto>
where
    S: SlotType<ItemHandle<T>>,
{
    inner: Pooled<T, Queue<ItemHandle<T>, S>, BoxedBuffer<DataStorage<T>>, Queue<IndexStorage>>,
}

#[allow(private_bounds)]
impl<T> PooledQueue<T, Auto> {
    pub fn new(size: usize) -> Self {
        Self::with_slot::<Auto>(size)
    }

    pub fn with_slot<S>(size: usize) -> PooledQueue<T, S>
    where
        S: SlotType<ItemHandle<T>>,
    {
        PooledQueue {
            inner: Pooled::new_from(
                Queue::with_slot(size),
                BoxedBuffer::new(size),
                Queue::with_slot(size),
            ),
        }
    }
}

impl<T, S> MPMCQueue for PooledQueue<T, S>
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
}

impl<T, S> ForcePushQueue for PooledQueue<T, S> where S: SlotType<ItemHandle<T>> {}

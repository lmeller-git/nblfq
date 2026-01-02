pub(crate) mod buffer;
mod queue;
pub use queue::Queue;

use crate::{
    Auto, ForcePushQueue, MPMCQueue, SlotType,
    pool::{DataStorage, IndexStorage, ItemHandle, Pooled},
    slot::PtrLike,
};
use core::ptr::NonNull;

unsafe impl<T> PtrLike for alloc::boxed::Box<T> {
    type Item = T;
    fn as_ptr(zelf: Self) -> NonNull<Self::Item> {
        NonNull::new(alloc::boxed::Box::into_raw(zelf)).unwrap()
    }

    fn from_raw(raw: NonNull<Self::Item>) -> Self {
        unsafe { alloc::boxed::Box::from_raw(raw.as_ptr()) }
    }
}

#[allow(private_bounds)]
pub struct PooledQueue<T, S = Auto>
where
    S: SlotType<ItemHandle<T>>,
{
    inner: Pooled<
        T,
        Queue<ItemHandle<T>, S>,
        buffer::BoxedBuffer<DataStorage<T>>,
        Queue<IndexStorage>,
    >,
}

#[allow(private_bounds)]
impl<T, S> PooledQueue<T, S>
where
    S: SlotType<ItemHandle<T>>,
{
    pub fn new(size: usize) -> Self {
        Self {
            inner: Pooled::new_from(
                Queue::new(size),
                buffer::BoxedBuffer::new(size),
                Queue::new(size),
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

use core::{
    cell::UnsafeCell,
    fmt::Debug,
    marker::PhantomData,
    ops::{Add, Sub},
    ptr::NonNull,
};

use crate::{
    ForcePushQueue, MPMCQueue,
    core::{buffer::Buffer, slot::PtrLike},
};

pub(crate) type IndexStorage = ItemHandle<()>;
pub(crate) type DataStorage<T> = UnsafeCell<Option<T>>;

struct Pool<T, DataBuf, Q> {
    data: DataBuf,
    free_slots: Q,
    _phantom: PhantomData<T>,
}

impl<T, DataBuf, Q> Pool<T, DataBuf, Q>
where
    Q: MPMCQueue<Item = IndexStorage>,
{
    fn new(data_buf: DataBuf, index_queue: Q) -> Self {
        let cap = index_queue.capacity();
        for i in 0..cap {
            _ = index_queue.push(ItemHandle::new(i + 1));
        }

        Self {
            data: data_buf,
            free_slots: index_queue,
            _phantom: PhantomData,
        }
    }
}

impl<T, DataBuf, Q> Pool<T, DataBuf, Q>
where
    DataBuf: Buffer<Slot = DataStorage<T>>,
    Q: MPMCQueue<Item = IndexStorage>,
{
    fn allocate(&self, item: T) -> Result<usize, T> {
        let next_free = self.free_slots.pop();
        if next_free.is_none() {
            return Err(item);
        }
        let next_free = next_free.unwrap().idx;
        // idx points to slot + 1
        let cell = self
            .data
            .inner()
            .get(next_free - 1)
            .expect("popped an invalid index from self.free_slots. This is a bug.");
        unsafe { &mut *cell.get() }.replace(item);
        let next_free = next_free;
        Ok(next_free)
    }

    fn deallocate(&self, idx: usize) -> Option<T> {
        let idx = idx;
        // idx points to slot + 1
        let slot = self.data.inner().get(idx - 1)?;
        let cell = unsafe { &mut *slot.get() };
        let item = cell.take();
        _ = self.free_slots.push(ItemHandle::new(idx));
        item
    }
}

unsafe impl<T, DataBuf, Q> Send for Pool<T, DataBuf, Q>
where
    DataBuf: Buffer<Slot = DataStorage<T>>,
    Q: MPMCQueue<Item = IndexStorage>,
    T: Send,
{
}
unsafe impl<T, DataBuf, Q> Sync for Pool<T, DataBuf, Q>
where
    DataBuf: Buffer<Slot = DataStorage<T>>,
    Q: MPMCQueue<Item = IndexStorage>,
    T: Sync,
{
}

pub(crate) struct ItemHandle<T> {
    idx: usize,
    _phantom: PhantomData<T>,
}

impl<T> ItemHandle<T> {
    fn new(idx: usize) -> Self {
        Self {
            idx,
            _phantom: PhantomData,
        }
    }
}

impl<T> Sub<usize> for ItemHandle<T> {
    type Output = Self;

    fn sub(mut self, rhs: usize) -> Self::Output {
        self.idx -= rhs;
        self
    }
}

impl<T> Add<usize> for ItemHandle<T> {
    type Output = Self;

    fn add(mut self, rhs: usize) -> Self::Output {
        self.idx += rhs;
        self
    }
}

unsafe impl<T> PtrLike for ItemHandle<T> {
    type Item = T;
    fn as_ptr(zelf: Self) -> NonNull<Self::Item> {
        NonNull::new(zelf.idx as *mut Self::Item).unwrap()
    }

    fn from_raw(raw: NonNull<Self::Item>) -> Self {
        Self::new(raw.as_ptr() as usize)
    }
}

impl<T> Default for ItemHandle<T> {
    fn default() -> Self {
        Self::new(usize::default())
    }
}

impl<T> Debug for ItemHandle<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ItemHandle")
            .field("index", &format_args!("{:?}", self.idx))
            .finish()
    }
}

// these should be autoderived anyways
// unsafe impl<T, I> Send for ItemHandle<T, I> where I: Send {}
// unsafe impl<T, I> Sync for ItemHandle<T, I> where I: Sync {}

pub(crate) struct Pooled<T, Q, DataBuf, IndexQ> {
    q: Q,
    pool: Pool<T, DataBuf, IndexQ>,
}

impl<T, Q, DataBuf, IndexQ> Pooled<T, Q, DataBuf, IndexQ>
where
    IndexQ: MPMCQueue<Item = IndexStorage>,
{
    pub(crate) fn new_from(queue: Q, data_buf: DataBuf, idx_buf: IndexQ) -> Self {
        Self {
            q: queue,
            pool: Pool::new(data_buf, idx_buf),
        }
    }
}

impl<T, Q, DataBuf, IndexQ> MPMCQueue for Pooled<T, Q, DataBuf, IndexQ>
where
    Q: MPMCQueue<Item = ItemHandle<T>>,
    DataBuf: Buffer<Slot = DataStorage<T>>,
    IndexQ: MPMCQueue<Item = IndexStorage>,
{
    type Item = T;

    fn push(&self, item: Self::Item) -> Result<(), Self::Item> {
        let idx = self.pool.allocate(item)?;
        let handle = ItemHandle::new(idx);
        // this could fail if cap of pool > cap of queue
        self.q.push(handle).map_err(|handle| {
            self.pool
                .deallocate(handle.idx)
                .expect("Wrong index handed to Pool::dellocate. This is a bug.")
        })
    }

    fn pop(&self) -> Option<Self::Item> {
        let handle = self.q.pop()?;
        Some(self.pool.deallocate(handle.idx).unwrap())
    }

    fn len(&self) -> usize {
        self.q.len()
    }

    fn capacity(&self) -> usize {
        self.q.capacity()
    }
}

// could reuse the allocation of a popped item here instead of reallocating
impl<T, Q, DataBuf, IndexQ> ForcePushQueue for Pooled<T, Q, DataBuf, IndexQ>
where
    Q: MPMCQueue<Item = ItemHandle<T>>,
    DataBuf: Buffer<Slot = DataStorage<T>>,
    IndexQ: MPMCQueue<Item = IndexStorage>,
{
}

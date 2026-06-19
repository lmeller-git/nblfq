use core::{fmt::Debug, marker::PhantomData};

use crate::{
    MPMCQueue,
    core::{AsPackedValue, TruncatedU64, buffer::Buffer},
    sync::cell::UnsafeCell,
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
            _ = index_queue.push(ItemHandle::new(OwnedIdx::new(i)));
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
    fn allocate(&self, item: T) -> Result<OwnedIdx, T> {
        let next_free = self.free_slots.pop();
        if next_free.is_none() {
            return Err(item);
        }
        let next_free = next_free.unwrap().idx;
        let cell = self
            .data
            .inner()
            .get(next_free.idx)
            .expect("popped an invalid index from self.free_slots. This is a bug.");
        // SAFETY:
        // Each index in the Index queue is unique exists as only one instance. If we own this Index, no other thread has it
        cell.with_mut(|c| unsafe { &mut *c }.replace(item));
        Ok(next_free)
    }

    fn deallocate(&self, idx: OwnedIdx) -> Option<T> {
        let slot = self.data.inner().get(idx.idx)?;
        // SAFETY:
        // Each index in the Index queue is unique exists as only one instance. If we own this Index, no other thread has it
        let item = slot.with_mut(|c| unsafe { &mut *c }.take());
        _ = self.free_slots.push(ItemHandle::new(idx));
        item
    }
}

// SAFETY:
// Pool stores items of type T.
// It uses a MPMCQueue to ensure thread-safety
unsafe impl<T, DataBuf, Q> Send for Pool<T, DataBuf, Q>
where
    DataBuf: Buffer<Slot = DataStorage<T>>,
    Q: MPMCQueue<Item = IndexStorage>,
    T: Send,
{
}
// SAFETY:
// Pool stores items of type T.
// It uses a MPMCQueue to ensure thread-safety
unsafe impl<T, DataBuf, Q> Sync for Pool<T, DataBuf, Q>
where
    DataBuf: Buffer<Slot = DataStorage<T>>,
    Q: MPMCQueue<Item = IndexStorage>,
    T: Sync,
{
}

/// An owned !Copy !Clone version of a usize index
#[derive(Debug)]
struct OwnedIdx {
    idx: usize,
}

impl OwnedIdx {
    fn new(idx: usize) -> Self {
        Self { idx }
    }
}

#[derive(Debug)]
pub(crate) struct ItemHandle<T> {
    idx: OwnedIdx,
    _phantom: PhantomData<T>,
}

impl<T> ItemHandle<T> {
    fn new(idx: OwnedIdx) -> Self {
        Self {
            idx,
            _phantom: PhantomData,
        }
    }

    fn idx(&self) -> usize {
        self.idx.idx
    }
}

// SAFETY:
// the caller must ensure that:
// - the index stored in ItemHandle<T> uses at most 48 bits, if stored in a TaggedPtr64
unsafe impl<T> AsPackedValue for ItemHandle<T> {
    const MIN_BIT_WIDTH: usize = 48;

    fn encode(zelf: Self) -> crate::core::TruncatedU64<Self> {
        debug_assert!(
            zelf.idx.idx <= 2_usize.pow(48),
            "Used an ItemHandle with an incompatible index. This either means misuse of the API,
            or that the capacity of the used pool is too high. The capacity of the pool should not exceed 2^48."
        );
        TruncatedU64::new(zelf.idx() as u64)
    }

    unsafe fn decode(raw: crate::core::TruncatedU64<Self>) -> Self {
        Self::new(OwnedIdx::new(raw.read() as usize))
    }

    fn is_rt_safe() -> bool {
        true
    }
}

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

// TODO could reuse the allocation of a popped item in force_push instead of reallocating
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
        // In practice this will never happen, as the cap of the pool == the cap of the queue, if constructed via the public API
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

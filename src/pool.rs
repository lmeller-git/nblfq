use core::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use crate::{
    MPMCQueue,
    core::{AsPackedValue, NonZeroTruncatedU64, buffer::Buffer},
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
            _ = index_queue.push(ItemHandle::new(OwnedIdx::new(i + 1)));
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
        // idx points to slot + 1
        let cell = self
            .data
            .inner()
            .get(next_free.idx - 1)
            .expect("popped an invalid index from self.free_slots. This is a bug.");
        // SAFETY:
        // The caller has to guarantee that each index is unique, i.e. that only one thread may own an index at a time.
        cell.with_mut(|c| unsafe { &mut *c }.replace(item));
        Ok(next_free)
    }

    fn deallocate(&self, idx: OwnedIdx) -> Option<T> {
        // idx points to slot + 1
        let slot = self.data.inner().get(idx.idx - 1)?;
        // SAFETY:
        // The caller has to guarantee that each index is unique, i.e. that only one thread may own an index at a time.
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

#[derive(Debug)]
struct OwnedIdx {
    idx: usize,
    _phantom: PhantomData<[()]>, // [()] is !Copy
}

impl OwnedIdx {
    fn new(idx: usize) -> Self {
        Self {
            idx,
            _phantom: PhantomData,
        }
    }
}

impl Add<usize> for OwnedIdx {
    type Output = Self;

    fn add(mut self, rhs: usize) -> Self::Output {
        self.idx += rhs;
        self
    }
}

impl AddAssign<usize> for OwnedIdx {
    fn add_assign(&mut self, rhs: usize) {
        self.idx += rhs
    }
}

impl Sub<usize> for OwnedIdx {
    type Output = Self;

    fn sub(mut self, rhs: usize) -> Self::Output {
        self.idx -= rhs;
        self
    }
}

impl SubAssign<usize> for OwnedIdx {
    fn sub_assign(&mut self, rhs: usize) {
        self.idx -= rhs
    }
}

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

impl<T> Debug for ItemHandle<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ItemHandlt")
            .field("idx", &format_args!("{:?}", self.idx))
            .finish()
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

// SAFETY:
// the caller must ensure that:
// - the index stored in ItemHandle<T> is never 0
// - the index stored in ItemHandle<T> uses at most 48 bits, if stored in a TaggedPtr64
unsafe impl<T> AsPackedValue for ItemHandle<T> {
    const MIN_BIT_WIDTH: usize = 48;

    fn encode(zelf: Self) -> crate::core::NonZeroTruncatedU64<Self> {
        NonZeroTruncatedU64::new(zelf.idx() as u64).unwrap()
    }

    unsafe fn decode(raw: crate::core::NonZeroTruncatedU64<Self>) -> Self {
        Self::new(OwnedIdx::new(raw.read() as usize))
    }
}

impl<T> Default for ItemHandle<T> {
    fn default() -> Self {
        Self::new(OwnedIdx::new(usize::default()))
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

use core::{fmt::Debug, marker::PhantomData};

use lf_slots::{SlotPoolMeta, core::RawSlotPool};

use crate::{
    MPMCQueue,
    core::{AsPackedValue, TruncatedU64, buffer::Buffer},
    sync::cell::UnsafeCell,
};

pub(crate) type DataStorage<T> = UnsafeCell<Option<T>>;

struct Pool<T, DataBuf, S> {
    data: DataBuf,
    free_slots: S,
    _phantom: PhantomData<T>,
}

impl<T, DataBuf, S> Pool<T, DataBuf, S>
where
    DataBuf: Buffer<Slot = DataStorage<T>>,
    S: SlotPoolMeta,
{
    #[track_caller]
    fn new(data_buf: DataBuf, slot_storage: S) -> Self {
        debug_assert!(
            slot_storage.capacity() >= data_buf.capacity(),
            "Slot storage capacity ({}) must be >= data buffer capacity ({})",
            slot_storage.capacity(),
            data_buf.capacity()
        );

        Self {
            data: data_buf,
            free_slots: slot_storage,
            _phantom: PhantomData,
        }
    }
}

impl<T, DataBuf, S> Pool<T, DataBuf, S>
where
    DataBuf: Buffer<Slot = DataStorage<T>>,
    S: RawSlotPool,
{
    fn allocate(&self, item: T) -> Result<OwnedIdx, T> {
        let Some(idx) = self.free_slots.pull_raw() else {
            return Err(item);
        };

        let cell = self
            .data
            .inner()
            .get(idx)
            .expect("Popped an invalid index from self.free_slots. This is a bug.");

        // SAFETY:
        // Each index in the slot storage is unique and returned to at most one caller at a time.
        // If we own this index, no other thread is concurrently writing to or reading from this cell.
        cell.with_mut(|c| unsafe { &mut *c }.replace(item));
        Ok(OwnedIdx::new(idx))
    }

    fn deallocate(&self, idx: OwnedIdx) -> Option<T> {
        let slot = self.data.inner().get(idx.idx)?;

        // SAFETY:
        // Exclusive access to this slot index is guaranteed by owning `idx`.
        let item = slot.with_mut(|c| unsafe { &mut *c }.take());

        // SAFETY:
        // `idx.idx` was originally produced by `pull_raw` on this pool's storage instance.
        unsafe {
            self.free_slots.put_raw(idx.idx);
        }

        item
    }
}

// SAFETY:
// Pool manages items of type T and delegates thread-safe allocation to S: RawSlotPool + Sync.
unsafe impl<T, DataBuf, S> Send for Pool<T, DataBuf, S>
where
    DataBuf: Buffer<Slot = DataStorage<T>>,
    S: RawSlotPool + Sync,
    T: Send,
{
}

// SAFETY:
// Pool manages items of type T and delegates thread-safe allocation to S: RawSlotPool + Sync.
unsafe impl<T, DataBuf, S> Sync for Pool<T, DataBuf, S>
where
    DataBuf: Buffer<Slot = DataStorage<T>>,
    S: RawSlotPool + Sync,
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
// The caller must ensure that index stored in ItemHandle<T> uses at most 48 bits.
unsafe impl<T> AsPackedValue for ItemHandle<T> {
    const MIN_BIT_WIDTH: usize = 48;

    fn encode(zelf: Self) -> TruncatedU64<Self> {
        debug_assert!(
            zelf.idx.idx <= (1_usize << 48),
            "Used an ItemHandle with an incompatible index exceeding 2^48."
        );
        TruncatedU64::new(zelf.idx() as u64)
    }

    unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
        Self::new(OwnedIdx::new(raw.read() as usize))
    }

    fn is_rt_safe() -> bool {
        true
    }
}

pub(crate) struct Pooled<T, Q, DataBuf, S> {
    q: Q,
    pool: Pool<T, DataBuf, S>,
}

impl<T, Q, DataBuf, S> Pooled<T, Q, DataBuf, S>
where
    DataBuf: Buffer<Slot = DataStorage<T>>,
    S: SlotPoolMeta,
{
    #[track_caller]
    pub(crate) fn new_from(queue: Q, data_buf: DataBuf, slot_storage: S) -> Self {
        Self {
            q: queue,
            pool: Pool::new(data_buf, slot_storage),
        }
    }
}

impl<T, Q, DataBuf, S> MPMCQueue for Pooled<T, Q, DataBuf, S>
where
    Q: MPMCQueue<Item = ItemHandle<T>>,
    DataBuf: Buffer<Slot = DataStorage<T>>,
    S: RawSlotPool + SlotPoolMeta,
{
    type Item = T;

    fn push(&self, item: Self::Item) -> Result<(), Self::Item> {
        let idx = self.pool.allocate(item)?;
        let handle = ItemHandle::new(idx);

        self.q.push(handle).map_err(|handle| {
            self.pool
                .deallocate(handle.idx)
                .expect("Wrong index handed to Pool::deallocate. This is a bug.")
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

#[cfg(feature = "dynamic")]
mod growable {
    use lf_slots::Slots;

    use super::*;
    use crate::growable::NewSized;

    impl<T, Q, DataBuf> NewSized for Pooled<T, Q, DataBuf, Slots>
    where
        Q: MPMCQueue<Item = ItemHandle<T>> + NewSized,
        DataBuf: Buffer<Slot = DataStorage<T>> + NewSized,
    {
        fn with_size(size: usize) -> Self {
            Self::new_from(
                Q::with_size(size),
                DataBuf::with_size(size),
                Slots::new(size),
            )
        }
    }
}

// cover is_rt_safe for ItemHandle<T>. This is not covered in any other case, since only ItemHandle<()>, which is zero sized is ever used.
#[cfg(test)]
mod tests {
    use super::ItemHandle;
    use crate::core::AsPackedValue;

    #[test]
    fn true_is_true() {
        assert!(ItemHandle::<()>::is_rt_safe());
    }
}

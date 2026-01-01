use core::{
    cell::UnsafeCell,
    marker::PhantomData,
    sync::atomic::{AtomicUsize, Ordering, fence},
};

use crate::{
    ForcePushQueue, MPMCQueue,
    array::{StaticQueue, buffer::ArrayBuf},
    buffer::Buffer,
    slot::{PtrLike, TaggedPtr64},
};

trait Idx: Copy {
    const BITS: usize;
    fn as_usize(zelf: Self) -> usize;
    fn from_usize(v: usize) -> Self;
}

impl Idx for usize {
    const BITS: usize = 64;
    fn as_usize(zelf: Self) -> usize {
        zelf
    }

    fn from_usize(v: usize) -> Self {
        v
    }
}

impl Idx for u32 {
    const BITS: usize = 32;
    fn as_usize(zelf: Self) -> usize {
        zelf as usize
    }

    fn from_usize(v: usize) -> Self {
        v as u32
    }
}

struct Pool<T, DataBuf, IndexBuf> {
    data: DataBuf,
    free_slots: IndexStack<IndexBuf>,
    _phantom: PhantomData<T>,
}

impl<T, DataBuf, IndexBuf, I> Pool<T, DataBuf, IndexBuf>
where
    IndexBuf: Buffer<Slot = UnsafeCell<Option<I>>>,
    I: Idx,
{
    fn new(data_buf: DataBuf, idx_buf: IndexBuf) -> Self {
        let cap = idx_buf.capacity();
        let stack = IndexStack {
            next_free: AtomicUsize::new(cap),
            index_list: idx_buf,
        };
        let arr = stack.index_list.inner();
        for i in 0..cap {
            let cell = unsafe { &mut *arr.get(i).unwrap().get() };
            cell.replace(I::from_usize(i));
        }
        Self {
            data: data_buf,
            free_slots: stack,
            _phantom: PhantomData,
        }
    }
}

impl<T, DataBuf, IndexBuf, I> Pool<T, DataBuf, IndexBuf>
where
    DataBuf: Buffer<Slot = UnsafeCell<Option<T>>>,
    IndexBuf: Buffer<Slot = UnsafeCell<Option<I>>>,
    I: Idx,
{
    fn allocate(&self, item: T) -> Result<I, T> {
        let next_free = self.free_slots.pop();
        if next_free.is_none() {
            return Err(item);
        }
        let next_free = next_free.unwrap();
        let cell = self
            .data
            .inner()
            .get(I::as_usize(next_free))
            .expect("popped an invalid index from self.free_slots. This is a bug.");
        unsafe { &mut *cell.get() }.replace(item);
        Ok(next_free)
    }

    fn deallocate(&self, idx: usize) -> Option<T> {
        let slot = self.data.inner().get(idx)?;
        let cell = unsafe { &mut *slot.get() };
        let item = cell.take();
        self.free_slots.push(idx);
        item
    }
}

unsafe impl<T, DataBuf, IndexBuf, I> Send for Pool<T, DataBuf, IndexBuf>
where
    DataBuf: Buffer<Slot = UnsafeCell<Option<T>>>,
    IndexBuf: Buffer<Slot = UnsafeCell<Option<I>>>, // may not need to restrict IndexBuf, as it is already restricted by IndexStack Send
    T: Send,
{
}
unsafe impl<T, DataBuf, IndexBuf, I> Sync for Pool<T, DataBuf, IndexBuf>
where
    DataBuf: Buffer<Slot = UnsafeCell<Option<T>>>,
    IndexBuf: Buffer<Slot = UnsafeCell<Option<I>>>, // may not need to restrict IndexBuf, as it is already restricted by IndexStack Sync
    T: Sync,
{
}

struct IndexStack<B> {
    index_list: B,
    next_free: AtomicUsize, // next_free - 1 is the largest idx in index_list containing a slot
}

impl<B: Buffer, I> IndexStack<B>
where
    B: Buffer<Slot = UnsafeCell<Option<I>>>,
    I: Idx,
{
    fn pop(&self) -> Option<I> {
        let mut current_head = self.next_free.load(Ordering::Acquire);
        loop {
            if current_head == 0 {
                return None;
            }
            match self.next_free.compare_exchange(
                current_head,
                current_head - 1,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(head) => {
                    let slot = self
                        .index_list
                        .inner()
                        .get(head - 1)
                        .expect("popped invalid head from stack. This is a Bug");
                    let cell = unsafe { &mut *slot.get() };
                    return cell.take();
                }
                Err(head) => current_head = head,
            }
        }
    }

    fn push(&self, idx: usize) {
        let current_head = self.next_free.fetch_add(1, Ordering::AcqRel);
        debug_assert!(current_head < self.index_list.capacity());
        let slot = self
            .index_list
            .inner()
            .get(current_head)
            .expect("popped invalid head from stack. This is a Bug");
        let cell = unsafe { &mut *slot.get() };
        cell.replace(I::from_usize(idx));
    }
}

unsafe impl<B, I> Send for IndexStack<B>
where
    B: Buffer<Slot = UnsafeCell<Option<I>>>,
    I: Send,
{
}

unsafe impl<B, I> Sync for IndexStack<B>
where
    B: Buffer<Slot = UnsafeCell<Option<I>>>,
    I: Sync,
{
}

struct ItemHandle<T, I> {
    idx: I,
    _phantom: PhantomData<T>,
}

impl<T, I> ItemHandle<T, I> {
    fn new(idx: I) -> Self {
        Self {
            idx,
            _phantom: PhantomData,
        }
    }
}

unsafe impl<T, I: Idx> PtrLike for ItemHandle<T, I> {
    type Item = T;
    fn as_ptr(zelf: Self) -> *mut Self::Item {
        Idx::as_usize(zelf.idx) as *mut Self::Item
    }

    fn from_raw(raw: *mut Self::Item) -> Option<Self> {
        Some(Self::new(I::from_usize(raw as usize)))
    }
}

impl<T, I: Default> Default for ItemHandle<T, I> {
    fn default() -> Self {
        Self::new(I::default())
    }
}

// these should be autoderived anyways
// unsafe impl<T, I> Send for ItemHandle<T, I> where I: Send {}
// unsafe impl<T, I> Sync for ItemHandle<T, I> where I: Sync {}

struct Pooled<T, Q, DataBuf, IndexBuf> {
    q: Q,
    pool: Pool<T, DataBuf, IndexBuf>,
}

impl<T, Q, DataBuf, IndexBuf, I> Pooled<T, Q, DataBuf, IndexBuf>
where
    IndexBuf: Buffer<Slot = UnsafeCell<Option<I>>>,
    I: Idx,
{
    fn new_from(queue: Q, data_buf: DataBuf, idx_buf: IndexBuf) -> Self {
        Self {
            q: queue,
            pool: Pool::new(data_buf, idx_buf),
        }
    }
}

impl<T, Q, DataBuf, IndexBuf, I> MPMCQueue for Pooled<T, Q, DataBuf, IndexBuf>
where
    Q: MPMCQueue<Item = ItemHandle<T, I>>,
    I: Idx,
    DataBuf: Buffer<Slot = UnsafeCell<Option<T>>>,
    IndexBuf: Buffer<Slot = UnsafeCell<Option<I>>>,
{
    type Item = T;

    fn push(&self, item: Self::Item) -> Result<(), Self::Item> {
        let idx = self.pool.allocate(item)?;
        fence(Ordering::Release);
        let handle = ItemHandle::new(idx);
        self.q.push(handle).map_err(|handle| {
            fence(Ordering::Acquire);
            self.pool
                .deallocate(I::as_usize(handle.idx))
                .expect("Wrong index handed to Pool::dellocate. This is a bug.")
        })
    }

    fn pop(&self) -> Option<Self::Item> {
        let handle = self.q.pop()?;
        fence(Ordering::Acquire);
        self.pool.deallocate(I::as_usize(handle.idx))
    }

    fn len(&self) -> usize {
        self.q.len()
    }

    fn capacity(&self) -> usize {
        self.q.capacity()
    }
}

impl<T, Q, DataBuf, IndexBuf, I> ForcePushQueue for Pooled<T, Q, DataBuf, IndexBuf>
where
    Q: MPMCQueue<Item = ItemHandle<T, I>>,
    I: Idx,
    DataBuf: Buffer<Slot = UnsafeCell<Option<T>>>,
    IndexBuf: Buffer<Slot = UnsafeCell<Option<I>>>,
{
}

fn foo() {
    let q: Pooled<
        usize,
        StaticQueue<10, TaggedPtr64<ItemHandle<usize, u32>>>,
        ArrayBuf<10, UnsafeCell<Option<usize>>>,
        ArrayBuf<10, UnsafeCell<Option<u32>>>,
    > = Pooled::new_from(StaticQueue::new(), ArrayBuf::new(), ArrayBuf::new());
    assert!(q.push(5).is_ok());
    assert_eq!(q.pop().unwrap(), 5);

    let q2: StaticPooledQueue_<usize, 10> = StaticPooledQueue_::new();
    q2.push(5).unwrap();
    q2.pop();
}

type StaticPooledQueue_<T, const N: usize> = Pooled<
    T,
    StaticQueue<N, TaggedPtr64<ItemHandle<T, u32>>>,
    ArrayBuf<N, UnsafeCell<Option<T>>>,
    ArrayBuf<N, UnsafeCell<Option<u32>>>,
>;

impl<T, const N: usize> StaticPooledQueue_<T, N> {
    pub fn new() -> Self {
        Self::new_from(StaticQueue::new(), ArrayBuf::new(), ArrayBuf::new())
    }
}

pub struct StaticPooledQueue<T, const N: usize>(StaticPooledQueue_<T, N>);

impl<T, const N: usize> StaticPooledQueue<T, N> {
    pub fn new() -> Self {
        Self(StaticPooledQueue_::new_from(
            StaticQueue::new(),
            ArrayBuf::new(),
            ArrayBuf::new(),
        ))
    }
}

impl<T, const N: usize> MPMCQueue for StaticPooledQueue<T, N> {
    type Item = T;
    fn push(&self, item: Self::Item) -> Result<(), Self::Item> {
        self.0.push(item)
    }

    fn pop(&self) -> Option<Self::Item> {
        self.0.pop()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn capacity(&self) -> usize {
        self.0.capacity()
    }
}

impl<T, const N: usize> ForcePushQueue for StaticPooledQueue<T, N> {
    fn force_push(&self, mut item: Self::Item) -> Option<Self::Item> {
        self.0.force_push(item)
    }
}

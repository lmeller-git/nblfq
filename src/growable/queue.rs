use alloc::boxed::Box;
use core::{marker::PhantomData, ptr::null_mut};

use crossbeam_utils::CachePadded;

use crate::{
    MPMCQueue,
    Resize,
    core::{
        AsPackedValue,
        queue::QueueCore,
        slots::{Auto, SlotType},
    },
    growable::NewSized,
    owned::buffer::BoxedBuffer,
    sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering},
    utils::Backoff,
};

/// A dynamically sized concurrent queue.
///
/// Operations on this queue are lock-free if `resize` is not actively used.
/// During an ongoing `resize` operation, `pop` and operations depending on it may block on stalled `pushes`.
pub(crate) struct GrowableQueueCore<T, Q, S = Auto> {
    cores: [AtomicPtr<Q>; 2],
    push_epoch: CachePadded<AtomicUsize>,
    pop_epoch: CachePadded<AtomicUsize>,
    active_pushes: CachePadded<[AtomicUsize; 2]>,
    active_reads: CachePadded<[AtomicUsize; 2]>,
    is_resizing: AtomicBool,
    _slot: PhantomData<(S, T)>,
}

impl<T, Q> GrowableQueueCore<T, Q, Auto>
where
    Q: NewSized,
{
    /// Constructs a new `Queue` with capacity `size` and slot type `S`.
    /// `T` must fit into the slot type `S`
    #[track_caller]
    pub(crate) fn with_slot<S>(size: usize) -> GrowableQueueCore<T, Q, S> {
        GrowableQueueCore {
            cores: [
                AtomicPtr::new(Box::into_raw(Box::new(Q::with_size(size)))),
                AtomicPtr::new(Box::into_raw(Box::new(Q::with_size(1)))),
            ],
            active_pushes: [AtomicUsize::new(0), AtomicUsize::new(0)].into(),
            active_reads: [AtomicUsize::new(0), AtomicUsize::new(0)].into(),
            push_epoch: AtomicUsize::new(0).into(),
            pop_epoch: AtomicUsize::new(0).into(),
            is_resizing: AtomicBool::new(false),
            _slot: PhantomData,
        }
    }
}

impl<T, Q, S> Drop for GrowableQueueCore<T, Q, S> {
    fn drop(&mut self) {
        let left = self.cores[0].swap(null_mut(), Ordering::Acquire);
        // Safety:
        // No concurrent drops of this ds can happen.
        // This queue was allocated in `new` or in `grow_by` with `Box::into_raw` and was not deallocated since then.
        _ = unsafe { Box::from_raw(left) };

        let right = self.cores[1].swap(null_mut(), Ordering::Acquire);
        // Safety:
        // No concurrent drops of this ds can happen.
        // This queue was allocated in `new` or in `grow_by` with `Box::into_raw` and was not deallocated since then.
        _ = unsafe { Box::from_raw(right) };
    }
}

// we need SeqCst on epoch atomics and active_* atomics on the synchronization points,
// because we have to synchronize between an asymmetric read/write - write/read pattern across the two

impl<T, Q, S> Resize for GrowableQueueCore<T, Q, S>
where
    Q: NewSized + MPMCQueue<Item = T>,
{
    fn resize(&self, size: usize) -> bool {
        if size == 0 {
            return false;
        }
        #[cfg(loom)]
        crate::sync::atomic::fence(Ordering::SeqCst);
        let push_epoch = self.push_epoch.load(Ordering::SeqCst);
        let pop_epoch = self.pop_epoch.load(Ordering::SeqCst);

        if pop_epoch != push_epoch {
            return false;
        }

        if self.active_reads[(push_epoch + 1) % 2].load(Ordering::Acquire) != 0 {
            // could happen if some thread started reading before pop_epoch got updated
            return false;
        }

        if self.is_resizing.swap(true, Ordering::AcqRel) {
            return false;
        }

        if self.push_epoch.load(Ordering::Acquire) != push_epoch {
            // could happen if an entire resize happens between load and this check
            self.is_resizing.store(false, Ordering::Release);
            return false;
        }

        // at this poitn we know that
        // a) no concurrent resize is happening
        // b) since pop_epoch == push_epoch the old queue is empty.
        // c) since pop_epoch == push_epoch AND active_reads == 0, we know that active_reads is STILL 0, becasue noone will acces the stale queue

        let old_idx = (push_epoch + 1) % 2;
        let mut backoff = Backoff::new();

        // wait on any stale readers of the old queue.
        // Note that at any point during AND after this loop other threads may still register as NEW readers on the OLD queue;
        // This is safe, because they will revalidate the epoch after registration and NOT actually read the underlying queue.
        // However this menas that we may spin for longer than strictly necessary to ensure noone is actually reading the old queue.
        while {
            crate::sync::atomic::fence(Ordering::SeqCst);
            self.active_reads[old_idx].load(Ordering::Acquire) != 0
                || self.active_pushes[old_idx].load(Ordering::Acquire) != 0
        } {
            backoff.backoff();
        }

        let new_queue = Box::into_raw(Box::new(Q::with_size(size)));

        // Safety:
        // since pop_epoch == push_epoch all concurrent threads acces the queue at push_epoch % 2.
        // pop ensures that no pushes are in flight to the old queue anymore and that it is empty. We can safely drop it.
        let old_queue = self.cores[old_idx].swap(new_queue, Ordering::AcqRel);

        self.push_epoch.fetch_add(1, Ordering::Release);

        // Safety:
        // old_queue was ocnstucted from a Box::into_raw and is dropped only once, as ensured by epoch guards
        let q = unsafe { Box::from_raw(old_queue) };

        #[cfg(not(any(loom, shuttle)))]
        debug_assert!(q.pop().is_none());
        #[cfg(any(loom, shuttle))]
        assert!(q.pop().is_none());

        self.is_resizing.store(false, Ordering::Release);
        true
    }
}

impl<T, Q, S> GrowableQueueCore<T, Q, S> {
    fn get_queue(&self, epoch: usize) -> &Q {
        let queue = self.cores[epoch % 2].load(Ordering::Acquire);
        // Safety:
        // It is guranteed by `resize` that no concurrent mutable access can happen to any queue in cores.
        // It is safe to access it concurrently via shared ref, as long as queue core is Sync.
        unsafe { &*queue }
    }

    fn register_push(&self, target_epoch: usize) -> bool {
        #[cfg(loom)]
        crate::sync::atomic::fence(Ordering::SeqCst);
        self.active_reads[target_epoch % 2].fetch_add(1, Ordering::SeqCst);

        #[cfg(loom)]
        crate::sync::atomic::fence(Ordering::SeqCst);
        let current_push = self.push_epoch.load(Ordering::SeqCst);

        // It is safe to read if the target epoch is still structurally active
        if target_epoch != current_push {
            self.deregister_reader(target_epoch);
            return false;
        }
        true
    }

    fn register_pop(&self, target_epoch: usize) -> bool {
        #[cfg(loom)]
        crate::sync::atomic::fence(Ordering::SeqCst);
        self.active_reads[target_epoch % 2].fetch_add(1, Ordering::SeqCst);

        #[cfg(loom)]
        crate::sync::atomic::fence(Ordering::SeqCst);
        let current_pop = self.pop_epoch.load(Ordering::SeqCst);

        // It is safe to read if the target epoch is still structurally active
        if target_epoch != current_pop {
            self.deregister_reader(target_epoch);
            return false;
        }
        true
    }

    fn deregister_reader(&self, epoch: usize) {
        self.active_reads[epoch % 2].fetch_sub(1, Ordering::Release);
    }
}

impl<T, Q, S> MPMCQueue for GrowableQueueCore<T, Q, S>
where
    Q: MPMCQueue<Item = T>,
{
    type Item = T;

    fn push(&self, item: Self::Item) -> Result<(), Self::Item> {
        loop {
            let push_epoch = self.push_epoch.load(Ordering::Acquire);
            self.active_pushes[push_epoch % 2].fetch_add(1, Ordering::Release);

            #[cfg(loom)]
            crate::sync::atomic::fence(Ordering::SeqCst);
            if self.push_epoch.load(Ordering::SeqCst) == push_epoch {
                let r = self.get_queue(push_epoch).push(item);

                self.active_pushes[push_epoch % 2].fetch_sub(1, Ordering::Release);
                return r;
            }
            self.active_pushes[push_epoch % 2].fetch_sub(1, Ordering::Release);
        }
    }

    /// This method may block on stalling pushes under concurrent resizes.
    ///
    /// For more info refer to the trait-level docs of `MPMCQueue`.
    fn pop(&self) -> Option<Self::Item> {
        let mut backoff = Backoff::new();
        loop {
            let push_epoch = self.push_epoch.load(Ordering::Acquire);
            let pop_epoch = self.pop_epoch.load(Ordering::Acquire);

            if pop_epoch != push_epoch {
                // drain old buffer

                if !self.register_pop(pop_epoch) {
                    continue;
                }

                // it is safe to call get_queue on pop_epoch here, since no resize can happen while we have not updated pop_epoch and reads on this epoch are happening
                let item = self.get_queue(pop_epoch).pop();

                self.deregister_reader(pop_epoch);

                if item.is_some() {
                    return item;
                }

                if self.active_pushes[pop_epoch % 2].load(Ordering::Acquire) == 0 {
                    if !self.register_pop(pop_epoch) {
                        continue;
                    }

                    let final_item = self.get_queue(pop_epoch).pop();

                    self.deregister_reader(pop_epoch);

                    if final_item.is_some() {
                        return final_item;
                    }

                    _ = self.pop_epoch.compare_exchange_weak(
                        pop_epoch,
                        pop_epoch + 1,
                        Ordering::AcqRel,
                        Ordering::Relaxed,
                    );

                    continue;
                }

                // at this point the old queue did not contain any items, even though items are in-flight. At this point the new queue may already contain items.
                // We face a tradeoff:
                //
                // a) continue in the inner loop and block on the active_pushers -> violates non-blocking/obstruction-freedom guarantees
                // b) check the new queue -> opens up the possibilty for item reordering, even in spsc scenarios, i.e. violates FIFO guarantees + linearizability
                // c) bail, even though the queue is non-empty -> violates linearizability (and emptiness assumptions) in that case.
                // Even worse: if some active_pusher is indefinitely dead, we will henceforth only bail. Thus this option implicitly blocks on the stalled pusher.
                //
                // b and c are unacceptable, because they make the queue behave in unexpected ways AND may lead to implicit blocking behaviours.
                backoff.backoff();
                continue;
            }

            if !self.register_push(push_epoch) {
                continue;
            }

            let item = self.get_queue(push_epoch).pop();

            self.deregister_reader(push_epoch);

            if item.is_none() && push_epoch != self.push_epoch.load(Ordering::Acquire) {
                continue;
            }

            return item;
        }
    }

    fn capacity(&self) -> usize {
        // the capacity of the currently active queue, i.e. the number of elements that can be pushed directly after resize
        loop {
            let push_epoch = self.push_epoch.load(Ordering::Acquire);
            if !self.register_push(push_epoch) {
                continue;
            }
            let cap = self.get_queue(push_epoch).capacity();
            self.deregister_reader(push_epoch);
            return cap;
        }
    }

    fn len(&self) -> usize {
        // the total elements in the queue. Note that len can be > capacity.
        loop {
            let push_epoch = self.push_epoch.load(Ordering::Acquire);
            if !self.register_push(push_epoch) {
                continue;
            }

            let pop_epoch = self.pop_epoch.load(Ordering::Acquire);
            let pop_len = if pop_epoch != push_epoch {
                if !self.register_pop(pop_epoch) {
                    self.deregister_reader(push_epoch);
                    continue;
                }

                let pop_len = self.get_queue(pop_epoch).len();
                self.deregister_reader(pop_epoch);
                pop_len
            } else {
                0
            };

            let len = self.get_queue(push_epoch).len() + pop_len;
            self.deregister_reader(push_epoch);
            return len;
        }
    }

    fn is_empty(&self) -> bool {
        // the queue is empty if pop() returns None
        loop {
            let push_epoch = self.push_epoch.load(Ordering::Acquire);
            if !self.register_push(push_epoch) {
                continue;
            }

            let pop_epoch = self.pop_epoch.load(Ordering::Acquire);
            let pop_is_empty = if pop_epoch != push_epoch {
                if !self.register_pop(pop_epoch) {
                    self.deregister_reader(push_epoch);
                    continue;
                }

                let pop_is_empty = self.get_queue(pop_epoch).is_empty();
                self.deregister_reader(pop_epoch);
                pop_is_empty
            } else {
                true
            };

            let is_empty = self.get_queue(push_epoch).is_empty() && pop_is_empty;
            self.deregister_reader(push_epoch);
            return is_empty;
        }
    }

    fn is_full(&self) -> bool {
        // the queue is full if push() fails
        loop {
            let push_epoch = self.push_epoch.load(Ordering::Acquire);
            if !self.register_push(push_epoch) {
                continue;
            }
            let is_full = self.get_queue(push_epoch).is_full();
            self.deregister_reader(push_epoch);

            return is_full;
        }
    }
}

impl<T, Q, S> NewSized for GrowableQueueCore<T, Q, S>
where
    Q: NewSized,
{
    #[track_caller]
    fn with_size(size: usize) -> GrowableQueueCore<T, Q, S> {
        GrowableQueueCore::with_slot(size)
    }
}

impl<S> NewSized for QueueCore<BoxedBuffer<S>>
where
    S: Default,
{
    #[track_caller]
    fn with_size(size: usize) -> Self {
        Self::new_in(BoxedBuffer::new(size))
    }
}

/// A dynamically sized concurrent queue.
///
/// Operations on this queue are lock-free if `resize` is not actively used.
/// During an ongoing `resize` operation, `pop` and operations depending on it may block on stalled `pushes`.
pub struct DynamicQueue<T, S = Auto>
where
    S: SlotType<T>,
    T: AsPackedValue,
{
    inner: GrowableQueueCore<T, QueueCore<BoxedBuffer<S::Slot>>, S>,
}

impl<T> DynamicQueue<T, Auto>
where
    T: AsPackedValue,
{
    /// Constructs a new `DynamicQueue` with capacity `size` and slot type `Auto`.
    /// `T` must fit into the chosen slot type
    #[track_caller]
    pub fn new(size: usize) -> Self {
        Self::with_slot::<Auto>(size)
    }

    /// Constructs a new `DynamicQueue` with capacity `size` and slot type `S`.
    /// `T` must fit into the slot type `S`
    #[track_caller]
    pub fn with_slot<S>(size: usize) -> DynamicQueue<T, S>
    where
        S: SlotType<T>,
    {
        DynamicQueue {
            inner: GrowableQueueCore::with_slot::<S>(size),
        }
    }
}

impl<T, S> MPMCQueue for DynamicQueue<T, S>
where
    T: AsPackedValue,
    S: SlotType<T>,
{
    type Item = T;

    fn push(&self, item: Self::Item) -> Result<(), Self::Item> {
        self.inner.push(item)
    }

    /// This method may block on stalling pushes under concurrent resizes.
    ///
    /// For more info refer to the trait-level docs of `MPMCQueue`.
    fn pop(&self) -> Option<Self::Item> {
        self.inner.pop()
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn is_full(&self) -> bool {
        self.inner.is_full()
    }
}

impl<T, S> Resize for DynamicQueue<T, S>
where
    T: AsPackedValue,
    S: SlotType<T>,
{
    fn resize(&self, size: usize) -> bool {
        self.inner.resize(size)
    }
}

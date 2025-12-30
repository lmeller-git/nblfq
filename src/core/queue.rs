use core::sync::atomic::{AtomicUsize, Ordering};

use crate::{
    core::{buffer::Buffer, slot::Slot},
    slot::PtrLike,
    utils::{comp, prev},
};

pub trait MPMCQueue {
    type Item;

    /// Attempts to push an item into the queue.
    /// Returns the item as an error if the queue is full.
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[cfg(all(feature = "tagged-ptr", target_has_atomic = "64"))]
    /// fn run() {
    ///     use nblf_queue::{array::StaticQueue, MPMCQueue, slot::TaggedPtr64};
    ///
    ///     let q: StaticQueue<2, TaggedPtr64<_>> = StaticQueue::new();
    ///
    ///     assert_eq!(q.push(&10), Ok(()));
    ///     assert_eq!(q.push(&20), Ok(()));
    ///     assert_eq!(q.push(&30), Err(&30));
    ///     assert_eq!(q.pop(), Some(&10));
    /// }
    ///
    /// #[cfg(all(feature = "tagged-ptr", target_has_atomic = "64"))]
    /// run()
    /// ```
    fn push(&self, item: Self::Item) -> Result<(), Self::Item>;
    /// pop the last item, if an item is contained
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[cfg(all(feature = "tagged-ptr", target_has_atomic = "64"))]
    /// fn run() {
    ///     use nblf_queue::{array::StaticQueue, MPMCQueue, slot::TaggedPtr64};
    ///
    ///     let q: StaticQueue<1, TaggedPtr64<_>> = StaticQueue::new();
    ///
    ///     assert_eq!(q.push(&10), Ok(()));
    ///     assert_eq!(q.pop(), Some(&10));
    ///     assert!(q.pop().is_none());
    /// }
    ///
    /// #[cfg(all(feature = "tagged-ptr", target_has_atomic = "64"))]
    /// run()
    /// ```
    fn pop(&self) -> Option<Self::Item>;
    /// Returns the current len of the queue.
    /// This value may be stale.
    fn len(&self) -> usize;
    /// Returns the total capacity of the underlying buffer.
    fn capacity(&self) -> usize;

    /// Indicates whether the queue is empty.
    /// The result may be stale.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Indicates whether the queue is full.
    /// The result may be stale.
    fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }
}

// TODO add this to MPMCQueue
pub trait ForcePushQueue: MPMCQueue {
    /// Pushes an item into the queue, overwriting the last item if it is full
    /// This method does NOT guarantee atomicity. It simply calls pop(), until push() is succesfull.
    /// This also means that this method may spin for some time.
    /// The last popped item is returned, if the queue was full
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[cfg(all(feature = "tagged-ptr", target_has_atomic = "64"))]
    /// fn run() {
    ///     use nblf_queue::{array::StaticQueue, ForcePushQueue, MPMCQueue, slot::TaggedPtr64};
    ///
    ///     let q: StaticQueue<2, TaggedPtr64<_>> = StaticQueue::new();
    ///
    ///     assert_eq!(q.force_push(&10), None);
    ///     assert_eq!(q.force_push(&20), None);
    ///     assert_eq!(q.force_push(&30), Some(&10));
    ///     assert_eq!(q.pop(), Some(&20));
    /// }
    ///
    /// #[cfg(all(feature = "tagged-ptr", target_has_atomic = "64"))]
    /// run()
    /// ```
    fn force_push(&self, mut item: Self::Item) -> Option<Self::Item> {
        let mut popped_item = None;
        let mut backoff = 1;
        while let Err(item_) = self.push(item) {
            item = item_;
            for _ in 0..backoff {
                core::hint::spin_loop();
            }
            backoff = (backoff * 2).min(1024);
            popped_item = self.pop();
        }
        popped_item
    }
}

pub(crate) struct QueueCore<B: Buffer> {
    /// The buffer of the queue holding Item<T>'s
    buffer: B,
    /// The head of the queue.
    ///
    /// This value indicates the next slot that can be pushed to.
    ///
    /// This value may be stale and must be checked for critical operations.
    head: AtomicUsize,
    /// The tail of the queue.
    ///
    /// This value indicates the next slot that can be popped from.
    ///
    /// This value may be stale and must be checked for critical operations.
    tail: AtomicUsize,
}

impl<B: Buffer> QueueCore<B> {
    pub(crate) fn new_in(buffer: B) -> Self {
        Self {
            buffer,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }
}

impl<B: Buffer> MPMCQueue for QueueCore<B> {
    type Item = <B::Slot as Slot>::Item;

    fn push(&self, mut item: Self::Item) -> Result<(), Self::Item> {
        let mut head = self.head.load(Ordering::Acquire);
        loop {
            let components = loop {
                let prev_idx = prev(head, self.buffer.len());
                let current_item = self
                    .buffer
                    .inner()
                    .get(head)
                    .expect("QueueCore.head is out of bounds. This is a Bug.");
                let prev_item = self
                    .buffer
                    .inner()
                    .get(prev_idx)
                    .expect("QueueCore.head is out of bounds. This is a Bug.");
                let prev_components = prev_item.components();
                let current_componets = current_item.components();

                if !B::Slot::is_empty(prev_components.state)
                    && B::Slot::is_empty(current_componets.state)
                {
                    break prev_components;
                }

                if !comp(
                    prev_idx,
                    prev_components.count,
                    head,
                    current_componets.count,
                    B::Slot::MAX_W,
                ) {
                    if B::Slot::is_empty(prev_components.state)
                        && B::Slot::is_empty(current_componets.state)
                    {
                        // empty list
                        break prev_components;
                    }
                    if !B::Slot::is_empty(prev_components.state)
                        && !B::Slot::is_empty(current_componets.state)
                    {
                        // list full
                        return Err(item);
                    }
                }
                head = (head + 1) % self.buffer.len();
            };

            let mut new_counter = components.count;
            if B::Slot::is_empty(components.state) {
                // empty list
                new_counter = (components.count + B::Slot::MAX_W - 1) % B::Slot::MAX_W;
            }

            if head == 0 {
                // wrap around
                new_counter = (new_counter + 1) % B::Slot::MAX_W;
            }

            item = if let Err(Some(item)) = self
                .buffer
                .inner()
                .get(head)
                .expect("QueueCore.head is out of bounds. This is a Bug.")
                .cmpxchg(B::Slot::EMPTY_PTR, new_counter, Some(item), new_counter)
            {
                item
            } else {
                self.head
                    .store((head + 1) % self.buffer.len(), Ordering::Release);
                return Ok(());
            };
        }
    }

    fn pop(&self) -> Option<Self::Item> {
        loop {
            let mut tail = self.tail.load(Ordering::Acquire);
            let mut prev_idx = prev(tail, self.buffer.len());
            let prev_item = self.buffer.inner().get(prev_idx)?;
            let mut current_item = self.buffer.inner().get(tail)?;
            let mut prev_components = prev_item.components();
            let mut current_components = current_item.components();

            while comp(
                prev_idx,
                prev_components.count,
                tail,
                current_components.count,
                B::Slot::MAX_W,
            ) {
                tail = (tail + 1) % self.buffer.len();
                prev_idx = prev(tail, self.buffer.len());
                current_item = self.buffer.inner().get(tail)?;
                (prev_components, current_components) =
                    (current_components, current_item.components());
            }

            if (B::Slot::is_empty(prev_components.state)
                && B::Slot::is_empty(current_components.state))
                || B::Slot::is_contested(current_components.state)
            {
                // empty queue
                return None;
            }

            let next_count = (current_components.count + 1) % <B::Slot as Slot>::MAX_W;

            if let Ok(item) = current_item.cmpxchg(
                current_components.state as *const <B::Slot as Slot>::Item,
                current_components.count,
                None,
                next_count,
            ) {
                self.tail
                    .store((tail + 1) % self.buffer.len(), Ordering::Release);
                return Some(item.expect("We popped an empty item from the queue. This is a Bug."));
            }
        }
    }

    fn len(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        if head != tail {
            if head < tail {
                // wrap around
                self.capacity() - tail + head
            } else {
                // no wrap around
                head - tail
            }
        } else {
            // may be full or empty
            let components = self
                .buffer
                .inner()
                .get(head)
                .expect("head outside of cap")
                .components();
            if B::Slot::is_empty(components.state) {
                // empty
                0
            } else {
                // full
                self.capacity()
            }
        }
    }

    fn capacity(&self) -> usize {
        self.buffer.capacity()
    }
}

impl<B: Buffer> ForcePushQueue for QueueCore<B> where <B::Slot as Slot>::Item: PtrLike {}

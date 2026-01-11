use crate::{
    MPMCQueue,
    core::{buffer::Buffer, slot::Slot},
    sync::atomic::{AtomicUsize, Ordering},
    utils::{comp, prev},
};

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

impl<B> MPMCQueue for QueueCore<B>
where
    B: Buffer,
    B::Slot: Slot,
{
    type Item = <B::Slot as Slot>::Item;

    fn push(&self, mut item: Self::Item) -> Result<(), Self::Item> {
        let mut head = self.head.load(Ordering::Acquire);
        loop {
            let components = loop {
                let prev_idx = prev(head, self.buffer.capacity());
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
                head = (head + 1) % self.buffer.capacity();
            };

            // at this point components is prev(current_component)
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
                .cmpxchg(B::Slot::EMPTY_VALUE, new_counter, Some(item), new_counter)
            {
                item
            } else {
                self.head
                    .store((head + 1) % self.buffer.capacity(), Ordering::Release);
                return Ok(());
            };
        }
    }

    fn pop(&self) -> Option<Self::Item> {
        loop {
            let mut tail = self.tail.load(Ordering::Acquire);
            let mut prev_idx = prev(tail, self.buffer.capacity());
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
                prev_idx = tail;
                tail = (tail + 1) % self.buffer.capacity();
                current_item = self.buffer.inner().get(tail)?;
                (prev_components, current_components) =
                    (current_components, current_item.components());
            }

            if B::Slot::is_empty(prev_components.state)
                && B::Slot::is_empty(current_components.state)
            {
                // empty queue
                return None;
            }

            let next_count = (current_components.count + 1) % B::Slot::MAX_W;

            if let Ok(item) = current_item.cmpxchg(
                current_components.state,
                current_components.count,
                None,
                next_count,
            ) {
                self.tail
                    .store((tail + 1) % self.buffer.capacity(), Ordering::Release);
                debug_assert!(item.is_some(), "we popped an empty item from the queue");
                return item;
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

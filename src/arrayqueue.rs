use core::{
    fmt::Debug,
    iter,
    marker::PhantomData,
    ptr::null,
    sync::atomic::{AtomicUsize, Ordering},
};

use cfg_if::cfg_if;

use crate::{
    components::{self, ItemInner, PtrType},
    utils::{comp, prev},
};

cfg_if! {
    if #[cfg(feature = "alloc")] {
        pub use heap_based::*;
        pub use heapless::*;
    } else {
        pub use heapless::*;
    }
}

pub(crate) struct ArrayQueue<T, B: components::Buffer<T>> {
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
    _data: PhantomData<*const T>,
}

impl<T, B: components::Buffer<T>> ArrayQueue<T, B> {
    fn new_in(buffer: B) -> Self {
        Self {
            buffer,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            _data: PhantomData,
        }
    }
}

impl<T, B: components::Buffer<T>> ArrayQueue<T, B> {
    /// pop the last item, if an item is contained
    pub fn pop(&self) -> Option<*const T> {
        loop {
            let mut tail = self.tail.load(Ordering::Acquire);
            let mut prev_idx = prev(tail, self.buffer.len());
            let prev_item = self.buffer.inner().get(prev_idx)?;
            let mut current_item = self.buffer.inner().get(tail)?;
            let (mut prev_count, mut prev_ptr) = prev_item.components();
            let (mut current_count, mut current_ptr) = current_item.components();

            while comp(
                prev_idx,
                prev_count,
                tail,
                current_count,
                PtrType::<T>::MAX_W,
            ) {
                tail = (tail + 1) % self.buffer.len();
                prev_idx = prev(tail, self.buffer.len());
                current_item = self.buffer.inner().get(tail)?;
                (prev_count, prev_ptr, (current_count, current_ptr)) =
                    (current_count, current_ptr, current_item.components());
            }

            if prev_ptr.is_null() && current_ptr.is_null() {
                // empty queue
                return None;
            }

            let next_count = (current_count + 1) % PtrType::<T>::MAX_W;

            if let Ok((_, item)) =
                current_item.cmpxchg(current_ptr, current_count, null(), next_count)
            {
                self.tail
                    .store((tail + 1) % self.buffer.len(), Ordering::Release);
                return Some(item);
            }
        }
    }

    /// Attempts to push an item into the queue.
    /// Returns the item as an error if the queue is full.
    fn push(&self, item: *const T) -> Result<(), *const T> {
        let mut head = self.head.load(Ordering::Acquire);
        loop {
            let (count, prev_ptr) = loop {
                let prev_idx = prev(head, self.buffer.len());
                let current_item = self.buffer.inner().get(head).ok_or(item)?;
                let prev_item = self.buffer.inner().get(prev_idx).ok_or(item)?;
                let (prev_count, prev_ptr) = prev_item.components();
                let (current_count, current_ptr) = current_item.components();

                if !prev_ptr.is_null() && current_ptr.is_null() {
                    break (prev_count, prev_ptr);
                }

                if !comp(
                    prev_idx,
                    prev_count,
                    head,
                    current_count,
                    PtrType::<T>::MAX_W,
                ) {
                    if prev_ptr.is_null() && current_ptr.is_null() {
                        // empty list
                        break (prev_count, prev_ptr);
                    }
                    if !prev_ptr.is_null() && !current_ptr.is_null() {
                        // list full
                        return Err(item);
                    }
                }
                head = (head + 1) % self.buffer.len();
            };

            let mut new_counter = count;
            if prev_ptr.is_null() {
                // empty list
                new_counter = (count + PtrType::<T>::MAX_W - 1) % PtrType::<T>::MAX_W;
            }

            if head == 0 {
                // wrap around
                new_counter = (new_counter + 1) % PtrType::<T>::MAX_W;
            }

            if self
                .buffer
                .inner()
                .get(head)
                .ok_or(item)?
                .cmpxchg(null(), new_counter, item, new_counter)
                .is_ok()
            {
                self.head
                    .store((head + 1) % self.buffer.len(), Ordering::Release);
                return Ok(());
            }
        }
    }

    /// Returns the total capacity of the underlying buffer.
    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    /// Returns the current len of the queue.
    /// This value may be stale.
    pub fn len(&self) -> usize {
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
            let (_, item) = self
                .buffer
                .inner()
                .get(head)
                .expect("head outside of cap")
                .components();
            if item.is_null() {
                // empty
                0
            } else {
                // full
                self.capacity()
            }
        }
    }

    /// Indicates whether the queue is empty.
    /// The result may be stale.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Indicates whether the queue is full.
    /// The result may be stale.
    pub fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }
}

#[cfg(feature = "alloc")]
mod heap_based {
    use super::*;
    use alloc::boxed::Box;

    pub struct HeapBackedQueue<T>(ArrayQueue<T, components::FixedBuf<T>>);

    impl<T> HeapBackedQueue<T> {
        pub fn new(size: usize) -> Self {
            assert!(size > 0, "Size of the queue must be greater than 0");
            Self(ArrayQueue::new_in(components::FixedBuf::new(size)))
        }

        /// Attempts to push an item into the queue.
        /// Returns the item as an error if the queue is full.
        ///
        /// # Examples
        ///
        /// ```
        /// use nblfq::HeapBackedQueue;
        ///
        /// let q = HeapBackedQueue::new(2);
        ///
        /// assert_eq!(q.push(10), Ok(()));
        /// assert_eq!(q.push(20), Ok(()));
        /// assert_eq!(q.push(30), Err(30));
        /// assert_eq!(q.pop(), Some(10));
        /// ```
        pub fn push(&self, item: T) -> Result<(), T> {
            let item = Box::into_raw(Box::new(item));
            self.0
                .push(item)
                .map_err(|item| unsafe { *Box::from_raw(item as *mut T) })
        }

        /// Pushes an item into the queue, overwriting the last item if it is full
        /// This method does NOT guarantee atomicity. It simply calls pop(), until push() is succesfull.
        /// This also means that this method may spin for some time.
        /// The last popped item is returned, if the queue was full
        ///
        /// # Examples
        ///
        /// ```
        /// use nblfq::HeapBackedQueue;
        ///
        /// let q = HeapBackedQueue::new(2);
        ///
        /// assert_eq!(q.force_push(10), None);
        /// assert_eq!(q.force_push(20), None);
        /// assert_eq!(q.force_push(30), Some(10));
        /// assert_eq!(q.pop(), Some(20));
        /// ```
        pub fn force_push(&self, item: T) -> Option<T> {
            let mut popped_item = None;
            let mut container = item;
            let mut backoff = 1;
            while let Err(item) = self.push(container) {
                container = item;
                for _ in 0..backoff {
                    use core::hint::spin_loop;

                    spin_loop();
                }
                backoff = (backoff * 2).min(1024);
                popped_item = self.pop();
            }
            popped_item
        }

        /// pop the last item, if an item is contained
        ///
        /// # Examples
        ///
        /// ```
        /// use nblfq::HeapBackedQueue;
        ///
        /// let q = HeapBackedQueue::new(1);
        /// assert_eq!(q.push(10), Ok(()));
        ///
        /// assert_eq!(q.pop(), Some(10));
        /// assert!(q.pop().is_none());
        /// ```
        pub fn pop(&self) -> Option<T> {
            self.0
                .pop()
                .map(|item| unsafe { *Box::from_raw(item as *mut T) })
        }

        /// Returns the total capacity of the underlying buffer.
        pub fn capacity(&self) -> usize {
            self.0.capacity()
        }

        /// Returns the current len of the queue.
        /// This value may be stale.
        pub fn len(&self) -> usize {
            self.0.len()
        }

        /// Indicates whether the queue is empty.
        /// The result may be stale.
        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        /// Indicates whether the queue is full.
        /// The result may be stale.
        pub fn is_full(&self) -> bool {
            self.0.is_full()
        }
    }

    impl<T> Debug for HeapBackedQueue<T> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.pad("HeapBackedQueue { ... }")
        }
    }

    impl<T> Drop for HeapBackedQueue<T> {
        fn drop(&mut self) {
            // drop all leaked boxes
            while self.pop().is_some() {}
        }
    }

    impl<T> IntoIterator for HeapBackedQueue<T> {
        type Item = T;
        type IntoIter = impl Iterator<Item = Self::Item>;

        fn into_iter(self) -> Self::IntoIter {
            iter::from_fn(move || self.pop())
        }
    }

    /// Safety: HeapBackedQueue sends owned T's between threads.
    /// It is only safe to do so, if T is Send
    unsafe impl<T: Send> Sync for HeapBackedQueue<T> {}
    unsafe impl<T: Send> Send for HeapBackedQueue<T> {}
}

mod heapless {
    use super::*;

    pub struct HeaplessQueue<const N: usize, T>(ArrayQueue<T, components::HeaplessBuf<N, T>>);

    impl<const N: usize, T> HeaplessQueue<N, T> {
        pub fn new() -> Self {
            assert!(N > 0, "Size of the queue must be greater than 0");
            Self(ArrayQueue::new_in(components::HeaplessBuf::new()))
        }

        /// Attempts to push an item into the queue.
        /// Returns the item as an error if the queue is full.
        ///
        /// # Examples
        ///
        /// ```
        /// use nblfq::HeaplessQueue;
        ///
        /// let q: HeaplessQueue<2, _> = HeaplessQueue::new();
        ///
        /// assert_eq!(q.push(&10), Ok(()));
        /// assert_eq!(q.push(&20), Ok(()));
        /// assert_eq!(q.push(&30), Err(&30));
        /// assert_eq!(q.pop(), Some(&10));
        /// ```
        pub fn push(&self, item: &'static T) -> Result<(), &'static T> {
            let item = item as *const T;
            self.0.push(item).map_err(|item| unsafe { &*item })
        }

        /// Pushes an item into the queue, overwriting the last item if it is full
        /// This method does NOT guarantee atomicity. It simply calls pop(), until push() is succesfull.
        /// This also means that this method may spin for some time.
        /// The last popped item is returned, if the queue was full
        ///
        /// # Examples
        ///
        /// ```
        /// use nblfq::HeaplessQueue;
        ///
        /// let q: HeaplessQueue<2, _> = HeaplessQueue::new();
        ///
        /// assert_eq!(q.force_push(&10), None);
        /// assert_eq!(q.force_push(&20), None);
        /// assert_eq!(q.force_push(&30), Some(&10));
        /// assert_eq!(q.pop(), Some(&20));
        /// ```
        pub fn force_push(&self, item: &'static T) -> Option<&'static T> {
            let mut popped_item = None;
            let mut backoff = 1;
            while self.push(item).is_err() {
                for _ in 0..backoff {
                    use core::hint::spin_loop;

                    spin_loop();
                }
                backoff = (backoff * 2).min(1024);
                popped_item = self.pop();
            }
            popped_item
        }

        /// pop the last item, if an item is contained
        ///
        /// # Examples
        ///
        /// ```
        /// use nblfq::HeaplessQueue;
        ///
        /// let q: HeaplessQueue<1, _> = HeaplessQueue::new();
        /// assert_eq!(q.push(&10), Ok(()));
        ///
        /// assert_eq!(q.pop(), Some(&10));
        /// assert!(q.pop().is_none());
        /// ```
        pub fn pop(&self) -> Option<&'static T> {
            self.0.pop().map(|item| unsafe { &*item })
        }

        /// Returns the total capacity of the underlying buffer.
        pub fn capacity(&self) -> usize {
            self.0.capacity()
        }

        /// Returns the current len of the queue.
        /// This value may be stale.
        pub fn len(&self) -> usize {
            self.0.len()
        }

        /// Indicates whether the queue is empty.
        /// The result may be stale.
        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        /// Indicates whether the queue is full.
        /// The result may be stale.
        pub fn is_full(&self) -> bool {
            self.0.is_full()
        }
    }

    impl<const N: usize, T> Default for HeaplessQueue<N, T> {
        fn default() -> Self {
            assert!(N > 0, "Size of the queue must be greater than 0");
            Self::new()
        }
    }

    impl<const N: usize, T> Debug for HeaplessQueue<N, T> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.pad("HeaplessQueue { ... }")
        }
    }

    impl<const N: usize, T: 'static> IntoIterator for HeaplessQueue<N, T> {
        type Item = &'static T;
        type IntoIter = impl Iterator<Item = Self::Item>;

        fn into_iter(self) -> Self::IntoIter {
            iter::from_fn(move || self.pop())
        }
    }

    /// Safety: HeaplessQueue sends static ref T's between threads.
    /// It is only safe to do so if T is Sync
    unsafe impl<const N: usize, T: Sync> Sync for HeaplessQueue<N, T> {}
    unsafe impl<const N: usize, T: Sync> Send for HeaplessQueue<N, T> {}
}

use core::{
    marker::PhantomData,
    ptr::null,
    sync::atomic::{AtomicUsize, Ordering},
};

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

use crate::{
    components::{self, Item, ItemInner, PtrType},
    utils::{comp, prev},
};

pub struct HeaplessQueue<const N: usize, T>(ArrayQueue<T, components::HeaplessBuf<N, T>>);

#[cfg(feature = "alloc")]
pub struct HeapBackedQueue<T>(ArrayQueue<T, components::FixedBuf<T>>);

pub(crate) struct ArrayQueue<T, B: components::Buffer<T>> {
    buffer: B,
    head: AtomicUsize,
    tail: AtomicUsize,
    _data: PhantomData<T>,
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
    /// Attempts to push an item into the queue.
    /// Returns the item as an error if the queue is full.
    pub fn push(&self, item: *const T) -> Result<(), *const T> {
        self.push_or_else(item, |_, _, _| Err(item))
    }

    /// Pushes an item into the queue, overwriting the last item if it is full
    pub fn force_push(&self, item: *const T) -> Option<*const T> {
        self.push_or_else(item, |mut prev_count, current_item, head| {
            if head == 0 {
                prev_count = (prev_count + 1) % PtrType::<T>::MAX_W;
            }
            if let Ok((_, value)) = current_item.cmpxchg(null(), prev_count, item, prev_count) {
                Err(value)
            } else {
                Ok(())
            }
        })
        .err()
    }

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

    fn push_or_else<F>(&self, item: *const T, f: F) -> Result<(), *const T>
    where
        F: Fn(u64, &Item<T>, usize) -> Result<(), *const T>,
    {
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
                        // TODO: call f() with relevant args here
                        f(prev_count, current_item, head)?;
                    }
                }
                head = (head + 1) % self.buffer.len();
            };
            let mut new_counter = count;
            // println!("found head: {}", head);
            if prev_ptr.is_null() {
                // empty list
                new_counter = (count + PtrType::<T>::MAX_W - 1) % PtrType::<T>::MAX_W;
                // println!("empty, count: {new_counter}");
            }

            if head == 0 {
                // wrap around
                new_counter = (new_counter + 1) % PtrType::<T>::MAX_W;
                // println!("wrapping, count: {new_counter}");
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
                // println!("done");
                return Ok(());
            }
        }
    }
}

#[cfg(feature = "alloc")]
impl<T> HeapBackedQueue<T> {
    pub fn new(size: usize) -> Self {
        Self(ArrayQueue::new_in(components::FixedBuf::new(size)))
    }
    /// Attempts to push an item into the queue.
    /// Returns the item as an error if the queue is full.
    pub fn push(&self, item: T) -> Result<(), T> {
        let item = Box::into_raw(Box::new(item));
        self.0
            .push(item)
            .map_err(|item| unsafe { *Box::from_raw(item as *mut T) })
    }

    /// Pushes an item into the queue, overwriting the last item if it is full
    pub fn force_push(&self, item: T) -> Option<T> {
        let item = Box::into_raw(Box::new(item));
        self.0
            .force_push(item)
            .map(|item| unsafe { *Box::from_raw(item as *mut T) })
    }

    /// pop the last item, if an item is contained
    pub fn pop(&self) -> Option<T> {
        self.0
            .pop()
            .map(|item| unsafe { *Box::from_raw(item as *mut T) })
    }

    /// Attempts to push an item into the queue.
    /// Returns the item as an error if the queue is full.
    /// Items are internally stored as ptrs, storing a 'static ref is cheaper than an owned value/
    pub fn push_ref(&self, item: &'static T) -> Result<(), &'static T> {
        let item = item as *const T;
        self.0.push(item).map_err(|item| unsafe { &*item })
    }

    /// Pushes an item into the queue, overwriting the last item if it is full
    /// Items are internally stored as ptrs, storing a 'static ref is cheaper than an owned value/
    pub fn force_push_ref(&self, item: &'static T) -> Option<&'static T> {
        let item = item as *const T;
        self.0.force_push(item).map(|item| unsafe { &*item })
    }

    /// pop the last item as 'static ref, if an item is contained
    pub fn pop_ref(&self) -> Option<&'static T> {
        self.0.pop().map(|item| unsafe { &*item })
    }
}

impl<const N: usize, T> HeaplessQueue<N, T> {
    pub fn new() -> Self {
        Self(ArrayQueue::new_in(components::HeaplessBuf::new()))
    }
    /// Attempts to push an item into the queue.
    /// Returns the item as an error if the queue is full.
    pub fn push(&self, item: &'static T) -> Result<(), &'static T> {
        let item = item as *const T;
        self.0.push(item).map_err(|item| unsafe { &*item })
    }

    /// Pushes an item into the queue, overwriting the last item if it is full
    pub fn force_push(&self, item: &'static T) -> Option<&'static T> {
        let item = item as *const T;
        self.0.force_push(item).map(|item| unsafe { &*item })
    }

    /// pop the last item, if an item is contained
    pub fn pop(&self) -> Option<&'static T> {
        self.0.pop().map(|item| unsafe { &*item })
    }
}

impl<const N: usize, T> Default for HeaplessQueue<N, T> {
    fn default() -> Self {
        Self::new()
    }
}

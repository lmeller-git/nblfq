//! TODO doc for crate

#![no_std]
#![warn(missing_docs)]
#![warn(clippy::missing_safety_doc)]
#![warn(unsafe_op_in_unsafe_fn)]
#![warn(clippy::undocumented_unsafe_blocks)]

#[cfg(any(feature = "alloc", test))]
extern crate alloc;

#[cfg(any(feature = "std", test))]
extern crate std;

mod array;
pub mod core;
#[cfg(any(feature = "alloc", test))]
mod owned;
#[cfg(feature = "pool")]
mod pool;
#[cfg(test)]
mod tests;
mod utils;

#[cfg(feature = "pool")]
pub use array::PooledStaticQueue;
pub use array::StaticQueue;
#[cfg(all(any(feature = "alloc", test), feature = "pool"))]
pub use owned::PooledQueue;
#[cfg(any(feature = "alloc", test))]
pub use owned::Queue;

/// The main trait used to interface with a MPMCQueue.
/// All methods in this trait are non-blocking and may fail.
pub trait MPMCQueue {
    /// The item stored in the queue
    type Item;

    /// Attempts to push an item into the queue.
    /// Returns the item as an error if the queue is full.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nblf_queue::{StaticQueue, MPMCQueue};
    ///
    /// let q: StaticQueue<_, 2> = StaticQueue::new();
    ///
    /// assert_eq!(q.push(&10), Ok(()));
    /// assert_eq!(q.push(&20), Ok(()));
    /// assert_eq!(q.push(&30), Err(&30));
    /// assert_eq!(q.pop(), Some(&10));
    /// ```
    fn push(&self, item: Self::Item) -> Result<(), Self::Item>;
    /// pop the last item, if an item is contained
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nblf_queue::{StaticQueue, MPMCQueue};
    ///
    /// let q: StaticQueue<_, 1> = StaticQueue::new();
    ///
    /// assert_eq!(q.push(&10), Ok(()));
    /// assert_eq!(q.pop(), Some(&10));
    /// assert!(q.pop().is_none());
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

    /// Pushes an item into the queue, overwriting the last item if it is full
    /// This method does NOT guarantee atomicity. It simply calls pop(), until push() is succesfull.
    /// This also means that this method may spin for some time.
    /// The last popped item is returned, if the queue was full
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nblf_queue::{StaticQueue, MPMCQueue};
    ///
    /// let q: StaticQueue<_, 2> = StaticQueue::new();
    ///
    /// assert_eq!(q.force_push(&10), None);
    /// assert_eq!(q.force_push(&20), None);
    /// assert_eq!(q.force_push(&30), Some(&10));
    /// assert_eq!(q.pop(), Some(&20));
    /// ```
    fn force_push(&self, mut item: Self::Item) -> Option<Self::Item> {
        let mut popped_item = None;
        let mut backoff = 1;
        while let Err(item_) = self.push(item) {
            item = item_;
            for _ in 0..backoff {
                ::core::hint::spin_loop();
            }
            backoff = (backoff * 2).min(1024);
            popped_item = self.pop();
        }
        popped_item
    }
}

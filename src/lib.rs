#![doc = include_str!("../README.md")]
#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![deny(missing_docs)]
#![deny(clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]
#![warn(unsafe_op_in_unsafe_fn)]

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
mod sync;
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
/// All implementations provided by this crate are atomic and non-blocking.
/// Fallible operations of this trait may fail spuriously.
///
/// # Examples
///
/// ```rust
/// use nblf_queue::{StaticQueue, MPMCQueue};
///
/// let q: StaticQueue<_, 2> = StaticQueue::new();
///
/// assert!(q.push(&42).is_ok());
/// assert!(q.push(&2).is_ok());
///
/// assert_eq!(q.len(), 2);
/// assert!(q.is_full());
///
/// assert_eq!(q.force_push(&0), Some(&42));
/// assert!(q.is_full());
///
/// assert_eq!(q.pop(), Some(&2));
/// assert_eq!(q.pop(), Some(&0));
/// assert_eq!(q.len(), 0);
/// assert!(q.is_empty());
/// ```
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
    /// assert!(q.push(&10).is_ok());
    /// assert!(q.push(&20).is_ok());
    /// assert_eq!(q.push(&30), Err(&30));
    /// assert_eq!(q.pop(), Some(&10));
    /// ```
    fn push(&self, item: Self::Item) -> Result<(), Self::Item>;
    /// Attempts to pop an item from the queue.
    /// Returns `None` if the queue was empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nblf_queue::{StaticQueue, MPMCQueue};
    ///
    /// let q: StaticQueue<_, 2> = StaticQueue::new();
    ///
    /// assert!(q.push(&10).is_ok());
    /// assert!(q.push(&42).is_ok());
    /// assert_eq!(q.pop(), Some(&10));
    /// assert_eq!(q.pop(), Some(&42));
    /// assert!(q.pop().is_none());
    /// ```
    fn pop(&self) -> Option<Self::Item>;
    /// Returns the current len of the queue.
    /// The returned value may be stale under concurrent access.
    fn len(&self) -> usize;
    /// Returns the total capacity of the queue.
    fn capacity(&self) -> usize;

    /// Indicates whether the queue is empty.
    /// The result may be stale under concurrent access.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Indicates whether the queue is full.
    /// The result may be stale under concurrent access.
    fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }

    /// Pushes an item into the queue, removing an existing item if the queue is full.
    ///
    /// If the queue is full, this method will remove items until space becomes available.
    /// The last removed item is returned.
    ///
    /// Under contention this method may spin for some time, however it will never block.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nblf_queue::{StaticQueue, MPMCQueue};
    ///
    /// let q: StaticQueue<_, 2> = StaticQueue::new();
    ///
    /// assert!(q.force_push(&10).is_none());
    /// assert!(q.force_push(&20).is_none());
    /// assert_eq!(q.force_push(&30), Some(&10));
    /// assert_eq!(q.pop(), Some(&20));
    /// ```
    fn force_push(&self, mut item: Self::Item) -> Option<Self::Item> {
        let mut popped_item = None;
        let mut backoff = 1;
        while let Err(item_) = self.push(item) {
            item = item_;
            for _ in 0..backoff {
                crate::sync::hint::spin_loop();
            }
            backoff = (backoff * 2).min(1024);
            popped_item = self.pop();
        }
        popped_item
    }
}

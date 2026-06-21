#![doc = include_str!("../README.md")]
#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![deny(missing_docs)]
#![deny(clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]
#![warn(unsafe_op_in_unsafe_fn)]

#[cfg(any(feature = "alloc", test))]
extern crate alloc;

#[cfg(any(feature = "std", test))]
extern crate std;

#[macro_use]
pub(crate) mod utils;
mod array;
pub mod core;
#[cfg(any(feature = "dynamic", test))]
mod growable;
#[cfg(any(feature = "alloc", test))]
mod owned;
#[cfg(feature = "pool")]
mod pool;
mod sync;
#[cfg(test)]
mod tests;

#[cfg(feature = "pool")]
pub use array::PooledStaticQueue;
pub use array::StaticQueue;
#[cfg(any(feature = "dynamic", test))]
pub use growable::DynamicQueue;
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
    /// The returned value may be stale under concurrent access and should not be used for synchronization.
    fn len(&self) -> usize;
    /// Returns the total capacity of the queue.
    fn capacity(&self) -> usize;

    /// Indicates whether the queue is empty.
    /// The returned value may be stale under concurrent access and should not be used for synchronization.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Indicates whether the queue is full.
    /// The returned value may be stale under concurrent access and should not be used for synchronization.
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
    fn force_push(&self, item: Self::Item) -> Option<Self::Item> {
        let mut item_container = None;
        self.force_push_and_do(item, |item| {
            item_container.replace(item);
        });
        item_container
    }

    /// Pushes an item into the queue, removing an existing item if the queue is full.
    ///
    /// If the queue is full, this method will remove items until space becomes available.
    /// The provided closure will be called on each removed item.
    ///
    /// Under contention this method may spin for some time, however it will never block, provided the passed closure does not block.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nblf_queue::{StaticQueue, MPMCQueue};
    ///
    /// let q: StaticQueue<_, 2> = StaticQueue::new();
    ///
    /// q.force_push_and_do(&10, |item| {});
    /// q.force_push_and_do(&20, |item| {});
    /// q.force_push_and_do(&30, |item| {
    ///     assert_eq!(item, &10)
    /// });
    /// assert_eq!(q.pop(), Some(&20));
    /// ```
    fn force_push_and_do<F>(&self, mut item: Self::Item, mut f: F)
    where
        F: FnMut(Self::Item),
    {
        let mut backoff = crate::utils::Backoff::new();
        while let Err(item_) = self.push(item) {
            item = item_;
            backoff.backoff();
            if let Some(next_popped_item) = self.pop() {
                f(next_popped_item);
            }
        }
    }
}

/// An extension trait for MPMCQueues, which allows dynamic growth of the queue.
pub trait Growable: MPMCQueue {
    /// Grows the capacity of the queue by `by` slots. This method may block, if the backign allocator is blocking.
    /// Only one concurrent `grow_by` may happen. A `grow_by` may not be conidered as finished until some time after the call to `grow_by` happened.
    /// Returns false if `grow_by` failed.
    fn grow_by(&self, by: usize) -> bool;
}

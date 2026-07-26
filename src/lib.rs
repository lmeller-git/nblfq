//! An atomic lock-free MPMC queue based on the NBLFQ algorithm.
//!
//! This repository provides multiple queue implementations with different storage and allocation strategies.
//!
//! All queues in this repository are safe to use in a concurrent context.
//! All variants with the exception of [`DynamicQueue`] and [`PooledDynamicQueue`] are strictly non-blocking and will never block the calling thread.
//!
//! ## Queue variants
//!
//! - **Static queues**: fixed-capacity queues backed by static storage.
//! - **Allocated queues**: fixed-capacity queues backed by dynamically allocated storage, only available on feature `alloc`.
//! - **Dynamic queues**: dynamically resizeable queues, only available on feature `dynamic`.
//! - **Pooled Queues**: variants of other queues, which may store arbitrary types, only available on feature `pool`.
//!
//! Non-pooled queues store items in atomically updated slots, restricting the stored items to small, pointer-like values.
//!
//! ## Usage
//!
//! [`StaticQueue`]:
//!
//! ```rust
//!   #[cfg(feature = "unsafe-ptr48")]
//!   fn run() {
//!     use nblf_queue::{StaticQueue, MPMCQueue};
//!
//!     let q: StaticQueue<_, 2> = StaticQueue::new();
//!
//!     assert!(q.push(&42).is_ok());
//!     assert!(q.push(&1).is_ok());
//!     assert!(q.push(&4242).is_err());
//!
//!     assert_eq!(q.pop(), Some(&42));
//!     assert_eq!(q.pop(), Some(&1));
//!     assert!(q.pop().is_none());
//!   }
//!
//!   #[cfg(feature = "unsafe-ptr48")]
//!   run();
//! ```
//!
//! [`PooledStaticQueue`]:
//!
//! ```rust
//!   #[cfg(feature = "pool")]
//!   fn run() {
//!     use nblf_queue::{PooledStaticQueue, MPMCQueue};
//!
//!     let q: PooledStaticQueue<_, 2> = PooledStaticQueue::new();
//!
//!     assert!(q.push(42).is_ok());
//!     assert!(q.push(1).is_ok());
//!     assert!(q.push(4242).is_err());
//!
//!     assert_eq!(q.pop(), Some(42));
//!     assert_eq!(q.pop(), Some(1));
//!     assert!(q.pop().is_none());
//!   }
//!
//!   #[cfg(feature = "pool")]
//!   run();
//! ```
//!
//! [`DynamicQueue`]:
//!
//! ```rust
//!   #[cfg(feature = "dynamic")]
//!   fn run() {
//!     use nblf_queue::{DynamicQueue, MPMCQueue, Resize};
//!
//!     let q = DynamicQueue::new(1);
//!
//!     assert!(q.push(42).is_ok());
//!     assert!(q.push(4242).is_err());
//!
//!     assert!(q.resize(2));
//!     assert_eq!(q.capacity(), 2);
//!     assert!(q.push(4242).is_ok());
//!
//!     assert_eq!(q.pop(), Some(42));
//!     assert_eq!(q.pop(), Some(4242));
//!     assert!(q.pop().is_none());
//!   }
//!
//!   #[cfg(feature = "dynamic")]
//!   run();
//! ```
//!
//! ## Choosing a queue type
//!
//! Do you have an allocator? -> Use a non-static Queue.
//! Do you want to send large owned items? -> Use `Pooled*`.
//! Do you want to resize your queue? -> Use `Dynamic*`.
//!
//! - [`StaticQueue`] and [`Queue`]: may only store small values and are optimized for this use case.
//!
//! - [`PooledStaticQueue`] and [`PooledQueue`]: may store arbitrary types, at the cost of higher memory usage and runtime cost.
//!
//! - [`DynamicQueue`] and [`PooledDynamicQueue`]: may be resized dynamically, at the cost of higher total memory usage and runtime cost. This cost is even higher for [`PooledDynamicQueue`].
//!
//! > [!WARNING]
//! > **Blocking Behaviour in Dynamic Queues**
//! >
//! > All dynamic queues may block on concurrent `resize` operations.
//! >
//! > Additionally, dynamic queues may block on `pop` operations (and operations depending on it), if a `push` is preempted by a concurrent `resize` and a concurrent `pop` happens.
//!
//! ## Platform Support
//!
//! Multiple storage types are available, dependent on platform:
//!
//! - **Tagged64** - platforms with native 64-bit atomic operations or feature `atomic-fallback`.
//!
//! - **Tagged128** - platforms with native 128-bit atomic operations or feature `atomic-fallback`.
//!
//! Storage types will be chosen automatically, unless sepcified explicitly.
//!
//! > [!NOTE]
//! > **ABA Safety & Storage Selection**
//! >
//! > If it is plausible that other threads could perform `(2^15 - 1) * queue_size`
//! > pop and push operations while a single thread is paused/preempted in pop/push, [`core::slots::Tagged128`] slots should be used to ensure ABA safety.
//! >
//! > **Tagged64 Safety**
//! >
//! > Sending ptr-types via [`core::slots::Tagged64`] slots is not safe if more than 48 bits are used for pointers.
//! > This is currently enforced with a runtime check, however some unsafe usages may be missed by this check.
//!
//! ## Feature Flags
//!
//! - `std`: Enables `std` and `alloc` support.
//! - `alloc`: Enables `alloc` support, allowing usage of some dynamically allocated queues.
//! - `pool`: Enables pooled queues, which may store any type.
//! - `dynamic`: Enables dynamic queues, which may be dynamically resized. Depends on `alloc`.
//! - `atomic-fallback`: Uses `portable-atomic` `fallback` feature for atomics if necessary. It is discouraged to use this feature, as `fallback` internally uses locks.
//! - `unsafe-ptr48`: implements AsPackedValue for pointers on `x86-64` and `aarch64`. This feature is safe to use if 48 or less bits are used for pointers on the target platform.
//! - `default`: `pool`
//!
//! ## Python Bindings
//!
//! Python bindings backed by [`PooledQueue`] and [`PooledDynamicQueue`] are available for concurrent applications.
//! Core operations detach from the GIL to allow parallel execution.
//!
//! > [!NOTE]
//! > The Python bindings strictly use [`core::slots::Auto`] slots without feature `atomic-fallback`.
//! > As a result, these bindings are only supported on platforms with native 64-bit or 128-bit atomic operations.
//!
//! ```python
//!   from nblf_queue import Queue, DynamicQueue
//!
//!   q: Queue[int] = Queue(10)
//!
//!   assert q.push(42) is None
//!   item = q.pop()
//!   assert item == 42
//!
//!   dq: DynamicQueue[str] = DynamicQueue(1)
//!
//!   assert dq.push("hello") is None
//!   assert dq.resize(42)
//!   assert dq.push("world") is None
//!
//! ```
//!
//! ## Testing
//!
//! The core test-suite of this crate was adapted from [`crossbeam-queue`](https://!github.com/crossbeam-rs/crossbeam/tree/main/crossbeam-queue).
//!
//! Current testing is based on:
//!
//! - **Miri** - to validate pointer arithmetic and catch UB.
//! - **Loom and Shuttle** - to test for race conditions and blocking code.
//! - **Echeneis** - to test basic obstruction freedom.
//! - **ASan** - to check for memory corruption.
//!
//! ## References
//!
//! Alexandre Denis, Charles Goedefroit. NBLFQ: a lock-free MPMC queue optimized for low contention.
//! IPDPS 2025 - 39th International Parallel & Distributed Processing Symposium, IEEE, Jun 2025,
//! Milan, Italy. hal-04851700v2

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
#[cfg(feature = "dynamic")]
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
#[cfg(feature = "dynamic")]
pub use growable::DynamicQueue;
#[cfg(all(feature = "dynamic", feature = "pool"))]
pub use growable::PooledDynamicQueue;
#[cfg(all(feature = "alloc", feature = "pool"))]
pub use owned::PooledQueue;
#[cfg(any(feature = "alloc", test))]
pub use owned::Queue;

/// The main trait used to interface with a `MPMCQueue`.
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
/// assert!(q.push(42).is_ok());
/// assert!(q.push(2).is_ok());
///
/// assert_eq!(q.len(), 2);
/// assert!(q.is_full());
///
/// assert_eq!(q.force_push(0), Some(42));
/// assert!(q.is_full());
///
/// assert_eq!(q.pop(), Some(2));
/// assert_eq!(q.pop(), Some(0));
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
    /// assert!(q.push(10).is_ok());
    /// assert!(q.push(20).is_ok());
    /// assert_eq!(q.push(30), Err(30));
    /// assert_eq!(q.pop(), Some(10));
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
    /// assert!(q.push(10).is_ok());
    /// assert!(q.push(42).is_ok());
    /// assert_eq!(q.pop(), Some(10));
    /// assert_eq!(q.pop(), Some(42));
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
    /// Note that the behaviour of this method depends on both `push` and `pop`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nblf_queue::{StaticQueue, MPMCQueue};
    ///
    /// let q: StaticQueue<_, 2> = StaticQueue::new();
    ///
    /// assert!(q.force_push(10).is_none());
    /// assert!(q.force_push(20).is_none());
    /// assert_eq!(q.force_push(30), Some(10));
    /// assert_eq!(q.pop(), Some(20));
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
    /// Note that the behaviour of this method depends on both `push` and `pop`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nblf_queue::{StaticQueue, MPMCQueue};
    ///
    /// let q: StaticQueue<_, 2> = StaticQueue::new();
    ///
    /// q.force_push_and_do(10, |item| {});
    /// q.force_push_and_do(20, |item| {});
    /// q.force_push_and_do(30, |item| {
    ///     assert_eq!(item, 10)
    /// });
    /// assert_eq!(q.pop(), Some(20));
    /// ```
    fn force_push_and_do<F>(&self, mut item: Self::Item, mut f: F)
    where
        F: FnMut(Self::Item),
    {
        let mut backoff = utils::Backoff::new();
        while let Err(item_) = self.push(item) {
            item = item_;
            backoff.backoff();
            if let Some(next_popped_item) = self.pop() {
                f(next_popped_item);
            }
        }
    }
}

/// An extension trait for `MPMCQueue` that allows dynamic resizing of the queue.
///
/// This trait makes **no** guarantees regarding the blocking behavior of the `resize` method itself.
/// It only guarantees that the core `MPMCQueue` operations maintain their original guarantees
/// and that the resize operation is atomically published to other threads.
///
/// # Examples
///
/// ```rust
/// #[cfg(feature = "dynamic")]
/// fn run() {
///  use nblf_queue::{DynamicQueue, MPMCQueue, Resize};
///
///  let q = DynamicQueue::new(1);
///
///  assert!(q.push(1).is_ok());
///  assert!(q.is_full());
///
///  assert!(q.resize(2 + q.capacity()));
///
///  assert_eq!(q.capacity(), 3);
///  assert!(!q.is_full());
///
///  assert!(q.push(2).is_ok());
///  assert!(q.push(3).is_ok());
///
///  assert_eq!(q.pop(), Some(1));
///  assert_eq!(q.pop(), Some(2));
///  assert_eq!(q.pop(), Some(3));
///
///  assert!(q.is_empty());
/// }
///
/// #[cfg(feature = "dynamic")]
/// run();
/// ```
pub trait Resize: MPMCQueue {
    /// Attempts to resize the capacity of the queue to `size` slots.
    ///
    /// **Note:** This method may block or fail spuriously.
    /// Further a growth event may not be considered finished in regards of an other `resize` being possible until some time after the call to `resize`.
    ///
    /// Returns `true` if the resize was successfull, or `false` if
    /// it failed. Failure can occur due to allocator exhaustion, thread
    /// contention, or other implementation-specific conditions.
    fn resize(&self, size: usize) -> bool;
}

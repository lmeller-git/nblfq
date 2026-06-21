pub(crate) mod buffer;
mod queue;
#[cfg(all(feature = "pool", feature = "alloc"))]
pub use queue::PooledQueue;
pub use queue::Queue;

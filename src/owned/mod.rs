pub(crate) mod buffer;
mod queue;
#[cfg(feature = "pool")]
pub use queue::PooledQueue;
pub use queue::Queue;

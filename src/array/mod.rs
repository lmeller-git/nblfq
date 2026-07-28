pub(crate) mod buffer;
mod queue;

pub use queue::InlineQueue;
#[cfg(feature = "pool")]
pub use queue::PooledInlineQueue;

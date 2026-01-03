pub(crate) mod buffer;
mod queue;

pub use queue::StaticQueue;

#[cfg(feature = "pool")]
pub use queue::PooledStaticQueue;

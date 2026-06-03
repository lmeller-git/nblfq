pub(crate) mod buffer;
mod queue;

#[cfg(feature = "pool")]
pub use queue::PooledStaticQueue;
pub use queue::StaticQueue;

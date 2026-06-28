#[cfg(feature = "pool")]
mod pooled;
mod queue;

#[cfg(feature = "pool")]
pub use pooled::*;
pub use queue::*;

use crate::owned::buffer::BoxedBuffer;

pub(crate) trait NewSized {
    fn with_size(size: usize) -> Self;
}

impl<T> NewSized for BoxedBuffer<T>
where
    T: Default,
{
    #[track_caller]
    fn with_size(size: usize) -> Self {
        Self::new(size)
    }
}

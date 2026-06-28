use alloc::boxed::Box;

use crate::core::buffer::Buffer;

pub(crate) struct BoxedBuffer<S> {
    inner: Box<[S]>,
}

impl<S: Default> BoxedBuffer<S> {
    #[track_caller]
    pub(crate) fn new(size: usize) -> Self {
        Self {
            inner: (0..size).map(|_| S::default()).collect(),
        }
    }
}

impl<S> Buffer for BoxedBuffer<S> {
    type Slot = S;

    fn capacity(&self) -> usize {
        self.inner.len()
    }

    fn inner(&self) -> &[Self::Slot] {
        &self.inner
    }
}

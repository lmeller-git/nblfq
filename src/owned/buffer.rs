use alloc::boxed::Box;

use crate::{buffer::Buffer, core::slot::Slot};

pub(crate) struct BoxedBuffer<S: Slot> {
    inner: Box<[S]>,
}

impl<S: Slot> BoxedBuffer<S> {
    pub(crate) fn new(size: usize) -> Self {
        Self {
            inner: (0..size).map(|_| S::new()).collect(),
        }
    }
}

impl<S: Slot> Buffer for BoxedBuffer<S> {
    type Slot = S;

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn capacity(&self) -> usize {
        self.len()
    }

    fn inner(&self) -> &[Self::Slot] {
        &self.inner
    }
}

use core::array;

use crate::{buffer::Buffer, core::slot::Slot};

pub(crate) struct ArrayBuf<const N: usize, S: Slot> {
    inner: [S; N],
}

impl<const N: usize, S: Slot> ArrayBuf<N, S> {
    pub fn new() -> Self {
        Self {
            inner: array::from_fn(|_| S::new()),
        }
    }
}

impl<const N: usize, S: Slot> Buffer for ArrayBuf<N, S> {
    type Slot = S;

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn capacity(&self) -> usize {
        N
    }

    fn inner(&self) -> &[Self::Slot] {
        &self.inner
    }
}

use core::array;

use crate::core::buffer::Buffer;

pub(crate) struct ArrayBuf<const N: usize, S> {
    inner: [S; N],
}

impl<const N: usize, S: Default> ArrayBuf<N, S> {
    pub fn new() -> Self {
        Self {
            inner: array::from_fn(|_| S::default()),
        }
    }
}

impl<const N: usize, S> Buffer for ArrayBuf<N, S> {
    type Slot = S;

    fn capacity(&self) -> usize {
        N
    }

    fn inner(&self) -> &[Self::Slot] {
        &self.inner
    }
}

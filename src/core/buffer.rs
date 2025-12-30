use crate::core::slot::Slot;

pub(crate) trait Buffer {
    type Slot: Slot;

    fn len(&self) -> usize;
    fn capacity(&self) -> usize;
    fn inner(&self) -> &[Self::Slot];
}

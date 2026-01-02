pub(crate) mod buffer;
pub(crate) mod queue;
pub(crate) mod slot;
pub use buffer::Buffer;
pub use queue::{ForcePushQueue, MPMCQueue};
pub use slot::PtrLike;

mod sealed {
    #[doc(hidden)]
    pub trait Sealed {}
}

use sealed::Sealed;

#[cfg(all(target_has_atomic = "64", target_endian = "little"))]
pub struct TaggedPtr64;
#[cfg(all(target_has_atomic = "64", target_endian = "little"))]
impl Sealed for TaggedPtr64 {}

#[cfg(target_has_atomic = "128")]
pub struct TaggedPtr128;
#[cfg(target_has_atomic = "128")]
impl Sealed for TaggedPtr128 {}

pub struct Auto;
impl Sealed for Auto {}

#[doc(hidden)]
pub trait SlotType<T: PtrLike>: Sealed {
    #[allow(private_bounds)]
    type Slot: slot::Slot<Item = T>;
}

#[cfg(all(target_has_atomic = "64", target_endian = "little"))]
impl<T: PtrLike> SlotType<T> for TaggedPtr64 {
    type Slot = slot::TaggedPtr64<T>;
}

#[cfg(target_has_atomic = "128")]
impl SlotType for TaggedPtr128 {
    type Slot<T: PtrLike> = slot::TaggedPtr128<T>;
}

impl<T: PtrLike> SlotType<T> for Auto {
    #[cfg(all(
        target_has_atomic = "64",
        target_endian = "little",
        not(target_has_atomic = "128")
    ))]
    type Slot = slot::TaggedPtr64<T>;
    #[cfg(target_has_atomic = "128")]
    type Slot = slot::TaggedPtr128<T>;
    #[cfg(all(
        any(not(target_has_atomic = "64"), not(target_endian = "little")),
        not(target_has_atomic = "128")
    ))]
    compile_error!("target arch is currently not supported");
}

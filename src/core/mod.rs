pub(crate) mod buffer;
pub(crate) mod queue;
pub(crate) mod slot;
pub use buffer::Buffer;
pub use queue::{ForcePushQueue, MPMCQueue};
pub use slot::PtrLike;

use crate::{cfg_taggedptr64, cfg_taggedptr128, utils::Sealed};

cfg_taggedptr64! {
    pub use tagged64::*;
mod tagged64 {
    use super::*;
    pub struct TaggedPtr64;
    impl Sealed for TaggedPtr64 {}
    impl<T: PtrLike> SlotType<T> for TaggedPtr64 {
        type Slot = slot::TaggedPtr64<T>;
    }
}
}

cfg_taggedptr128! {
    pub use tagged128::*;
mod tagged128 {
    use super::*;
    pub struct TaggedPtr128;
    impl Sealed for TaggedPtr128 {}
        impl<T: PtrLike> SlotType<T> for TaggedPtr128 {
            type Slot = slot::TaggedPtr128<T>;
        }

}
}

pub struct Auto;
impl Sealed for Auto {}

#[doc(hidden)]
pub trait SlotType<T: PtrLike>: Sealed {
    #[allow(private_bounds)]
    type Slot: slot::Slot<Item = T>;
}

impl<T: PtrLike> SlotType<T> for Auto {
    #[cfg(all(
        any(target_has_atomic = "64", feature = "atomic-fallback"),
        target_endian = "little",
        not(target_has_atomic = "128")
    ))]
    type Slot = slot::TaggedPtr64<T>;
    #[cfg(any(
        target_has_atomic = "128",
        all(feature = "atomic-fallback", not(target_endian = "little"))
    ))]
    type Slot = slot::TaggedPtr128<T>;
    #[cfg(all(
        any(
            not(any(target_has_atomic = "64", feature = "atomic-fallback")),
            not(target_endian = "little")
        ),
        not(target_has_atomic = "128")
    ))]
    compile_error!("target arch is currently not supported");
}

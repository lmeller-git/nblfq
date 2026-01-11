//! Core traits and types for interface with the queues in this crate.
//!
//! This module contains functionality, which interacts with the underlying implementations of queues.
//! For most use cases it should not be neccessary to use any of this functionality.

pub(crate) mod buffer;
pub(crate) mod packed;
pub(crate) mod queue;
pub(crate) mod slot;

pub use packed::{AsPackedValue, NonZeroTruncatedU64};

pub mod slots {
    //! Module containing types used to determine the underlying storage type in nblf-queue Queues.
    //! In most cases the `Auto` type, which is used as default across this crate, should suffice.

    use super::*;
    use crate::utils::Sealed;

    cfg_taggedptr64! {
        pub use tagged64::*;
        mod tagged64 {
            use super::*;
            /// Slot type describing a tagged 64 bit pointer.
            /// Only available if `target_has_atomic = "64"` is true or on feature `atomic-fallback`.
            pub struct TaggedPtr64;
            impl Sealed for TaggedPtr64 {}
            impl<T: AsPackedValue> SlotType<T> for TaggedPtr64 {
                type Slot = slot::TaggedPtr64<T>;
            }
        }
    }

    cfg_taggedptr128! {
        pub use tagged128::*;
        mod tagged128 {
            use super::*;
            /// Slot type describing a tagged 128 bit pointer.
            /// Only available if `target_has_atomic = "128"` is true or on feature `atomic-fallback`.
            pub struct TaggedPtr128;
            impl Sealed for TaggedPtr128 {}
            impl<T: AsPackedValue> SlotType<T> for TaggedPtr128 {
                type Slot = slot::TaggedPtr128<T>;
            }

        }
    }

    /// Slot type which chooses a concrete implementation based on arch and feature flags.
    pub struct Auto;
    impl Sealed for Auto {}

    #[doc(hidden)]
    pub trait SlotType<T: AsPackedValue>: Sealed {
        #[allow(private_bounds)]
        type Slot: slot::Slot<Item = T>;
    }

    impl<T: AsPackedValue> SlotType<T> for Auto {
        #[cfg(all(
            any(target_has_atomic = "64", feature = "atomic-fallback"),
            not(target_has_atomic = "128")
        ))]
        type Slot = slot::TaggedPtr64<T>;
        #[cfg(any(
            target_has_atomic = "128",
            all(feature = "atomic-fallback", not(target_endian = "little"))
        ))]
        type Slot = slot::TaggedPtr128<T>;
        #[cfg(all(
            not(any(target_has_atomic = "64", feature = "atomic-fallback")),
            not(target_has_atomic = "128")
        ))]
        compile_error!("target arch is currently not supported");
    }
}

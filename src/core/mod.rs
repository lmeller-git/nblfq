//! Core traits and types for interface with the queues in this crate.
//!
//! This module contains functionality which interacts with the underlying implementations of queues.
//! For most use cases it should not be neccessary to use any of this functionality.

pub(crate) mod buffer;
pub(crate) mod packed;
pub(crate) mod queue;
pub(crate) mod slot;

pub use packed::{AsPackedValue, TruncatedU64};

pub mod slots {
    //! Module containing types used to determine the underlying storage type in nblf-queue Queues.
    //! In most cases the `Auto` type, which is used as default across this crate, should suffice.

    use super::*;
    use crate::utils::Sealed;

    cfg_atomic_tagged64! {
        pub use tagged64::*;
        mod tagged64 {
            use super::*;
            /// Slot type describing a tagged 64 bit value.
            /// Only available if `target_has_atomic = "64"` is true or on feature `atomic-fallback`.
            pub struct Tagged64;
            impl Sealed for Tagged64 {}
            impl<T: AsPackedValue> SlotType<T> for Tagged64 {
                type Slot = slot::Tagged64<T>;
            }
        }
    }

    cfg_atomic_tagged128! {
        pub use tagged128::*;
        mod tagged128 {
            use super::*;
            /// Slot type describing a tagged 128 bit value.
            /// Only available if `target_has_atomic = "128"` is true or on feature `atomic-fallback`.
            pub struct Tagged128;
            impl Sealed for Tagged128 {}
            impl<T: AsPackedValue> SlotType<T> for Tagged128 {
                type Slot = slot::Tagged128<T>;
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
        #[cfg(all(not(target_has_atomic = "128"), target_has_atomic = "64"))]
        type Slot = slot::Tagged64<T>;
        #[cfg(any(
            target_has_atomic = "128",
            all(not(target_has_atomic = "64"), feature = "atomic-fallback")
        ))]
        type Slot = slot::Tagged128<T>;

        #[cfg(all(
            not(target_has_atomic = "128"),
            not(target_has_atomic = "64"),
            not(feature = "atomic-fallback")
        ))]
        compile_error!("target arch is currently not supported");
    }
}

#[cfg(feature = "pool")]
pub mod pool_storage {
    use lf_slots::{RawStorage, StorageData};

    pub trait InlineSlotStore<const N: usize> {
        type Storage: RawStorage + StorageData + Default;
    }

    macro_rules! impl_inline_slot_store {
        ($($n:expr),* $(,)?) => {
            $(
                impl InlineSlotStore<$n> for super::slots::Auto {
                    type Storage = lf_slots::core::InlineStorage<$n, { lf_slots::core::full_shard_count($n) }>;
                }
            )*
        };
    }

    impl_inline_slot_store!(
        2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536
    );

    #[macro_export]
    macro_rules! impl_pool_capacity {
        ($name:ident, $n:expr) => {
            pub struct $name;
            impl $crate::InlineSlotStore<$n> for $name {
                type Storage = lf_slots::core::InlineStorage<$n, { lf_slots::core::full_shard_count($n, 512) };
            }
        };
    }
}

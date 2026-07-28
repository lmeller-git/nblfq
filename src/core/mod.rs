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
    //! In most cases the [`Auto`] type, which is used as default across this crate, should suffice.

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
#[macro_use]
pub mod inline_pool_storage {
    //! This module contains functionality to declare statically sized pools.

    use lf_slots::{SlotPoolMeta, core::RawSlotPool};

    use crate::core::{AsPackedValue, slots::SlotType};

    /// The type of slot pool and queue slot associated with some marker type.
    ///
    /// The slot pool implements the traits
    /// `RawSlotPool`, `SlotPoolMeta` and `Default` from the crate `lf-slots`.
    ///
    /// The slot implements the trait `SlotType` as declared in `nblf_queue::core::slots::SlotType`.
    pub trait InlineSlotStore<T: AsPackedValue, const N: usize> {
        /// The slot pool type associated with this type.
        ///
        /// The index pool is used to distribute space for items.
        type Pool: RawSlotPool + SlotPoolMeta + Default;
        /// The queue slot type associate with this type.
        type SlotType: SlotType<T>;
    }

    macro_rules! impl_inline_slot_store {
        ($($n:expr),* $(,)?) => {
            $(
                impl<T: $crate::core::AsPackedValue> InlineSlotStore<T, $n> for $crate::core::slots::Auto {
                    type Pool = lf_slots::batched::WordPool<
                        lf_slots::InlineSlots<
                            { $n * lf_slots::core::Word::BITS as usize },
                            { lf_slots::core::shard_count($n * lf_slots::core::Word::BITS as usize, lf_slots::core::words_per_shard($n * lf_slots::core::Word::BITS as usize)) },
                            { lf_slots::core::words_per_shard($n * lf_slots::core::Word::BITS as usize) },
                        >
                    >;
                    type SlotType = $crate::core::slots::Auto;
                }
            )*
        };
    }

    impl_inline_slot_store!(
        2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536
    );

    /// Defines a new storage layout of size `n`.
    ///
    /// Usage:
    ///
    /// ```rust
    /// use nblf_queue::{impl_pool_capacity, PooledInlineQueue};
    ///
    /// impl_pool_capacity!(Storage42, 42);
    ///
    /// _ = PooledInlineQueue::<(), 42, Storage42>::with_conf();
    ///
    /// ```
    #[macro_export]
    macro_rules! impl_pool_capacity {
        ($vis:vis $name:ident, $n:expr, $slot:path) => {
            $vis struct $name;
            impl<T: $crate::core::AsPackedValue>
                $crate::core::inline_pool_storage::InlineSlotStore<T, $n> for $name
            {
                type SlotType = $slot;
                type Pool = lf_slots::batched::WordPool<
                    lf_slots::InlineSlots<
                        { $n * lf_slots::core::Word::BITS as usize },
                        { lf_slots::core::shard_count($n * lf_slots::core::Word::BITS as usize, lf_slots::core::words_per_shard($n * lf_slots::core::Word::BITS as usize)) },
                        { lf_slots::core::words_per_shard($n * lf_slots::core::Word::BITS as usize) },
                    >
                >;
            }
        };
        ($vis:vis $name:ident, $n:expr) => {
            $crate::impl_pool_capacity!($vis $name, $n, $crate::core::slots::Auto)
        };
    }
}

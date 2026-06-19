use crate::core::{AsPackedValue, TruncatedU64};

cfg_atomic_tagged128! {
    pub use tagged_ptr_u128_portable::*;
}
cfg_atomic_tagged64! {
    pub use tagged_ptr64::*;
}

pub(crate) trait Slot: Default {
    type Item;
    type Storage: Copy;
    const MAX_W: u64;
    const EMPTY_VALUE: Self::Storage;
    const MAX_CARGO_BIT_WIDTH: usize;

    fn new() -> Self;
    fn components(&self) -> SlotComponents<Self>;
    fn cmpxchg(
        &self,
        old: SlotComponents<Self>,
        item: Option<Self::Item>,
        new_count: u64,
    ) -> Result<Option<Self::Item>, Option<Self::Item>>;
    fn is_empty(components: Self::Storage) -> bool;
    fn extract_count(value: Self::Storage) -> u64;
    fn put_count(container: Self::Storage, count: u64) -> Self::Storage;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotComponents<S>
where
    S: Slot,
{
    value: S::Storage,
}

impl<S> SlotComponents<S>
where
    S: Slot,
    S::Storage: Copy,
{
    fn new(value: S::Storage) -> Self {
        Self { value }
    }

    pub(crate) fn raw(&self) -> S::Storage {
        self.value
    }

    pub(crate) fn get_count(&self) -> u64 {
        S::extract_count(self.value)
    }

    pub(crate) fn put_count(&mut self, count: u64) {
        self.value = S::put_count(self.value, count)
    }

    pub(crate) fn set_empty(&mut self) {
        let new = S::EMPTY_VALUE;
        self.value = S::put_count(new, S::extract_count(self.value));
    }
}

cfg_atomic_tagged64! {
    mod tagged_ptr64 {
        use core::marker::PhantomData;

        use crate::sync::atomic::{AtomicU64, Ordering};

        use super::*;

        const MAX_CARGO_BIT_WIDTH: usize = 48;
        const NON_COUNT_BITS: usize = MAX_CARGO_BIT_WIDTH + 1;

        // this slot stores the item in a tagged U64 value.
        // `count` takes up the upper 15 bits and `item` takes up the lower 48 bits.
        // this leaves 1 bit of state, which is used to encode `empty` vs `full`

        pub struct Tagged64<T: AsPackedValue> {
            state: AtomicU64,
            _data: PhantomData<T>,
        }

        impl<T: AsPackedValue> Slot for Tagged64<T> {
            type Item = T;
            type Storage = u64;
            const MAX_W: u64 = u16::MAX as u64 / 2 + 1;
            const EMPTY_VALUE: Self::Storage = 0;
            const MAX_CARGO_BIT_WIDTH: usize = MAX_CARGO_BIT_WIDTH;

            fn new() -> Self {
                const {
                    assert!(
                        Self::MAX_CARGO_BIT_WIDTH >= T::MIN_BIT_WIDTH,
                        "the stored item must be representable with 48 or less bits"
                    )
                };

                if Self::MAX_CARGO_BIT_WIDTH < size_of::<T>() * 8 && !<T as AsPackedValue>::is_rt_safe() {
                    panic!("Trying to store a type that is not encodeable in a packed 64bit slot (48 bits) is unsafe");
                }

                Self {
                    state: AtomicU64::new(0),
                    _data: PhantomData,
                }
            }

            fn components(&self) -> SlotComponents<Self> {
                SlotComponents::new(self.state.load(Ordering::Acquire))
            }

            fn cmpxchg(
                &self,
                old: SlotComponents<Self>,
                new_value: Option<T>,
                new_count: u64,
            ) -> Result<Option<T>, Option<T>> {
                let new_trunc = new_value.map(|v| AsPackedValue::encode(v));
                let new = pack!((new_count, new_trunc.map_or(Self::EMPTY_VALUE, |v| v.read() | (1 << (NON_COUNT_BITS - 1)))): NON_COUNT_BITS);

                self.state
                    .compare_exchange(
                        old.raw(),
                        new,
                        core::sync::atomic::Ordering::AcqRel,
                        core::sync::atomic::Ordering::Relaxed,
                    )
                    .map(|cargo| {
                        (!Self::is_empty(cargo)).then(||
                            // Safety:
                            // we just checked that a value is contained in cargo. This value is decoded only once, here
                            unsafe { AsPackedValue::decode(TruncatedU64::new(cargo)) },
                        )
                    })
                    .map_err(|_| {
                        new_trunc.map(|v| {
                            // Safety:
                            // this value got passed in new_value
                            unsafe { AsPackedValue::decode(v) }
                        })
                    })
            }

            fn is_empty(components: Self::Storage) -> bool {
                components & (1 << (NON_COUNT_BITS - 1)) == 0
            }

            fn extract_count(value: Self::Storage) -> u64 {
                unpack!((value): NON_COUNT_BITS).0
            }

            fn put_count(container: Self::Storage, count: u64) -> Self::Storage {
                pack!((count, unpack!((container): NON_COUNT_BITS).1): NON_COUNT_BITS)
            }
        }

        impl<T: AsPackedValue> Drop for Tagged64<T> {
            fn drop(&mut self) {
                let components = self.components();
                let _cargo: Option<T> = (!Self::is_empty(components.raw())).then(||
                    // Safety:
                    // we just checked that we have a stored item.
                    // this item is decoded the once and dropped
                    unsafe { AsPackedValue::decode(TruncatedU64::new(components.raw())) },
                );
            }
        }

        impl<T: AsPackedValue> Default for Tagged64<T> {
            fn default() -> Self {
                Self::new()
            }
        }

        // SAFETY:
        // TaggedPtr<T> is essentially a version of a type implementing PtrLike. It should have the same Send + Sync.
        unsafe impl<T: AsPackedValue + Send> Send for Tagged64<T> {}
        // SAFETY:
        // TaggedPtr<T> is essentially a version of a type implementing PtrLike. It should have the same Send + Sync.
        unsafe impl<T: AsPackedValue + Sync> Sync for Tagged64<T> {}
    }
}

cfg_atomic_tagged128! {
    mod tagged_ptr_u128_portable {
        use core::marker::PhantomData;

        use crate::sync::atomic::{AtomicU128, Ordering};

        use super::*;

        const MAX_CARGO_BIT_WIDTH: usize = 64;
        const NON_COUNT_BITS: usize = MAX_CARGO_BIT_WIDTH + 1;

        // this slot stores the item in a tagged U128 value.
        // `count` takes up the upper 63 bits and `item` takes up the lower 64 bits.
        // this leaves 1 bit of state, which is used to encode `empty` vs `full`

        pub struct Tagged128<T: AsPackedValue> {
            storage: AtomicU128,
            _data: PhantomData<T>,
        }

        impl<T: AsPackedValue> Slot for Tagged128<T> {
            type Item = T;
            type Storage = u128;
            const MAX_W: u64 = u64::MAX / 2; // artificially set MAX_W low, to ensure it does not overlfow
            const EMPTY_VALUE: Self::Storage = 0;
            const MAX_CARGO_BIT_WIDTH: usize = MAX_CARGO_BIT_WIDTH;

            fn new() -> Self {
                const {
                    assert!(
                        Self::MAX_CARGO_BIT_WIDTH >= T::MIN_BIT_WIDTH,
                        "the stored item must be representable with 64 or less bits"
                    )
                };

                if Self::MAX_CARGO_BIT_WIDTH < size_of::<T>() * 8 && !<T as AsPackedValue>::is_rt_safe() {
                    panic!("Trying to store a type that is not encodeable in a packed 128bit slot (64 bits) is unsafe");
                }

                Self {
                    storage: AtomicU128::new(0),
                    _data: PhantomData,
                }
            }

            fn components(&self) -> SlotComponents<Self> {
                SlotComponents::new(self.storage.load(Ordering::Acquire))
            }

            fn cmpxchg(
                &self,
                old: SlotComponents<Self>,
                item: Option<Self::Item>,
                new_count: u64,
            ) -> Result<Option<Self::Item>, Option<Self::Item>> {
                let new_trunc = item.map(|v| AsPackedValue::encode(v));
                let new = pack!((new_count as u128, new_trunc.map_or(Self::EMPTY_VALUE, |v| v.read() as u128 | (1 << (NON_COUNT_BITS - 1)))): NON_COUNT_BITS);

                self.storage
                    .compare_exchange(
                        old.raw(),
                        new,
                        core::sync::atomic::Ordering::AcqRel,
                        core::sync::atomic::Ordering::Relaxed,
                    )
                    .map(|cargo| {
                        (!Self::is_empty(cargo)).then(||
                            // Safety:
                            // we just checked that cargo is not empty.
                            // we can simply truncate to u64, since the whole item lives in the lower 64 bits
                            unsafe { AsPackedValue::decode(TruncatedU64::new(cargo as u64)) },
                        )
                    })
                    .map_err(|_| {
                        new_trunc.map(|v| {
                            // Safety:
                            // we got passed this value
                            unsafe { AsPackedValue::decode(v) }
                        })
                    })
            }

            fn is_empty(components: Self::Storage) -> bool {
                components & (1 << (NON_COUNT_BITS - 1)) == 0
            }

            fn extract_count(value: Self::Storage) -> u64 {
                unpack!((value): NON_COUNT_BITS).0 as u64
            }

            fn put_count(container: Self::Storage, count: u64) -> Self::Storage {
                pack!((count as u128, unpack!((container): NON_COUNT_BITS).1): NON_COUNT_BITS)
            }
        }

        impl<T: AsPackedValue> Drop for Tagged128<T> {
            fn drop(&mut self) {
                let components = self.components();
                let _cargo: Option<T> = (!Self::is_empty(components.raw())).then(||
                    // Safety:
                    // we juts checked that an item is stored.
                    // we decode this once and drop it.
                    // we can truncate to u64, since item is stored in the lower 64 bits
                    unsafe { AsPackedValue::decode(TruncatedU64::new(components.raw() as u64)) },
                );
            }
        }

        impl<T: AsPackedValue> Default for Tagged128<T> {
            fn default() -> Self {
                Self::new()
            }
        }

        // SAFETY:
        // TaggedPtr<T> is essentially a version of a type implementing PtrLike. It should have the same Send + Sync.
        unsafe impl<T: AsPackedValue + Send> Send for Tagged128<T> {}
        // SAFETY:
        // TaggedPtr<T> is essentially a version of a type implementing PtrLike. It should have the same Send + Sync.
        unsafe impl<T: AsPackedValue + Sync> Sync for Tagged128<T> {}
    }
}

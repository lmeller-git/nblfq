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

    #[allow(dead_code)]
    pub(crate) fn with_count<F>(&mut self, mut f: F)
    where
        F: FnMut(u64) -> u64,
    {
        self.put_count(f(self.get_count()));
    }

    pub(crate) fn set_empty(&mut self) {
        let new = S::EMPTY_VALUE;
        self.value = S::put_count(new, S::extract_count(self.value));
    }
}

// TODO
// use more bits of the tagged value for count, based on Item::MIN_BITS

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

#[cfg(false)]
pub use item_slot::*;

#[cfg(false)]
// TODO fix this. This currently livelocks/(deadlocks?) in mpmc_ringbuffer test
mod item_slot {
    use super::*;
    use crate::utils::{components_as_num, components_from_num};

    use core::{
        cell::UnsafeCell,
        sync::atomic::{AtomicU64, Ordering},
    };

    // state transition:
    // empty: [EMPTY_PTR | COUNT], data empty
    // on push: cmpxchg(empty, full) ->
    // [EMPTY_PTR -> CONTESTED | COUNT -> NEWCOUNT] ->
    // data empty -> full ->
    // [CONTESTED -> FULL | NEWCOUNT]
    // full: [FULL | COUNT]
    // on pop: cmpxchg(old, empty) ->
    // [FULL_PTR -> CONTESTED | NEWCOUNT] ->
    // take data
    // [CONTESTED -> EMPTY | NEWCOUNT]

    const EMPTY: usize = 0b000;
    const RESERVED_PUSH: usize = 0b001;
    const RESERVED_POP: usize = 0b010;
    const FULL: usize = 0b100;
    const STATE_MASK: usize = 0b111;

    // instead of a ptr we store a state bimask, which we can use to determine contested states
    // TODO we do not need to do stuff like sign extension, as we do not use an actual ptr
    pub(crate) struct OwnedSlot<T> {
        state: AtomicU64,
        data: UnsafeCell<Option<T>>,
    }

    impl<T> Slot for OwnedSlot<T> {
        type Item = T;
        const MAX_BITS: usize = usize::MAX;
        const MAX_W: u64 = u16::MAX as u64 + 1; // TODO the max count could be higher, as only 2 bits are used for state, but this would require new retrieval functions
        const EMPTY_PTR: *const Self::Item = EMPTY as *const Self::Item; // TODO this is not actually a ptr to Self::Item, but a state mask

        fn new() -> Self {
            Self {
                state: AtomicU64::new(EMPTY as u64),
                data: UnsafeCell::new(None),
            }
        }

        fn components(&self) -> SlotComponents {
            let (count, state) = components_from_num(self.state.load(Ordering::Acquire));
            SlotComponents {
                count,
                state: state,
            }
        }

        fn cmpxchg(
            &self,
            old_ptr: *const Self::Item,
            old_count: u64,
            item: Option<Self::Item>,
            new_count: u64,
        ) -> Result<Option<Self::Item>, Option<Self::Item>> {
            // check validity of request:
            // a cmpxchg without payload on an empty state is not allowed,
            // as is a cmpxchg with payload on full state,
            // as is a cmpxchg on a RESERVED slot
            if old_ptr as usize == RESERVED_POP
                || old_ptr as usize == RESERVED_PUSH
                || item.is_some() && old_ptr as usize == FULL
                || item.is_none() && old_ptr as usize == EMPTY
            {
                return Err(item);
            }

            // if there is a pyaload, assume to be in push, else assume pop
            let reserved = if item.is_some() {
                RESERVED_PUSH
            } else {
                RESERVED_POP
            };

            let contested_state = components_as_num(new_count, reserved as u64);
            let old_sate = components_as_num(old_count, old_ptr as u64);

            if self
                .state
                .compare_exchange(
                    old_sate,
                    contested_state,
                    Ordering::Acquire,
                    Ordering::Relaxed,
                )
                .is_err()
            {
                return Err(item);
            }

            // SAFETY we just ensured via the RESERVED state that only one concurrent acces to self.data happens. Any other thread will fail the above cas.
            let data = unsafe { &mut *self.data.get() };
            let old_item = data.take();
            *data = item;

            // now do a second atomic store to publish the item/the empty slot
            let new_state = if data.is_some() { FULL } else { EMPTY };
            let final_state = components_as_num(new_count, new_state as u64);

            self.state.store(final_state, Ordering::Release);

            Ok(old_item)
        }

        fn is_empty(state: u64) -> bool {
            state as usize == EMPTY
        }
    }

    impl<T> Default for OwnedSlot<T> {
        fn default() -> Self {
            Self::new()
        }
    }

    unsafe impl<T: Send> Send for OwnedSlot<T> {}
    unsafe impl<T: Sync> Sync for OwnedSlot<T> {}
}

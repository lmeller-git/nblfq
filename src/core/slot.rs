use ::core::ptr::NonNull;

cfg_taggedptr128! {
    pub use tagged_ptr_u128_portable::*;
}
cfg_taggedptr64! {
    pub use tagged_ptr64::*;
}
/// This trait allows the type to be stored in a TaggedPtr.
/// SAFETY:
/// - Since `PtrLike::as_ptr` and `PtrLike::from_raw` may be called during any queue operation, both must be atomic and wait-free.
/// - `PtrLike::as_ptr` should never return a nullptr, as the nullptr is reserved for empty slots.
/// - `TaggedPtr64` will truncate the ptr handed out from `PtrLike::as_ptr` to 48 bits and later cal `PtrLike::from_ptr` on the sign extended version of this, thus your pointer must fit into 48 bits.
/// - The ptr handed out in `PtrLike::as_ptr` must be stable and valid for at least as long it is stored in the queue, i.e. the queues lifetime
pub unsafe trait PtrLike: Sized {
    type Item;
    fn as_ptr(zelf: Self) -> NonNull<Self::Item>;
    fn from_raw(raw: NonNull<Self::Item>) -> Self;
}

unsafe impl<T> PtrLike for *const T {
    type Item = T;
    fn as_ptr(zelf: Self) -> NonNull<T> {
        NonNull::new(zelf as *mut T).expect("tried to store a nullptr in queue. This is UB")
    }

    fn from_raw(raw: NonNull<Self::Item>) -> Self {
        raw.as_ptr()
    }
}

unsafe impl<T> PtrLike for *mut T {
    type Item = T;
    fn as_ptr(zelf: Self) -> NonNull<T> {
        NonNull::new(zelf).expect("tried to store a nullptr in queue. This is UB")
    }

    fn from_raw(raw: NonNull<Self::Item>) -> Self {
        raw.as_ptr()
    }
}

unsafe impl<T> PtrLike for NonNull<T> {
    type Item = T;
    fn as_ptr(zelf: Self) -> NonNull<Self::Item> {
        zelf
    }

    fn from_raw(raw: NonNull<Self::Item>) -> Self {
        raw
    }
}

unsafe impl<T> PtrLike for &'static T {
    type Item = T;
    fn as_ptr(zelf: Self) -> NonNull<T> {
        NonNull::from_ref(zelf)
    }

    fn from_raw(raw: NonNull<T>) -> Self {
        unsafe { raw.as_ref() }
    }
}

pub(crate) trait Slot: Default {
    type Item;
    // TODO do some validation that PtrLike fits into this
    #[allow(unused)]
    const MAX_BITS: usize;
    const MAX_W: u64;
    const EMPTY_PTR: *const Self::Item;

    fn new() -> Self;
    fn components(&self) -> SlotComponents;
    fn cmpxchg(
        &self,
        old_ptr: *const Self::Item,
        old_count: u64,
        item: Option<Self::Item>,
        new_count: u64,
    ) -> Result<Option<Self::Item>, Option<Self::Item>>;
    fn is_empty(state: u64) -> bool;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotComponents {
    pub count: u64,
    pub state: u64,
}

impl From<(u64, u64)> for SlotComponents {
    fn from(value: (u64, u64)) -> Self {
        Self {
            count: value.0,
            state: value.1,
        }
    }
}

// TODO currently all tagged ptrs, ... assume little-endian architecture. This should be validated/ support for big-endian
// TODO TaggedPtr64 should be feature gated by target_has_atomic = "64" and maybe ptr-size
cfg_taggedptr64! {
mod tagged_ptr64 {
    use core::{
        marker::PhantomData,
        ptr::{null, null_mut},
        sync::atomic::Ordering,
    };

    use portable_atomic::AtomicU64;

    use crate::utils::{components_as_tagged, components_from_tagged};

    use super::*;

    pub struct TaggedPtr64<T: PtrLike> {
        ptr: AtomicU64,
        _data: PhantomData<T>,
    }

    impl<T: PtrLike> Slot for TaggedPtr64<T> {
        type Item = T;
        const MAX_BITS: usize = 48;
        const MAX_W: u64 = u16::MAX as u64 + 1;
        const EMPTY_PTR: *const Self::Item = null();

        fn new() -> Self {
            Self {
                ptr: AtomicU64::new(0),
                _data: PhantomData,
            }
        }

        fn components(&self) -> SlotComponents {
            let (c, p) = components_from_tagged::<T::Item>(self.ptr.load(Ordering::Acquire));
            (c, p as u64).into()
        }

        fn cmpxchg(
            &self,
            old_ptr: *const T,
            old_count: u64,
            new_ptr: Option<T>,
            new_count: u64,
        ) -> Result<Option<T>, Option<T>> {
            let new_ptr_ = new_ptr.map_or(null_mut(), |p| PtrLike::as_ptr(p).as_ptr());
            let new_state = components_as_tagged(new_count, new_ptr_);
            let old_state = components_as_tagged(old_count, old_ptr);
            match self.ptr.compare_exchange(
                old_state,
                new_state,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(v) => {
                    let nonnull =
                        NonNull::new(components_from_tagged::<T::Item>(v).1 as *mut T::Item);
                    Ok(nonnull.map(|ptr| PtrLike::from_raw(ptr)))
                }
                Err(_) => {
                    let nonnull = NonNull::new(
                        components_from_tagged::<T::Item>(new_ptr_ as u64).1 as *mut T::Item,
                    );
                    Err(nonnull.map(|ptr| PtrLike::from_raw(ptr)))
                }
            }
        }

        fn is_empty(ptr: u64) -> bool {
            (ptr as *const T::Item).is_null()
        }
    }

    impl<T: PtrLike> Drop for TaggedPtr64<T> {
        fn drop(&mut self) {
            let components = self.components();
            if let Some(ptr) = NonNull::new(components.state as *mut T::Item) {
                let _ptr = T::from_raw(ptr);
            }
        }
    }

    impl<T: PtrLike> Default for TaggedPtr64<T> {
        fn default() -> Self {
            Self::new()
        }
    }

    unsafe impl<T: PtrLike + Send> Send for TaggedPtr64<T> {}
    unsafe impl<T: PtrLike + Sync> Sync for TaggedPtr64<T> {}
}
}

cfg_taggedptr128! {
mod tagged_ptr_u128_portable {
    use super::*;
    use crate::utils::{components_as_u128, components_from_u128};

    use portable_atomic::AtomicU128;

    use core::{
        marker::PhantomData,
        ptr::{null, null_mut},
        sync::atomic::Ordering,
    };

    pub struct TaggedPtr128<T: PtrLike> {
        storage: AtomicU128,
        _data: PhantomData<T>,
    }

    impl<T: PtrLike> TaggedPtr128<T> {
        pub(crate) fn from_u128(value: u128) -> Self {
            Self {
                storage: AtomicU128::new(value),
                _data: PhantomData,
            }
        }
    }

    impl<T: PtrLike> Slot for TaggedPtr128<T> {
        type Item = T;
        const MAX_BITS: usize = 64; // techincally we could use more here, as the counter does not use the full 64 bits currently
        const MAX_W: u64 = u64::MAX / 2; // artificially set MAX_W low, to ensure it does not overlfow
        const EMPTY_PTR: *const Self::Item = null();

        fn new() -> Self {
            Self::from_u128(0)
        }

        fn components(&self) -> SlotComponents {
            let (c, p) = components_from_u128::<T::Item>(self.storage.load(Ordering::Acquire));
            (c, p as u64).into()
        }

        fn cmpxchg(
            &self,
            old_ptr: *const Self::Item,
            old_count: u64,
            item: Option<Self::Item>,
            new_count: u64,
        ) -> Result<Option<Self::Item>, Option<Self::Item>> {
            let new_ptr = item.map_or(null_mut(), |ptr| PtrLike::as_ptr(ptr).as_ptr());
            let old = components_as_u128(old_count, old_ptr);
            let new = components_as_u128(new_count, new_ptr);
            match self
                .storage
                .compare_exchange(old, new, Ordering::AcqRel, Ordering::Relaxed)
            {
                Ok(v) => {
                    let nonnull =
                        NonNull::new(components_from_u128::<T::Item>(v).1 as *mut T::Item);
                    Ok(nonnull.map(|ptr| PtrLike::from_raw(ptr)))
                }
                Err(_) => {
                    let nonnull = NonNull::new(
                        components_from_u128::<T::Item>(new_ptr as u128).1 as *mut T::Item,
                    );
                    Err(nonnull.map(|ptr| PtrLike::from_raw(ptr)))
                }
            }
        }

        fn is_empty(ptr: u64) -> bool {
            (ptr as *const T::Item).is_null()
        }
    }

    impl<T: PtrLike> Drop for TaggedPtr128<T> {
        fn drop(&mut self) {
            let components = self.components();
            if let Some(ptr) = NonNull::new(components.state as *mut T::Item) {
                let _ptr = T::from_raw(ptr);
            }
        }
    }

    impl<T: PtrLike> Default for TaggedPtr128<T> {
        fn default() -> Self {
            Self::new()
        }
    }

    unsafe impl<T: PtrLike + Send> Send for TaggedPtr128<T> {}
    unsafe impl<T: PtrLike + Sync> Sync for TaggedPtr128<T> {}
}
}
#[cfg(false)]
pub use item_slot::*;

use crate::{cfg_taggedptr64, cfg_taggedptr128};
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

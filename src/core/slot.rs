use ::core::ptr::NonNull;

use portable_atomic::cfg_has_atomic_128;
#[cfg(all(feature = "tagged-ptr", target_has_atomic = "64"))]
pub use tagged_ptr64::*;

pub unsafe trait PtrLike: Sized {
    type Item;
    fn as_ptr(zelf: Self) -> *mut Self::Item;
    fn from_raw(raw: *mut Self::Item) -> Option<Self>;
}

unsafe impl<T> PtrLike for *const T {
    type Item = T;
    fn as_ptr(zelf: Self) -> *mut T {
        zelf as *mut T
    }

    fn from_raw(raw: *mut Self::Item) -> Option<Self> {
        Some(raw)
    }
}

unsafe impl<T> PtrLike for *mut T {
    type Item = T;
    fn as_ptr(zelf: Self) -> *mut T {
        zelf
    }

    fn from_raw(raw: *mut Self::Item) -> Option<Self> {
        Some(raw as *mut T)
    }
}

unsafe impl<T> PtrLike for NonNull<T> {
    type Item = T;
    fn as_ptr(zelf: Self) -> *mut Self::Item {
        zelf.as_ptr()
    }

    fn from_raw(raw: *mut Self::Item) -> Option<Self> {
        NonNull::new(raw as *mut T)
    }
}

unsafe impl<T> PtrLike for &'static T {
    type Item = T;
    fn as_ptr(zelf: Self) -> *mut T {
        zelf as *const T as *mut T
    }

    fn from_raw(raw: *mut T) -> Option<Self> {
        if raw.is_null() {
            None
        } else {
            Some(unsafe { &*raw })
        }
    }
}

pub trait Slot {
    type Item;
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
    fn is_contested(state: u64) -> bool;
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
#[cfg(all(feature = "tagged-ptr", target_has_atomic = "64"))]
mod tagged_ptr64 {
    use core::{
        marker::PhantomData,
        ptr::{null, null_mut},
        sync::atomic::{AtomicU64, Ordering},
    };

    use crate::utils::{components_as_tagged, components_from_tagged};

    use super::*;

    pub struct TaggedPtr64<T: PtrLike> {
        ptr: AtomicU64,
        _data: PhantomData<T>,
    }

    impl<T: PtrLike> Slot for TaggedPtr64<T> {
        type Item = T;
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
            let new_ptr_ = new_ptr.map_or(null_mut(), |p| PtrLike::as_ptr(p));
            let new_state = components_as_tagged(new_count, new_ptr_);
            let old_state = components_as_tagged(old_count, old_ptr);
            self.ptr
                .compare_exchange(old_state, new_state, Ordering::AcqRel, Ordering::Relaxed)
                .map(|ptr| {
                    PtrLike::from_raw(components_from_tagged::<T::Item>(ptr).1 as *mut T::Item)
                })
                .map_err(|_| PtrLike::from_raw(new_ptr_))
        }

        fn is_empty(ptr: u64) -> bool {
            (ptr as *const T::Item).is_null()
        }

        fn is_contested(_: u64) -> bool {
            false
        }
    }

    impl<T: PtrLike> Drop for TaggedPtr64<T> {
        fn drop(&mut self) {
            let components = self.components();
            let _ptr = T::from_raw(components.state as *mut T::Item);
        }
    }

    unsafe impl<T: PtrLike + Send> Send for TaggedPtr64<T> {}
    unsafe impl<T: PtrLike + Sync> Sync for TaggedPtr64<T> {}
}

#[cfg(false)]
pub use item_slot::*;
#[cfg(false)]
mod item_slot {
    use super::*;
    use crate::utils::components_as_tagged;

    use core::{
        cell::UnsafeCell,
        sync::atomic::{AtomicU64, Ordering},
    };

    const RESERVED_BIT: usize = 0b01;
    const FILLED_BIT: usize = 0b10;
    const STATE_MASK: usize = 0b11;

    // instead of a ptr we store a state bimask, which we can use to determine contested states
    pub(crate) struct OwnedSlot<T> {
        state: AtomicU64,
        data: UnsafeCell<Option<T>>,
    }

    impl<T> Slot for OwnedSlot<T> {
        type Item = T;
        const MAX_W: u64 = u16::MAX as u64 + 1;
        const EMPTY_PTR: *const Self::Item = null();

        fn new() -> Self {
            todo!()
        }

        fn components(&self) -> SlotComponents {
            todo!()
        }

        fn cmpxchg(
            &self,
            old_ptr: *const Self::Item,
            old_count: u64,
            mut item: Option<Self::Item>,
            new_count: u64,
        ) -> Result<Option<Self::Item>, Option<Self::Item>> {
            let new_state = 0;
            let old_state = components_as_tagged(old_count, old_ptr);
            let new_state = components_as_tagged(new_count, new_state as *const usize);
            let the_last_state = components_as_tagged(new_count, FILLED_BIT as *const usize);

            if let Err(_) = self.state.compare_exchange(
                old_state,
                new_state,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                return Err(item);
            }

            if item.is_some() {
                unsafe { &mut *self.data.get() }.replace(item.take().unwrap());
            } else {
                item = unsafe { &mut *self.data.get() }.take();
            }

            match self.state.compare_exchange(
                new_state,
                the_last_state,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => Ok(item),
                Err(_) => Err(item),
            }
        }

        fn is_empty(state: u64) -> bool {
            state as usize & STATE_MASK == 0
        }

        fn is_contested(state: u64) -> bool {
            state as usize & RESERVED_BIT == RESERVED_BIT
        }
    }

    unsafe impl<T: Send> Send for OwnedSlot<T> {}
    unsafe impl<T: Sync> Sync for OwnedSlot<T> {}
}

cfg_has_atomic_128! {
    pub use tagged_ptr_u128_portable::*;
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
                let new_ptr = item.map_or(null_mut(), |ptr| PtrLike::as_ptr(ptr));
                let old = components_as_u128(old_count, old_ptr);
                let new = components_as_u128(new_count, new_ptr);
                self.storage
                    .compare_exchange(old, new, Ordering::AcqRel, Ordering::Relaxed)
                    .map(|dword| {
                        PtrLike::from_raw(components_from_u128::<T::Item>(dword).1 as *mut T::Item)
                    })
                    .map_err(|_| PtrLike::from_raw(new_ptr))
            }

            fn is_empty(ptr: u64) -> bool {
                (ptr as *const T::Item).is_null()
            }

            fn is_contested(_: u64) -> bool {
                false
            }
        }


        impl<T: PtrLike> Drop for TaggedPtr128<T> {
            fn drop(&mut self) {
                let components = self.components();
                let _ptr = T::from_raw(components.state as *mut T::Item);
            }
        }

        unsafe impl<T: PtrLike + Send> Send for TaggedPtr128<T> {}
        unsafe impl<T: PtrLike + Sync> Sync for TaggedPtr128<T> {}
    }
}

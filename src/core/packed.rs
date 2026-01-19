use core::marker::PhantomData;
use core::{num::NonZeroU64, ptr::NonNull};

// TODO add safety comments in branch `allow_empty`

/// This trait is used to store the value in a `Slot`.
/// The slot may truncate the value to `MIN_BIT_WIDTH` bits.
/// Types implementing `AsPackedValue` may be stored in slots with `MAX_CARGO_BIT_WIDTH` >= `MIN_BIT_WIDTH`. This will be checked at compile time.
/// # SAFETY
/// TODO
pub unsafe trait AsPackedValue: Sized {
    /// The minimal bit width from which this type may be reconstructed.
    const MIN_BIT_WIDTH: usize;
    /// Truncates `Self` to the lower `MIN_BIT_WIDTH` bits.
    fn encode(zelf: Self) -> NonZeroTruncatedU64<Self>;

    /// Reconstructs `Self` from the lower `MIN_BIT_WIDTH` bits returned by `encode`.
    /// # SAFETY
    /// TODO
    unsafe fn decode(raw: NonZeroTruncatedU64<Self>) -> Self;
}

/// A NonZero U64, with the upper N bit set to 0.
#[repr(transparent)]
#[derive(Debug, PartialEq, Eq)]
pub struct NonZeroTruncatedU64<T> {
    v: NonZeroU64,
    _phantom: PhantomData<T>,
}

impl<T> Clone for NonZeroTruncatedU64<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for NonZeroTruncatedU64<T> {}

impl<T> NonZeroTruncatedU64<T> {
    #[allow(dead_code)]
    pub(crate) fn new_with_size<const BIT_WIDTH: usize>(mut value: u64) -> Option<Self> {
        if BIT_WIDTH < 64 {
            value = unpack!((value): BIT_WIDTH).1;
        }
        Some(Self {
            v: NonZeroU64::new(value)?,
            _phantom: PhantomData,
        })
    }

    /// Returns the raw u64 stored in this type
    pub fn read(&self) -> u64 {
        self.v.get()
    }

    #[allow(dead_code)]
    // Safety:
    // TODO
    pub(crate) unsafe fn new_unchecked(value: u64) -> Self {
        Self {
            // Safety:
            // TODO
            v: unsafe { NonZeroU64::new_unchecked(value) },
            _phantom: PhantomData,
        }
    }
}

impl<T: AsPackedValue> NonZeroTruncatedU64<T> {
    /// Contructs a new `NonZeroTruncatedU64` from an u64.
    /// This method will zero the upper 64 - `T::MIN_BIT_WIDTH` bits.
    /// Returns `None` if value was 0.
    pub fn new(mut value: u64) -> Option<Self> {
        // TODO make this a const created mask
        if T::MIN_BIT_WIDTH < 64 {
            value = unpack!((value): T::MIN_BIT_WIDTH).1;
        }
        Some(Self {
            v: NonZeroU64::new(value)?,
            _phantom: PhantomData,
        })
    }
}

// TODO make following non-panic, where possible

macro_rules! atomic_encode_primitive {
    ($type:ty) => {
        // Safety:
        // primitve numeric types with size <= WIDTH can be typecast safely
        unsafe impl $crate::core::AsPackedValue for $type {
            // storing one bit more allows also storing nul.
            // TODO remove this on `allow_empty`
            const MIN_BIT_WIDTH: usize = size_of::<$type>() * 8 + 1;

            fn encode(zelf: Self) -> $crate::core::NonZeroTruncatedU64<Self> {
                $crate::core::NonZeroTruncatedU64::new(zelf as u64 + 1).unwrap()
            }

            unsafe fn decode(raw: $crate::core::NonZeroTruncatedU64<Self>) -> Self {
                (raw.read() - 1) as Self
            }
        }
    };
}

atomic_encode_primitive!(u32);
atomic_encode_primitive!(u16);
atomic_encode_primitive!(u8);
atomic_encode_primitive!(i32);
atomic_encode_primitive!(i16);
atomic_encode_primitive!(i8);

// Safety:
// TODO
unsafe impl AsPackedValue for () {
    // do not truncate to 0
    const MIN_BIT_WIDTH: usize = 1;

    fn encode(_zelf: Self) -> NonZeroTruncatedU64<Self> {
        NonZeroTruncatedU64::new(1).unwrap()
    }

    unsafe fn decode(_raw: NonZeroTruncatedU64<Self>) -> Self {}
}

// TODO for targets with ptr width <=48 bits, we could also atomic_encode_primitive ptrs + usize

#[cfg(all(target_arch = "x86_64", target_pointer_width = "64"))]
mod x86_64 {
    use super::*;

    // Safety:
    // TODO
    unsafe impl<T> AsPackedValue for *const T
    where
        T: Sized,
    {
        const MIN_BIT_WIDTH: usize = 48;
        fn encode(zelf: Self) -> NonZeroTruncatedU64<Self> {
            NonZeroTruncatedU64::new(zelf as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64<Self>) -> Self {
            crate::utils::sign_extend(raw.read()) as *const T
        }
    }

    // Safety:
    // TODO
    unsafe impl<T> AsPackedValue for *mut T
    where
        T: Sized,
    {
        const MIN_BIT_WIDTH: usize = 48;
        fn encode(zelf: Self) -> NonZeroTruncatedU64<Self> {
            NonZeroTruncatedU64::new(zelf as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64<Self>) -> Self {
            crate::utils::sign_extend(raw.read()) as *mut T
        }
    }

    // Safety:
    // TODO
    unsafe impl<T> AsPackedValue for NonNull<T>
    where
        T: Sized,
    {
        const MIN_BIT_WIDTH: usize = 48;
        fn encode(zelf: Self) -> NonZeroTruncatedU64<Self> {
            NonZeroTruncatedU64::new(zelf.as_ptr() as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64<Self>) -> Self {
            Self::new(crate::utils::sign_extend(raw.read()) as *mut T)
                .expect("tried to recosntruct a NonNull from 0")
        }
    }

    // Safety:
    // TODO
    unsafe impl<T> AsPackedValue for &'static T {
        const MIN_BIT_WIDTH: usize = 48;
        fn encode(zelf: Self) -> NonZeroTruncatedU64<Self> {
            NonZeroTruncatedU64::new(zelf as *const T as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64<Self>) -> Self {
            // Safety:
            // TODO
            unsafe { &*(crate::utils::sign_extend(raw.read()) as *const T) }
        }
    }

    // Safety:
    // TODO
    unsafe impl<T> AsPackedValue for &'static mut T {
        const MIN_BIT_WIDTH: usize = 48;
        fn encode(zelf: Self) -> NonZeroTruncatedU64<Self> {
            NonZeroTruncatedU64::new(zelf as *mut T as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64<Self>) -> Self {
            // Safety:
            // TODO
            unsafe { &mut *(crate::utils::sign_extend(raw.read()) as *mut T) }
        }
    }

    #[cfg(any(feature = "alloc", test))]
    mod alloc_ {
        use super::*;

        use alloc::{boxed::Box, rc::Rc, sync::Arc};

        // Safety:
        // TODO
        unsafe impl<T> AsPackedValue for Arc<T> {
            const MIN_BIT_WIDTH: usize = 48;
            fn encode(zelf: Self) -> NonZeroTruncatedU64<Self> {
                NonZeroTruncatedU64::new(Arc::into_raw(zelf) as u64)
                    .expect("tried to store null ptr in queue. This is UB")
            }

            unsafe fn decode(raw: NonZeroTruncatedU64<Self>) -> Self {
                // Safety:
                // TODO
                unsafe { Arc::from_raw(crate::utils::sign_extend(raw.read()) as *mut T) }
            }
        }

        // Safety:
        // TODO
        unsafe impl<T> AsPackedValue for Box<T> {
            const MIN_BIT_WIDTH: usize = 48;
            fn encode(zelf: Self) -> NonZeroTruncatedU64<Self> {
                NonZeroTruncatedU64::new(Box::into_raw(zelf) as u64)
                    .expect("tried to store null ptr in queue. This is UB")
            }

            unsafe fn decode(raw: NonZeroTruncatedU64<Self>) -> Self {
                // Safety:
                // TODO
                unsafe { Box::from_raw(crate::utils::sign_extend(raw.read()) as *mut T) }
            }
        }

        // Safety:
        // TODO
        unsafe impl<T> AsPackedValue for Rc<T> {
            const MIN_BIT_WIDTH: usize = 48;
            fn encode(zelf: Self) -> NonZeroTruncatedU64<Self> {
                NonZeroTruncatedU64::new(Rc::into_raw(zelf) as u64)
                    .expect("tried to store null ptr in queue. This is UB")
            }

            unsafe fn decode(raw: NonZeroTruncatedU64<Self>) -> Self {
                // Safety:
                // TODO
                unsafe { Rc::from_raw(crate::utils::sign_extend(raw.read()) as *mut T) }
            }
        }
    }
}

#[cfg(all(not(target_arch = "x86_64"), target_pointer_width = "64"))]
mod x86_64 {
    use super::*;

    // Safety:
    // TODO
    unsafe impl<T> AsPackedValue for *const T
    where
        T: Sized,
    {
        const MIN_BIT_WIDTH: usize = 64;
        fn encode(zelf: Self) -> NonZeroTruncatedU64<Self> {
            NonZeroTruncatedU64::new(zelf as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64<Self>) -> Self {
            raw.read() as *const T
        }
    }

    // Safety:
    // TODO
    unsafe impl<T> AsPackedValue for *mut T
    where
        T: Sized,
    {
        const MIN_BIT_WIDTH: usize = 64;
        fn encode(zelf: Self) -> NonZeroTruncatedU64<Self> {
            NonZeroTruncatedU64::new(zelf as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64<Self>) -> Self {
            raw.read() as *mut T
        }
    }

    // Safety:
    // TODO
    unsafe impl<T> AsPackedValue for NonNull<T>
    where
        T: Sized,
    {
        const MIN_BIT_WIDTH: usize = 64;
        fn encode(zelf: Self) -> NonZeroTruncatedU64<Self> {
            NonZeroTruncatedU64::new(zelf.as_ptr() as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64<Self>) -> Self {
            Self::new(raw.read() as *mut T).expect("tried to recosntruct a NonNull from 0")
        }
    }

    // Safety:
    // TODO
    unsafe impl<T> AsPackedValue for &'static T {
        const MIN_BIT_WIDTH: usize = 64;
        fn encode(zelf: Self) -> NonZeroTruncatedU64<Self> {
            NonZeroTruncatedU64::new(zelf as *const T as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64<Self>) -> Self {
            // Safety:
            // TODO
            unsafe { &*(raw.read() as *const T) }
        }
    }

    // Safety:
    // TODO
    unsafe impl<T> AsPackedValue for &'static mut T {
        const MIN_BIT_WIDTH: usize = 64;
        fn encode(zelf: Self) -> NonZeroTruncatedU64<Self> {
            NonZeroTruncatedU64::new(zelf as *mut T as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64<Self>) -> Self {
            // Safety:
            // TODO
            unsafe { &mut *(raw.read() as *mut T) }
        }
    }

    #[cfg(any(feature = "alloc", test))]
    mod alloc_ {
        use super::*;

        use alloc::{boxed::Box, rc::Rc, sync::Arc};

        // Safety:
        // TODO
        unsafe impl<T> AsPackedValue for Arc<T> {
            const MIN_BIT_WIDTH: usize = 48;
            fn encode(zelf: Self) -> NonZeroTruncatedU64<Self> {
                NonZeroTruncatedU64::new(Arc::into_raw(zelf) as u64)
                    .expect("tried to store null ptr in queue. This is UB")
            }

            unsafe fn decode(raw: NonZeroTruncatedU64<Self>) -> Self {
                // Safety:
                // TODO
                unsafe { Arc::from_raw(raw.read() as *mut T) }
            }
        }

        // Safety:
        // TODO
        unsafe impl<T> AsPackedValue for Box<T> {
            const MIN_BIT_WIDTH: usize = 48;
            fn encode(zelf: Self) -> NonZeroTruncatedU64<Self> {
                NonZeroTruncatedU64::new(Box::into_raw(zelf) as u64)
                    .expect("tried to store null ptr in queue. This is UB")
            }

            unsafe fn decode(raw: NonZeroTruncatedU64<Self>) -> Self {
                // Safety:
                // TODO
                unsafe { Box::from_raw(raw.read() as *mut T) }
            }
        }

        // Safety:
        // TODO
        unsafe impl<T> AsPackedValue for Rc<T> {
            const MIN_BIT_WIDTH: usize = 48;
            fn encode(zelf: Self) -> NonZeroTruncatedU64<Self> {
                NonZeroTruncatedU64::new(Rc::into_raw(zelf) as u64)
                    .expect("tried to store null ptr in queue. This is UB")
            }

            unsafe fn decode(raw: NonZeroTruncatedU64<Self>) -> Self {
                // Safety:
                // TODO
                unsafe { Rc::from_raw(raw.read() as *mut T) }
            }
        }
    }
}

// assuming there are no pointer widths > 64 bits and all smaller widths are <= 32 bit.
// TODO: verify this
#[cfg(not(target_pointer_width = "64"))]
mod x86_64 {
    use super::*;

    // Safety:
    // TODO
    unsafe impl<T> AsPackedValue for *const T
    where
        T: Sized,
    {
        const MIN_BIT_WIDTH: usize = 32;
        fn encode(zelf: Self) -> NonZeroTruncatedU64<Self> {
            NonZeroTruncatedU64::new(zelf as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64<Self>) -> Self {
            raw.read() as *const T
        }
    }

    // Safety:
    // TODO
    unsafe impl<T> AsPackedValue for *mut T
    where
        T: Sized,
    {
        const MIN_BIT_WIDTH: usize = 32;
        fn encode(zelf: Self) -> NonZeroTruncatedU64<Self> {
            NonZeroTruncatedU64::new(zelf as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64<Self>) -> Self {
            raw.read() as *mut T
        }
    }

    // Safety:
    // TODO
    unsafe impl<T> AsPackedValue for NonNull<T>
    where
        T: Sized,
    {
        const MIN_BIT_WIDTH: usize = 32;
        fn encode(zelf: Self) -> NonZeroTruncatedU64<Self> {
            NonZeroTruncatedU64::new(zelf.as_ptr() as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64<Self>) -> Self {
            Self::new(raw.read() as *mut T).expect("tried to recosntruct a NonNull from 0")
        }
    }

    // Safety:
    // TODO
    unsafe impl<T> AsPackedValue for &'static T {
        const MIN_BIT_WIDTH: usize = 32;
        fn encode(zelf: Self) -> NonZeroTruncatedU64<Self> {
            NonZeroTruncatedU64::new(zelf as *const T as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64<Self>) -> Self {
            // Safety:
            // TODO
            unsafe { &*(raw.read() as *const T) }
        }
    }

    // Safety:
    // TODO
    unsafe impl<T> AsPackedValue for &'static mut T {
        const MIN_BIT_WIDTH: usize = 32;
        fn encode(zelf: Self) -> NonZeroTruncatedU64<Self> {
            NonZeroTruncatedU64::new(zelf as *mut T as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64<Self>) -> Self {
            // Safety:
            // TODO
            unsafe { &mut *(raw.read() as *mut T) }
        }
    }

    atomic_encode_primitive!(usize);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        static VAL: i32 = 0;
        let reference: &'static i32 = &VAL;

        let encoded = AsPackedValue::encode(reference);
        // Safety:
        // we just encoded it above
        let decoded: &'static i32 = unsafe { AsPackedValue::decode(encoded) };

        assert_eq!(reference as *const i32, decoded as *const i32,);

        let mut ptr = reference;
        for _ in 0..10000 {
            let enc = AsPackedValue::encode(ptr);
            // Safety:
            // we just encoded it above
            ptr = unsafe { AsPackedValue::decode(enc) };
        }
        assert_eq!(reference as *const i32, ptr as *const i32);
    }
}

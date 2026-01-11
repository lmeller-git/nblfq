use core::{num::NonZeroU64, ptr::NonNull};

/// # SAFETY
/// TODO
pub unsafe trait AsPackedValue {
    /// TODO
    const MIN_BIT_WIDTH: usize;
    /// TODO
    fn encode(zelf: Self) -> NonZeroTruncatedU64;

    /// # SAFETY
    /// TODO
    unsafe fn decode(raw: NonZeroTruncatedU64) -> Self;
}

/// TODO
#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct NonZeroTruncatedU64 {
    v: NonZeroU64,
}

impl NonZeroTruncatedU64 {
    /// TODO
    pub fn new<const BIT_WIDTH: usize>(mut value: u64) -> Option<Self> {
        if BIT_WIDTH < 64 {
            value = pack!((value): BIT_WIDTH);
        }
        Some(Self {
            v: NonZeroU64::new(value)?,
        })
    }

    /// TODO
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
        }
    }
}

// struct PackedComponents<const UPPER_BITS: usize, const LOWER_BITS: usize> {
//     upper: NonZeroTruncatedU64,
//     lower: NonZeroTruncatedU64<LOWER_BITS>,
// }

macro_rules! atomic_encode_primitive {
    ($type:ty) => {
        // Safety:
        // primitve numeric types with size <= WIDTH can be typecast safely
        unsafe impl $crate::core::AsPackedValue for $type {
            const MIN_BIT_WIDTH: usize = size_of::<$type>() * 8;

            fn encode(zelf: Self) -> $crate::core::NonZeroTruncatedU64 {
                $crate::core::NonZeroTruncatedU64::new::<{ Self::MIN_BIT_WIDTH }>(zelf as u64)
                    .expect("tried to store a zero value in queue. This is UB.")
            }

            unsafe fn decode(raw: $crate::core::NonZeroTruncatedU64) -> Self {
                raw.read() as Self
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
        fn encode(zelf: Self) -> NonZeroTruncatedU64 {
            NonZeroTruncatedU64::new::<48>(zelf as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64) -> Self {
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
        fn encode(zelf: Self) -> NonZeroTruncatedU64 {
            NonZeroTruncatedU64::new::<48>(zelf as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64) -> Self {
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
        fn encode(zelf: Self) -> NonZeroTruncatedU64 {
            NonZeroTruncatedU64::new::<48>(zelf.as_ptr() as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64) -> Self {
            Self::new(crate::utils::sign_extend(raw.read()) as *mut T)
                .expect("tried to recosntruct a NonNull from 0")
        }
    }

    // Safety:
    // TODO
    unsafe impl<T> AsPackedValue for &'static T {
        const MIN_BIT_WIDTH: usize = 48;
        fn encode(zelf: Self) -> NonZeroTruncatedU64 {
            NonZeroTruncatedU64::new::<48>(zelf as *const T as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64) -> Self {
            // Safety:
            // TODO
            unsafe { &*(crate::utils::sign_extend(raw.read()) as *const T) }
        }
    }

    // Safety:
    // TODO
    unsafe impl<T> AsPackedValue for &'static mut T {
        const MIN_BIT_WIDTH: usize = 48;
        fn encode(zelf: Self) -> NonZeroTruncatedU64 {
            NonZeroTruncatedU64::new::<48>(zelf as *mut T as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64) -> Self {
            // Safety:
            // TODO
            unsafe { &mut *(crate::utils::sign_extend(raw.read()) as *mut T) }
        }
    }

    #[cfg(feature = "alloc")]
    mod alloc_ {
        use super::*;

        use alloc::{boxed::Box, rc::Rc, sync::Arc};

        // Safety:
        // TODO
        unsafe impl<T> AsPackedValue for Arc<T> {
            const MIN_BIT_WIDTH: usize = 48;
            fn encode(zelf: Self) -> NonZeroTruncatedU64 {
                NonZeroTruncatedU64::new::<48>(Arc::into_raw(zelf) as u64)
                    .expect("tried to store null ptr in queue. This is UB")
            }

            unsafe fn decode(raw: NonZeroTruncatedU64) -> Self {
                // Safety:
                // TODO
                unsafe { Arc::from_raw(crate::utils::sign_extend(raw.read()) as *mut T) }
            }
        }

        // Safety:
        // TODO
        unsafe impl<T> AsPackedValue for Box<T> {
            const MIN_BIT_WIDTH: usize = 48;
            fn encode(zelf: Self) -> NonZeroTruncatedU64 {
                NonZeroTruncatedU64::new::<48>(Box::into_raw(zelf) as u64)
                    .expect("tried to store null ptr in queue. This is UB")
            }

            unsafe fn decode(raw: NonZeroTruncatedU64) -> Self {
                // Safety:
                // TODO
                unsafe { Box::from_raw(crate::utils::sign_extend(raw.read()) as *mut T) }
            }
        }

        // Safety:
        // TODO
        unsafe impl<T> AsPackedValue for Rc<T> {
            const MIN_BIT_WIDTH: usize = 48;
            fn encode(zelf: Self) -> NonZeroTruncatedU64 {
                NonZeroTruncatedU64::new::<48>(Rc::into_raw(zelf) as u64)
                    .expect("tried to store null ptr in queue. This is UB")
            }

            unsafe fn decode(raw: NonZeroTruncatedU64) -> Self {
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
        fn encode(zelf: Self) -> NonZeroTruncatedU64 {
            NonZeroTruncatedU64::new::<64>(zelf as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64) -> Self {
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
        fn encode(zelf: Self) -> NonZeroTruncatedU64 {
            NonZeroTruncatedU64::new::<64>(zelf as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64) -> Self {
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
        fn encode(zelf: Self) -> NonZeroTruncatedU64 {
            NonZeroTruncatedU64::new::<64>(zelf.as_ptr() as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64) -> Self {
            Self::new(raw.read() as *mut T).expect("tried to recosntruct a NonNull from 0")
        }
    }

    // Safety:
    // TODO
    unsafe impl<T> AsPackedValue for &'static T {
        const MIN_BIT_WIDTH: usize = 64;
        fn encode(zelf: Self) -> NonZeroTruncatedU64 {
            NonZeroTruncatedU64::new::<64>(zelf as *const T as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64) -> Self {
            // Safety:
            // TODO
            unsafe { &*(raw.read() as *const T) }
        }
    }

    // Safety:
    // TODO
    unsafe impl<T> AsPackedValue for &'static mut T {
        const MIN_BIT_WIDTH: usize = 64;
        fn encode(zelf: Self) -> NonZeroTruncatedU64 {
            NonZeroTruncatedU64::new::<64>(zelf as *mut T as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64) -> Self {
            // Safety:
            // TODO
            unsafe { &mut *(raw.read() as *mut T) }
        }
    }

    #[cfg(feature = "alloc")]
    mod alloc_ {
        use super::*;

        use alloc::{boxed::Box, rc::Rc, sync::Arc};

        // Safety:
        // TODO
        unsafe impl<T> AsPackedValue for Arc<T> {
            const MIN_BIT_WIDTH: usize = 48;
            fn encode(zelf: Self) -> NonZeroTruncatedU64 {
                NonZeroTruncatedU64::new::<48>(Arc::into_raw(zelf) as u64)
                    .expect("tried to store null ptr in queue. This is UB")
            }

            unsafe fn decode(raw: NonZeroTruncatedU64) -> Self {
                // Safety:
                // TODO
                unsafe { Arc::from_raw(raw.read() as *mut T) }
            }
        }

        // Safety:
        // TODO
        unsafe impl<T> AsPackedValue for Box<T> {
            const MIN_BIT_WIDTH: usize = 48;
            fn encode(zelf: Self) -> NonZeroTruncatedU64 {
                NonZeroTruncatedU64::new::<48>(Box::into_raw(zelf) as u64)
                    .expect("tried to store null ptr in queue. This is UB")
            }

            unsafe fn decode(raw: NonZeroTruncatedU64) -> Self {
                // Safety:
                // TODO
                unsafe { Box::from_raw(raw.read() as *mut T) }
            }
        }

        // Safety:
        // TODO
        unsafe impl<T> AsPackedValue for Rc<T> {
            const MIN_BIT_WIDTH: usize = 48;
            fn encode(zelf: Self) -> NonZeroTruncatedU64 {
                NonZeroTruncatedU64::new::<48>(Rc::into_raw(zelf) as u64)
                    .expect("tried to store null ptr in queue. This is UB")
            }

            unsafe fn decode(raw: NonZeroTruncatedU64) -> Self {
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
        fn encode(zelf: Self) -> NonZeroTruncatedU64 {
            NonZeroTruncatedU64::new::<32>(zelf as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64) -> Self {
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
        fn encode(zelf: Self) -> NonZeroTruncatedU64 {
            NonZeroTruncatedU64::new::<32>(zelf as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64) -> Self {
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
        fn encode(zelf: Self) -> NonZeroTruncatedU64 {
            NonZeroTruncatedU64::new::<32>(zelf.as_ptr() as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64) -> Self {
            Self::new(raw.read() as *mut T).expect("tried to recosntruct a NonNull from 0")
        }
    }

    // Safety:
    // TODO
    unsafe impl<T> AsPackedValue for &'static T {
        const MIN_BIT_WIDTH: usize = 32;
        fn encode(zelf: Self) -> NonZeroTruncatedU64 {
            NonZeroTruncatedU64::new::<32>(zelf as *const T as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64) -> Self {
            // Safety:
            // TODO
            unsafe { &*(raw.read() as *const T) }
        }
    }

    // Safety:
    // TODO
    unsafe impl<T> AsPackedValue for &'static mut T {
        const MIN_BIT_WIDTH: usize = 32;
        fn encode(zelf: Self) -> NonZeroTruncatedU64 {
            NonZeroTruncatedU64::new::<32>(zelf as *mut T as u64)
                .expect("tried to store null ptr in queue. This is UB")
        }

        unsafe fn decode(raw: NonZeroTruncatedU64) -> Self {
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

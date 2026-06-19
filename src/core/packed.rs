use core::{
    marker::PhantomData,
    num::{NonZeroI8, NonZeroI16, NonZeroI32, NonZeroU8, NonZeroU16, NonZeroU32},
    ptr::NonNull,
};

// TODO add safety comments in branch `allow_empty`

/// This trait is used to store the value in a `Slot`.
/// The slot may truncate the value to `MIN_BIT_WIDTH` bits.
/// Types implementing `AsPackedValue` may be stored in slots with `MAX_CARGO_BIT_WIDTH` >= `MIN_BIT_WIDTH`. This will be checked at compile time.
/// `MIN_BIT_WIDTH` cannot be larger than 64
/// # SAFETY
/// - both `decode` and `encode` must be atomic and non-blocking
/// - `decode` must only be called on a value returned by `encode`
/// - the encoded value must be reconstructable fully from the lower `MIN_BIT_WIDTH` bits
pub unsafe trait AsPackedValue: Sized {
    /// The minimal bit width from which this type may be reconstructed.
    const MIN_BIT_WIDTH: usize;
    /// Truncates `Self` to the lower `MIN_BIT_WIDTH` bits.
    /// The caller is responsible for reconstructing this value usign `decode`
    fn encode(zelf: Self) -> TruncatedU64<Self>;

    /// Reconstructs `Self` from the lower `MIN_BIT_WIDTH` bits returned by `encode`.
    /// # SAFETY
    /// The caller must ensure that the passed value is a valid value returned by `encode`
    unsafe fn decode(raw: TruncatedU64<Self>) -> Self;

    /// Validates wether self is actually safe to pack into Self::MIN_BIT_WIDTH bits at runtime.
    fn is_rt_safe() -> bool {
        true
    }
}

/// An U64, with the upper N bits set to 0.
#[repr(transparent)]
#[derive(Debug, PartialEq, Eq)]
pub struct TruncatedU64<T> {
    v: u64,
    _phantom: PhantomData<T>,
}

impl<T> Clone for TruncatedU64<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for TruncatedU64<T> {}

impl<T> TruncatedU64<T> {
    /// Returns the raw u64 stored in this type
    pub fn read(&self) -> u64 {
        self.v
    }
}

impl<T: AsPackedValue> TruncatedU64<T> {
    /// Contructs a new `TruncatedU64` from an u64.
    /// This method will zero the upper 64 - `T::MIN_BIT_WIDTH` bits.
    pub fn new(mut value: u64) -> Self {
        // TODO make this a const created mask
        if T::MIN_BIT_WIDTH < 64 {
            value = unpack!((value): T::MIN_BIT_WIDTH).1;
        }
        Self {
            v: value,
            _phantom: PhantomData,
        }
    }
}

macro_rules! atomic_encode_primitive {
    ($type:ty) => {
        // Safety:
        // primitve numeric types with size <= WIDTH can be typecast safely
        unsafe impl $crate::core::AsPackedValue for $type {
            const MIN_BIT_WIDTH: usize = size_of::<$type>() * 8;

            fn encode(zelf: Self) -> $crate::core::TruncatedU64<Self> {
                $crate::core::TruncatedU64::new(zelf as u64)
            }

            unsafe fn decode(raw: $crate::core::TruncatedU64<Self>) -> Self {
                (raw.read()) as Self
            }
        }
    };
}

macro_rules! atomic_encode_non_zero_primitive {
    ($type:ty, $raw:ty) => {
        // Safety:
        // primitve numeric types with size <= WIDTH can be typecast safely
        unsafe impl $crate::core::AsPackedValue for $type {
            const MIN_BIT_WIDTH: usize = size_of::<$type>() * 8;

            fn encode(zelf: Self) -> $crate::core::TruncatedU64<Self> {
                $crate::core::TruncatedU64::new(zelf.get() as u64)
            }

            unsafe fn decode(raw: $crate::core::TruncatedU64<Self>) -> Self {
                Self::new(raw.read() as $raw)
                    .expect("trying to construct a NonZero from a zero value")
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

atomic_encode_non_zero_primitive!(NonZeroU32, u32);
atomic_encode_non_zero_primitive!(NonZeroU16, u16);
atomic_encode_non_zero_primitive!(NonZeroU8, u8);
atomic_encode_non_zero_primitive!(NonZeroI32, i32);
atomic_encode_non_zero_primitive!(NonZeroI16, i16);
atomic_encode_non_zero_primitive!(NonZeroI8, i8);

// Safety:
// () has no size and data
unsafe impl AsPackedValue for () {
    const MIN_BIT_WIDTH: usize = 0;

    fn encode(_zelf: Self) -> TruncatedU64<Self> {
        TruncatedU64::new(0)
    }

    // Safety:
    // nothing to do
    unsafe fn decode(_raw: TruncatedU64<Self>) -> Self {}
}

// TODO for targets with ptr width <=48 bits, we could also atomic_encode_primitive ptrs + usize

/// Some x86_64 and aarch64 based hardware has support for level 5 pagetables / use more than 48 bits for pointers. These implementations are not safe on hardware using more that 48 bits for pointers.
/// This invariant will be checked at runtime.
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
mod bit64 {
    use super::*;

    fn assert_ptr_safety() {
        let dummy = 42;
        let raw = &dummy as *const i32;
        let addr = raw as u64;
        let top_16 = addr >> 48;
        let bit_47 = (addr >> 47) & 1;

        assert!(
            (bit_47 == 0 && top_16 == 0) || (bit_47 == 1 && top_16 == 0xFFFF),
            "Pointer {:p} exceeds 48-bit address space! AsPackedValue is unsafe here. Consider using a PooledQueue, a Tagged128 Slot or a custom ptrlike value encodeable in <= 48bits.",
            raw
        );
    }

    // Safety:
    // This implementation assumes that pointers may be reconstructed from 48-bits using sign extenion.
    //
    // WARNING: This implementation is unsound on systems using more than 48 bits
    unsafe impl<T> AsPackedValue for *const T
    where
        T: Sized,
    {
        const MIN_BIT_WIDTH: usize = 48;

        fn encode(zelf: Self) -> TruncatedU64<Self> {
            TruncatedU64::new(zelf as u64)
        }

        unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
            crate::utils::sign_extend(raw.read()) as *const T
        }

        fn is_rt_safe() -> bool {
            assert_ptr_safety();
            true
        }
    }

    // Safety:
    // This implementation assumes that pointers may be reconstructed from 48-bits using sign extenion.
    //
    // WARNING: This implementation is unsound on systems using more than 48 bits
    unsafe impl<T> AsPackedValue for *mut T
    where
        T: Sized,
    {
        const MIN_BIT_WIDTH: usize = 48;

        fn encode(zelf: Self) -> TruncatedU64<Self> {
            TruncatedU64::new(zelf as u64)
        }

        unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
            crate::utils::sign_extend(raw.read()) as *mut T
        }

        fn is_rt_safe() -> bool {
            assert_ptr_safety();
            true
        }
    }

    // Safety:
    // This implementation assumes that pointers may be reconstructed from 48-bits using sign extenion.
    //
    // WARNING: This implementation is unsound on systems using more than 48 bits
    unsafe impl<T> AsPackedValue for NonNull<T>
    where
        T: Sized,
    {
        const MIN_BIT_WIDTH: usize = 48;

        fn encode(zelf: Self) -> TruncatedU64<Self> {
            TruncatedU64::new(zelf.as_ptr() as u64)
        }

        unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
            Self::new(crate::utils::sign_extend(raw.read()) as *mut T)
                .expect("tried to recosntruct a NonNull from 0")
        }

        fn is_rt_safe() -> bool {
            assert_ptr_safety();
            true
        }
    }

    // Safety:
    // This implementation assumes that pointers may be reconstructed from 48-bits using sign extenion.
    //
    // WARNING: This implementation is unsound on systems using more than 48 bits
    unsafe impl<T> AsPackedValue for &'static T {
        const MIN_BIT_WIDTH: usize = 48;

        fn encode(zelf: Self) -> TruncatedU64<Self> {
            TruncatedU64::new(zelf as *const T as u64)
        }

        unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
            // Safety:
            // The caller must ensure that this is called on a value created by `encode` and the reference is still valid
            unsafe { &*(crate::utils::sign_extend(raw.read()) as *const T) }
        }

        fn is_rt_safe() -> bool {
            assert_ptr_safety();
            true
        }
    }

    // Safety:
    // This implementation assumes that pointers may be reconstructed from 48-bits using sign extenion.
    //
    // WARNING: This implementation is unsound on systems using more than 48 bits
    unsafe impl<T> AsPackedValue for &'static mut T {
        const MIN_BIT_WIDTH: usize = 48;

        fn encode(zelf: Self) -> TruncatedU64<Self> {
            TruncatedU64::new(zelf as *mut T as u64)
        }

        unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
            // Safety:
            // The caller must ensure that this is called only once on a value created by `encode` and the reference is still valid
            unsafe { &mut *(crate::utils::sign_extend(raw.read()) as *mut T) }
        }

        fn is_rt_safe() -> bool {
            assert_ptr_safety();
            true
        }
    }

    #[cfg(any(feature = "alloc", test))]
    mod alloc_ {
        use alloc::{
            boxed::Box,
            rc::{self, Rc},
            sync::{self, Arc},
        };

        use super::*;

        // Safety:
        // This implementation assumes that pointers may be reconstructed from 48-bits using sign extenion.
        //
        // WARNING: This implementation is unsound on systems using more than 48 bits
        unsafe impl<T> AsPackedValue for Box<T> {
            const MIN_BIT_WIDTH: usize = 48;

            fn encode(zelf: Self) -> TruncatedU64<Self> {
                let raw = Box::into_raw(zelf);
                TruncatedU64::new(raw as u64)
            }

            unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
                // Safety:
                // The caller must ensure that this is called only once on a value created by `encode` and the underlying allocation is still valid
                unsafe { Box::from_raw(crate::utils::sign_extend(raw.read()) as *mut T) }
            }

            fn is_rt_safe() -> bool {
                assert_ptr_safety();
                true
            }
        }

        // Safety:
        // This implementation assumes that pointers may be reconstructed from 48-bits using sign extenion.
        //
        // WARNING: This implementation is unsound on systems using more than 48 bits
        unsafe impl<T> AsPackedValue for Rc<T> {
            const MIN_BIT_WIDTH: usize = 48;

            fn encode(zelf: Self) -> TruncatedU64<Self> {
                let raw = Rc::into_raw(zelf);
                TruncatedU64::new(raw as u64)
            }

            unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
                // Safety:
                // The caller must ensure that this is called only once on a value created by `encode` and the underlying allocation is still valid
                unsafe { Rc::from_raw(crate::utils::sign_extend(raw.read()) as *mut T) }
            }

            fn is_rt_safe() -> bool {
                assert_ptr_safety();
                true
            }
        }

        // Safety:
        // This implementation assumes that pointers may be reconstructed from 48-bits using sign extenion.
        //
        // WARNING: This implementation is unsound on systems using more than 48 bits
        unsafe impl<T> AsPackedValue for Arc<T> {
            const MIN_BIT_WIDTH: usize = 48;

            fn encode(zelf: Self) -> TruncatedU64<Self> {
                let raw = Arc::into_raw(zelf);
                TruncatedU64::new(raw as u64)
            }

            unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
                // Safety:
                // The caller must ensure that this is called only once on a value created by `encode` and the underlying allocation is still valid
                unsafe { Arc::from_raw(crate::utils::sign_extend(raw.read()) as *mut T) }
            }

            fn is_rt_safe() -> bool {
                assert_ptr_safety();
                true
            }
        }

        // Safety:
        // This implementation assumes that pointers may be reconstructed from 48-bits using sign extenion.
        //
        // WARNING: This implementation is unsound on systems using more than 48 bits
        unsafe impl<T> AsPackedValue for rc::Weak<T> {
            const MIN_BIT_WIDTH: usize = 48;

            fn encode(zelf: Self) -> TruncatedU64<Self> {
                let raw = rc::Weak::into_raw(zelf);
                TruncatedU64::new(raw as u64)
            }

            unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
                // Safety:
                // The caller must ensure that this is called only once on a value created by `encode` and the underlying allocation is still valid
                unsafe { rc::Weak::from_raw(crate::utils::sign_extend(raw.read()) as *mut T) }
            }

            fn is_rt_safe() -> bool {
                assert_ptr_safety();
                true
            }
        }

        // Safety:
        // This implementation assumes that pointers may be reconstructed from 48-bits using sign extenion.
        //
        // WARNING: This implementation is unsound on systems using more than 48 bits
        unsafe impl<T> AsPackedValue for sync::Weak<T> {
            const MIN_BIT_WIDTH: usize = 48;

            fn encode(zelf: Self) -> TruncatedU64<Self> {
                let raw = sync::Weak::into_raw(zelf);
                TruncatedU64::new(raw as u64)
            }

            unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
                // Safety:
                // The caller must ensure that this is called only once on a value created by `encode` and the underlying allocation is still valid
                unsafe { sync::Weak::from_raw(crate::utils::sign_extend(raw.read()) as *mut T) }
            }

            fn is_rt_safe() -> bool {
                assert_ptr_safety();
                true
            }
        }
    }
}

/// Cannot guarantee that all 64 bit architectures use 48bit pointers.
#[cfg(all(
    not(target_arch = "x86_64"),
    not(target_arch = "aarch64"),
    target_pointer_width = "64"
))]
mod full_bit64 {
    use super::*;

    // Safety:
    // casting *const T from and to u64 is safe
    unsafe impl<T> AsPackedValue for *const T
    where
        T: Sized,
    {
        const MIN_BIT_WIDTH: usize = 64;

        fn encode(zelf: Self) -> TruncatedU64<Self> {
            TruncatedU64::new(zelf as u64)
        }

        unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
            raw.read() as *const T
        }
    }

    // Safety:
    // casting *const T from and to u64 is safe
    unsafe impl<T> AsPackedValue for *mut T
    where
        T: Sized,
    {
        const MIN_BIT_WIDTH: usize = 64;

        fn encode(zelf: Self) -> TruncatedU64<Self> {
            TruncatedU64::new(zelf as u64)
        }

        unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
            raw.read() as *mut T
        }
    }

    // Safety:
    // casting *const T from and to u64 is safe
    // casting from u64 to NonNull<T> is safe, if that u64 was obtained from a NonNull<T>
    unsafe impl<T> AsPackedValue for NonNull<T>
    where
        T: Sized,
    {
        const MIN_BIT_WIDTH: usize = 64;

        fn encode(zelf: Self) -> TruncatedU64<Self> {
            TruncatedU64::new(zelf.as_ptr() as u64)
        }

        unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
            Self::new(raw.read() as *mut T).expect("tried to recosntruct a NonNull from 0")
        }
    }

    // Safety:
    // casting *const T from and to u64 is safe
    unsafe impl<T> AsPackedValue for &'static T {
        const MIN_BIT_WIDTH: usize = 64;

        fn encode(zelf: Self) -> TruncatedU64<Self> {
            TruncatedU64::new(zelf as *const T as u64)
        }

        unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
            // Safety:
            // The caller must ensure that the value was returned by `encode` and the reference is still valid
            unsafe { &*(raw.read() as *const T) }
        }
    }

    // Safety:
    // casting *const T from and to u64 is safe
    unsafe impl<T> AsPackedValue for &'static mut T {
        const MIN_BIT_WIDTH: usize = 64;

        fn encode(zelf: Self) -> TruncatedU64<Self> {
            TruncatedU64::new(zelf as *mut T as u64)
        }

        unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
            // Safety:
            // The caller must ensure that this is called only once on a value returned by `encode` and the reference is still valid
            unsafe { &mut *(raw.read() as *mut T) }
        }
    }

    #[cfg(any(feature = "alloc", test))]
    mod alloc_ {
        use alloc::{
            boxed::Box,
            rc::{self, Rc},
            sync::{self, Arc},
        };

        use super::*;

        // Safety:
        // casting *const T from and to u64 is safe
        unsafe impl<T> AsPackedValue for Box<T> {
            const MIN_BIT_WIDTH: usize = 64;

            fn encode(zelf: Self) -> TruncatedU64<Self> {
                TruncatedU64::new(Box::into_raw(zelf) as u64)
            }

            unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
                // Safety:
                // The caller must ensure that this is called only once on a value returned by `encode` and the allocation is still valid
                unsafe { Box::from_raw(raw.read() as *mut T) }
            }
        }

        // Safety:
        // casting *const T from and to u64 is safe
        unsafe impl<T> AsPackedValue for Rc<T> {
            const MIN_BIT_WIDTH: usize = 64;

            fn encode(zelf: Self) -> TruncatedU64<Self> {
                let raw = Rc::into_raw(zelf);
                TruncatedU64::new(raw as u64)
            }

            unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
                // Safety:
                // The caller must ensure that this is called only once on a value created by `encode` and the underlying allocation is still valid
                unsafe { Rc::from_raw(raw.read() as *mut T) }
            }
        }

        // Safety:
        // casting *const T from and to u64 is safe
        unsafe impl<T> AsPackedValue for Arc<T> {
            const MIN_BIT_WIDTH: usize = 64;

            fn encode(zelf: Self) -> TruncatedU64<Self> {
                let raw = Arc::into_raw(zelf);
                TruncatedU64::new(raw as u64)
            }

            unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
                // Safety:
                // The caller must ensure that this is called only once on a value created by `encode` and the underlying allocation is still valid
                unsafe { Arc::from_raw(raw.read() as *mut T) }
            }
        }

        // Safety:
        // casting *const T from and to u64 is safe
        unsafe impl<T> AsPackedValue for rc::Weak<T> {
            const MIN_BIT_WIDTH: usize = 64;

            fn encode(zelf: Self) -> TruncatedU64<Self> {
                let raw = rc::Weak::into_raw(zelf);
                TruncatedU64::new(raw as u64)
            }

            unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
                // Safety:
                // The caller must ensure that this is called only once on a value created by `encode` and the underlying allocation is still valid
                unsafe { rc::Weak::from_raw(raw.read() as *mut T) }
            }
        }

        // Safety:
        // casting *const T from and to u64 is safe
        unsafe impl<T> AsPackedValue for sync::Weak<T> {
            const MIN_BIT_WIDTH: usize = 64;

            fn encode(zelf: Self) -> TruncatedU64<Self> {
                let raw = sync::Weak::into_raw(zelf);
                TruncatedU64::new(raw as u64)
            }

            unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
                // Safety:
                // The caller must ensure that this is called only once on a value created by `encode` and the underlying allocation is still valid
                unsafe { sync::Weak::from_raw(raw.read() as *mut T) }
            }
        }
    }
}

/// if the native ptr-size is <= 48 bits, it is always safe to pack a ptr into 48 or more bits
#[cfg(not(target_pointer_width = "64"))]
mod bit32 {
    use core::num::NonZeroUsize;

    use super::*;

    // Safety:
    // Casting from a ptr to a usize is safe
    unsafe impl<T> AsPackedValue for *const T
    where
        T: Sized,
    {
        const MIN_BIT_WIDTH: usize = size_of::<usize>() * 8;

        fn encode(zelf: Self) -> TruncatedU64<Self> {
            TruncatedU64::new(zelf as u64)
        }

        unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
            raw.read() as *const T
        }
    }

    // Safety:
    // Casting from a ptr to a usize is safe
    unsafe impl<T> AsPackedValue for *mut T
    where
        T: Sized,
    {
        const MIN_BIT_WIDTH: usize = size_of::<usize>() * 8;

        fn encode(zelf: Self) -> TruncatedU64<Self> {
            TruncatedU64::new(zelf as u64)
        }

        unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
            raw.read() as *mut T
        }
    }

    // Safety:
    // Casting from a ptr to a usize is safe
    unsafe impl<T> AsPackedValue for NonNull<T>
    where
        T: Sized,
    {
        const MIN_BIT_WIDTH: usize = size_of::<usize>() * 8;

        fn encode(zelf: Self) -> TruncatedU64<Self> {
            TruncatedU64::new(zelf.as_ptr() as u64)
        }

        unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
            Self::new(raw.read() as *mut T)
                .expect("Constructing a NonNull form a null ptr wich was not obtained from encode")
        }
    }

    // Safety:
    // Casting from a ptr to a usize is safe
    unsafe impl<T> AsPackedValue for &'static T {
        const MIN_BIT_WIDTH: usize = size_of::<usize>() * 8;

        fn encode(zelf: Self) -> TruncatedU64<Self> {
            TruncatedU64::new(zelf as *const T as u64)
        }

        unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
            // Safety:
            // The caller must ensure that this is called only on a value returned by `encode` and the reference is still valid
            unsafe { &*(raw.read() as *const T) }
        }
    }

    // Safety:
    // Casting from a ptr to a usize is safe
    unsafe impl<T> AsPackedValue for &'static mut T {
        const MIN_BIT_WIDTH: usize = size_of::<usize>() * 8;

        fn encode(zelf: Self) -> TruncatedU64<Self> {
            TruncatedU64::new(zelf as *mut T as u64)
        }

        unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
            // Safety:
            // The caller must ensure that this is called only once on a value returned by `encode` and the reference is still valid
            unsafe { &mut *(raw.read() as *mut T) }
        }
    }

    #[cfg(feature = "alloc")]
    mod alloc_ {
        use alloc::{
            boxed::Box,
            rc::{self, Rc},
            sync::{self, Arc},
        };

        use super::*;

        // Safety:
        // Casting from a ptr to a usize is safe
        unsafe impl<T> AsPackedValue for Box<T> {
            const MIN_BIT_WIDTH: usize = size_of::<usize>() * 8;

            fn encode(zelf: Self) -> TruncatedU64<Self> {
                TruncatedU64::new(Box::into_raw(zelf) as u64)
            }

            unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
                // Safety:
                // The caller must ensure that this is called only once on a value returned by `encode` and the allocation is still valid
                unsafe { Box::from_raw(raw.read() as *mut T) }
            }
        }

        // Safety:
        // Casting from a ptr to a usize is safe
        unsafe impl<T> AsPackedValue for Rc<T> {
            const MIN_BIT_WIDTH: usize = size_of::<usize>() * 8;

            fn encode(zelf: Self) -> TruncatedU64<Self> {
                let raw = Rc::into_raw(zelf);
                TruncatedU64::new(raw as u64)
            }

            unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
                // Safety:
                // The caller must ensure that this is called only once on a value created by `encode` and the underlying allocation is still valid
                unsafe { Rc::from_raw(raw.read() as *mut T) }
            }
        }

        // Safety:
        // Casting from a ptr to a usize is safe
        unsafe impl<T> AsPackedValue for Arc<T> {
            const MIN_BIT_WIDTH: usize = size_of::<usize>() * 8;

            fn encode(zelf: Self) -> TruncatedU64<Self> {
                let raw = Arc::into_raw(zelf);
                TruncatedU64::new(raw as u64)
            }

            unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
                // Safety:
                // The caller must ensure that this is called only once on a value created by `encode` and the underlying allocation is still valid
                unsafe { Arc::from_raw(raw.read() as *mut T) }
            }
        }

        // Safety:
        // Casting from a ptr to a usize is safe
        unsafe impl<T> AsPackedValue for rc::Weak<T> {
            const MIN_BIT_WIDTH: usize = size_of::<usize>() * 8;

            fn encode(zelf: Self) -> TruncatedU64<Self> {
                let raw = rc::Weak::into_raw(zelf);
                TruncatedU64::new(raw as u64)
            }

            unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
                // Safety:
                // The caller must ensure that this is called only once on a value created by `encode` and the underlying allocation is still valid
                unsafe { rc::Weak::from_raw(raw.read() as *mut T) }
            }
        }

        // Safety:
        // Casting from a ptr to a usize is safe
        unsafe impl<T> AsPackedValue for sync::Weak<T> {
            const MIN_BIT_WIDTH: usize = size_of::<usize>() * 8;

            fn encode(zelf: Self) -> TruncatedU64<Self> {
                let raw = sync::Weak::into_raw(zelf);
                TruncatedU64::new(raw as u64)
            }

            unsafe fn decode(raw: TruncatedU64<Self>) -> Self {
                // Safety:
                // The caller must ensure that this is called only once on a value created by `encode` and the underlying allocation is still valid
                unsafe { sync::Weak::from_raw(raw.read() as *mut T) }
            }
        }
    }

    atomic_encode_primitive!(usize);
    atomic_encode_non_zero_primitive!(NonZeroUsize, usize);
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! generate_test {
        ($name:ident: $constructor:expr, $type:ty, $deref:expr, $drop:expr) => {
            #[test]
            fn $name() {
                #[allow(dead_code)]
                static VALUE: i32 = 42;
                const WIDTH: usize = <$type as AsPackedValue>::MIN_BIT_WIDTH;

                assert!(<$type>::is_rt_safe());

                let ptr1: $type = $constructor;
                let expected = $deref(&ptr1);

                let mut encoded = AsPackedValue::encode(ptr1);

                if WIDTH < 64 {
                    let packed_encoded = pack!((!0, encoded.read()): WIDTH);
                    encoded = TruncatedU64::new(packed_encoded);
                }

                // Safety:
                // we just encoded this value
                let decoded: $type = unsafe { AsPackedValue::decode(encoded) };

                assert_eq!($deref(&decoded), expected);

                #[allow(dropping_copy_types, dropping_references)]
                $drop(decoded);
            }
        };
        ($name:ident: $constructor:expr, $type:ty) => {
            generate_test!($name: $constructor, $type, |x: &$type| x.clone(), drop);
        };
    }

    generate_test!(raw: &VALUE as *const i32, *const i32);
    generate_test!(raw_mut: &VALUE as *const i32 as *mut i32, *mut i32);
    generate_test!(r#ref: &VALUE, &'static i32);
    // we cannot compare two mutable borrows to the same data. Since the addrs is the same across *const T and &mut T, we simply deref to *const T.
    generate_test!(ref_mut: Box::leak(Box::new(VALUE)), & 'static mut i32, |x: &&'static mut i32| *x as *const i32,
        |x: &'static mut i32|
            // Safety:
            // drop is only called once on the decoded value
            unsafe {Box::from_raw(x as *mut i32)}
    );
    generate_test!(nonnull: NonNull::new(&VALUE as *const i32 as *mut i32).unwrap(), NonNull<i32>);
    generate_test!(primitive_u32: 42, u32);
    generate_test!(primitive_nonzero_u32: NonZeroU32::new(42).unwrap(), NonZeroU32);
    generate_test!(unit: (), ());

    #[cfg(feature = "alloc")]
    mod alloc_ {
        use alloc::{
            boxed::Box,
            rc::{self, Rc},
            sync::{self, Arc},
        };

        use super::*;

        generate_test!(r#box: Box::new(VALUE), Box<i32>);
        generate_test!(r#arc: Arc::new(VALUE), Arc<i32>);
        generate_test!(r#rc: Rc::new(VALUE), Rc<i32>);
        generate_test!(weak_rc: Rc::downgrade(&Rc::new(VALUE)), rc::Weak<i32>, |x: &rc::Weak<i32>| x.as_ptr(), drop);
        generate_test!(weak_arc: Arc::downgrade(&Arc::new(VALUE)), sync::Weak<i32>, |x: &sync::Weak<i32>| x.as_ptr(), drop);
    }
}

/// cfg that disables Tagged64 slot based on architecture and feature flags.
///
/// Usage:
/// ```rust
/// use nblf_queue::cfg_atomic_tagged64;
///
/// cfg_atomic_tagged64! {
///     use nblf_queue::core::slots::Tagged64;
/// }
/// ```
#[macro_export]
macro_rules! cfg_atomic_tagged64 {
    ($($item:item)*) => {
       $(
           #[cfg(any(target_has_atomic = "64", feature = "atomic-fallback"))]
            $item
        )*
    };
}

/// cfg that disables Tagged128 slot based on architecture and feature flags.
///
/// Usage:
/// ```rust
/// use nblf_queue::cfg_atomic_tagged128;
///
/// cfg_atomic_tagged128! {
///     use nblf_queue::core::slots::Tagged128;
/// }
/// ```
#[macro_export]
macro_rules! cfg_atomic_tagged128 {
    ($($item:item)*) => {
        $(
            #[cfg(any(target_has_atomic = "128", all(feature = "atomic-fallback", not(loom), not(shuttle))))]
            $item
        )*
    };
}

pub(crate) use sealed::Sealed;

#[cfg(all(not(shuttle), not(loom)))]
const MAX_SPINLOOP: usize = 1024;

pub(crate) struct Backoff {
    #[cfg(all(not(shuttle), not(loom)))]
    state: usize,
}

impl Backoff {
    pub fn new() -> Self {
        #[cfg(all(not(shuttle), not(loom)))]
        return Self { state: 1 };
        #[cfg(any(shuttle, loom))]
        return Self {};
    }

    pub fn backoff(&mut self) {
        #[cfg(all(not(shuttle), not(loom)))]
        {
            for _ in 0..self.state {
                crate::sync::hint::spin_loop();
            }
            self.state = (self.state * 2).min(MAX_SPINLOOP);
        }
        #[cfg(any(shuttle, loom))]
        crate::sync::thread::yield_now();
    }
}

pub(crate) fn prev(i: usize, size: usize) -> usize {
    (i + size - 1) % size
}

pub(crate) fn comp(i: usize, u: u64, j: usize, v: u64, w_max: u64) -> bool {
    if u == v {
        i < j
    } else {
        (v.wrapping_add(w_max).wrapping_sub(u)) % w_max < w_max / 2
    }
}

pub(crate) mod sealed {
    #[doc(hidden)]
    pub trait Sealed {}
}

/// # Safety:
/// width here is the bit-width taken up by the lower value
/// we assume that lower is already properly truncated
macro_rules! pack {
    (($upper:expr, $lower:expr): $width:expr) => {
        (($upper) << $width) | ($lower)
    };
}

/// # Safety:
/// width is the bit-width taken up by the lower value
/// the value as produced by pack!() with the correct parameters
macro_rules! unpack {
    (($packed:expr): $width:expr) => {{
        // make sure to evaluate passed exprs only once
        let width = $width;
        let packed_value = $packed;
        let upper = packed_value >> width;
        let lower = packed_value & ((1 << width) - 1);
        (upper, lower)
    }};
}

#[cfg(test)]
pub(crate) fn sign_erase(ptr: u64) -> u64 {
    ptr & ((1u64 << 48) - 1)
}

pub(crate) fn sign_extend(ptr: u64) -> u64 {
    if ptr & (1u64 << 47) != 0 {
        ptr | (!((1u64 << 48) - 1))
    } else {
        ptr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod tagged_ptr {
        use core::ptr::null;

        use super::{sign_erase, sign_extend};

        #[test]
        fn into_tagged() {
            let ptr = u64::MAX as *const u8;
            let count = 0xDEAD;
            let res = pack!((count, sign_erase(ptr as u64)): 48);
            assert_eq!(res, 0xDEAD_FFFF_FFFF_FFFF);

            let ptr2 = 0xDEAD_BEEF as *const u8;
            let res = pack!((count, sign_erase(ptr2 as u64)): 48);
            assert_eq!(res, 0xDEAD_0000_DEAD_BEEF);

            let ptr: *const u8 = null();
            assert_eq!(pack!((0, ptr as u64): 48), 0);
        }

        #[test]
        fn from_tagged() {
            let ptr = u64::MAX as *const u8;
            let count = 0xDEAD;
            let res = 0xDEAD_FFFF_FFFF_FFFF;

            let (v1, v2) = unpack!((res): 48);

            assert_eq!((v1, sign_extend(v2)), (count, ptr as u64));

            let ptr2 = 0xDEAD_BEEF as *const u8;
            let res = 0xDEAD_0000_DEAD_BEEF;

            let (v1, v2) = unpack!((res): 48);

            assert_eq!((v1, sign_extend(v2)), (count, ptr2 as u64));

            let ptr: *const u8 = null();
            assert_eq!(unpack!((0_u64): 48), (0, ptr as u64))
        }

        #[test]
        fn tagged() {
            let ptr = u64::MAX as *const u8;
            let ptr2 = 0xDEAD_BEEF as *const u8;
            let count = 0xDEAD;

            let (v1, v2) = unpack!((pack!((count, sign_erase(ptr as u64)): 48)): 48);

            assert_eq!((v1, sign_extend(v2)), (count, ptr as u64));

            let (v1, v2) = unpack!((pack!((count, sign_erase(ptr2 as u64)): 48)): 48);

            assert_eq!((v1, sign_extend(v2)), (count, ptr2 as u64));

            let data = &4242;
            let count = 42;
            let ptr = pack!((count, data as *const i32 as u64): 48);
            let (count_, data_): (_, u64) = unpack!((ptr): 48);
            assert_eq!(count, count_);
            // SAFETY:
            // ptr to data or data was not modified, if components_as_tagged + from_tagged work as intended
            assert_eq!(*data, unsafe { *(sign_extend(data_) as *const i32) });
        }
    }

    mod dword {
        use core::ptr::null;

        #[test]
        fn into_dword() {
            let ptr = u64::MAX as *const u8;
            let count = 0xDEAD;
            let res = pack!((count, ptr as u128): 64);
            assert_eq!(res, 0xDEAD_u128 << 64 | u64::MAX as u128);

            let ptr2 = 0xDEAD_BEEF as *const u8;
            let res = pack!((count, ptr2 as u128): 64);
            assert_eq!(res, 0xDEAD_u128 << 64 | 0xDEAD_BEEF_u128);

            let ptr: *const u8 = null();
            assert_eq!(pack!((0, ptr as u128): 64), 0);
        }

        #[test]
        fn from_dword() {
            let ptr = u64::MAX as *const u8;
            let count = 0xDEAD;
            let res = 0xDEAD_u128 << 64 | u64::MAX as u128;

            assert_eq!(unpack!((res): 64), (count, ptr as u128));

            let ptr2 = 0xDEAD_BEEF as *const u8;
            let res = 0xDEAD_u128 << 64 | 0xDEAD_BEEF_u128;

            assert_eq!(unpack!((res): 64), (count, ptr2 as u128));

            let ptr: *const u8 = null();
            assert_eq!(unpack!((0_u128): 64), (0, ptr as u128));
        }

        #[test]
        fn dword() {
            let ptr = u64::MAX as *const u8;
            let ptr2 = 0xDEAD_BEEF as *const u8;
            let count = 0xDEAD;

            assert_eq!(
                unpack!((pack!((count, ptr as u128): 64)): 64),
                (count, ptr as u128)
            );
            assert_eq!(
                unpack!((pack!((count, ptr2 as u128): 64)): 64),
                (count, ptr2 as u128)
            );

            let data = &4242;
            let count = 42;
            let val = pack!((count, data as *const i32 as *const u8 as u128): 64);
            let (count_, data_): (_, u128) = unpack!((val): 64);
            assert_eq!(count, count_);
            // Safety:
            // ptr to data or data was not modified, if components_as_tagged + from_tagged work as intended
            assert_eq!(unsafe { *(data_ as *const i32) }, *data);
        }
    }

    #[test]
    fn prev_() {
        assert_eq!(prev(9, 10), 8);
        assert_eq!(prev(0, 5), 4);
    }

    #[test]
    fn comp_() {
        // cells are part of the same round,
        // cell i is before j, if i < j
        assert!(comp(0, 0, 1, 0, u16::MAX as u64 + 1));
        assert!(!comp(1, 1, 0, 1, u16::MAX as u64 + 1));

        // cells are part of different rounds,
        // cell i is before cell j, if its count is "1 less" than js
        assert!(comp(0, 1, 1, 2, u16::MAX as u64 + 1));
        assert!(!comp(0, 1, 1, 0, u16::MAX as u64 + 1));
        assert!(comp(0, u16::MAX as u64, 1, 0, u16::MAX as u64 + 1));
    }
}

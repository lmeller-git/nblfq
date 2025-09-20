pub(crate) fn prev(i: usize, size: usize) -> usize {
    (i + size - 1) % size
}

pub(crate) fn comp(i: usize, u: u64, j: usize, v: u64, w_max: u64) -> bool {
    if u == v {
        i < j
    } else {
        (v + w_max - u) % w_max < w_max / 2
    }
}

// tagged ptr 64bit:
// |--16 bit--|----48 bit----|
//    count   |     ptr

#[cfg(feature = "tagged_ptr")]
pub(crate) fn components_as_tagged<T>(count: u64, ptr: *const T) -> u64 {
    debug_assert!(count <= u16::MAX as u64, "Count too large for 16-bit field");
    let ptr_non_extended = ptr as usize as u64 & ((1u64 << 48) - 1);
    (count << 48) | ptr_non_extended
}

#[cfg(feature = "tagged_ptr")]
pub(crate) fn components_from_tagged<T>(ptr: u64) -> (u64, *const T) {
    let count = ptr >> 48;
    let ptr_mask = (1u64 << 48) - 1;
    let raw_ptr = ptr & ptr_mask;
    (count, sign_extend(raw_ptr) as *const T)
}

fn sign_extend(ptr: u64) -> u64 {
    if ptr & (1u64 << 47) != 0 {
        ptr | (!((1u64 << 48) - 1))
    } else {
        ptr
    }
}

#[cfg(test)]
mod tests {

    use core::ptr::null;

    use super::*;

    #[test]
    fn into_tagged() {
        let ptr = u64::MAX as *const u8;
        let count = 0xDEAD;
        let res = components_as_tagged(count, ptr);
        assert_eq!(res, 0xDEAD_FFFF_FFFF_FFFF);

        let ptr2 = 0xDEAD_BEEF as *const u8;
        let res = components_as_tagged(count, ptr2);
        assert_eq!(res, 0xDEAD_0000_DEAD_BEEF);

        let ptr: *const u8 = null();
        assert_eq!(components_as_tagged(0, ptr), 0);
    }

    #[test]
    fn from_tagged() {
        let ptr = u64::MAX as *const u8;
        let count = 0xDEAD;
        let res = 0xDEAD_FFFF_FFFF_FFFF;

        assert_eq!(components_from_tagged(res), (count, ptr));

        let ptr2 = 0xDEAD_BEEF as *const u8;
        let res = 0xDEAD_0000_DEAD_BEEF;

        assert_eq!(components_from_tagged(res), (count, ptr2));

        let ptr: *const u8 = null();
        assert_eq!(components_from_tagged(0), (0, ptr))
    }

    #[test]
    fn tagged() {
        let ptr = u64::MAX as *const u8;
        let ptr2 = 0xDEAD_BEEF as *const u8;
        let count = 0xDEAD;

        assert_eq!(
            components_from_tagged(components_as_tagged(count, ptr)),
            (count, ptr)
        );
        assert_eq!(
            components_from_tagged(components_as_tagged(count, ptr2)),
            (count, ptr2)
        );

        let data = &4242;
        let count = 42;
        let ptr = components_as_tagged(count, data as *const i32);
        let (count_, data_): (_, *const i32) = components_from_tagged(ptr);
        assert_eq!(count, count_);
        assert_eq!(*data, unsafe { *data_ });
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

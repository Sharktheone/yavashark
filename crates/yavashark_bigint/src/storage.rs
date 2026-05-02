use std::{mem, ptr};
use std::rc::Rc;

/// If p2 and p3 are zero, then we have a Small BigInt (just one i64).
/// The sign is always stored in p1.
/// Small: when p2 and p3 are zero, p1 is the value of the BigInt (with the sign bit).
/// Large: when the last bit in p3 is set.
/// Dynamic: when p1 is is zero except for the sign bit.
#[repr(C)]
pub(crate) struct BigIntStorage {
    p1: i64,
    p2: u64,
    p3: Ptr,
}

const STORAGE_P1_MASK: u64 = 0x7FFF_FFFF_FFFF_FFFF;
const STORAGE_P3_MASK: u64 = 0xFFFF_FFFF_FFFF_FFFE;
const P3_LARGE_BIT: u64 = 0b1;

impl BigIntStorage {
    const fn new_small(val: i64) -> Self {
        Self {
            p1: val,
            p2: 0,
            p3: Ptr { val: 0 },
        }
    }

    const fn new_large(val: [u64; 3], sign: Sign) -> Self {
        let sign = sign as u64;

        let p1 = ((val[0] & STORAGE_P1_MASK) | (sign << 63)) as i64;



        Self {
            p1,
            p2: val[1],
            p3: Ptr { val: (val[2] & STORAGE_P3_MASK) | P3_LARGE_BIT },
        }
    }

    fn new_dynamic(val: Rc<[u64]>, sign: Sign) -> Self {
        unsafe {
            Self::new_dynamic_raw(Rc::into_raw(val), sign)
        }
    }

    const unsafe fn new_dynamic_raw(ptr: *const [u64], sign: Sign) -> Self {
        let len = unsafe { (&*ptr).len() } as u64;
        let ptr = unsafe { (&*ptr).as_ptr() };

        let sign_bit = match sign {
            Sign::Positive => 0,
            Sign::Negative => 1,
        };
        Self {
            p1: (sign_bit << 63),
            p2: len,
            p3: Ptr { ptr },
        }
    }

    const fn sign(&self) -> Sign {
        if self.p1 < 0 {
            Sign::Negative
        } else {
            Sign::Positive
        }
    }

    const fn is_small(&self) -> bool {
        self.p2 == 0 && unsafe { self.p3.val } == 0
    }

    const fn is_large(&self) -> bool {
        (unsafe { self.p3.val } & P3_LARGE_BIT) != 0
    }

    const fn is_dynamic(&self) -> bool {
        let p3_val = unsafe { self.p3.val };
        p3_val != 0 && (p3_val & P3_LARGE_BIT) == 0
    }


    const unsafe fn small_unchecked(&self) -> i64 {
        self.p1
    }

    const unsafe fn large_unchecked(&self) -> [u64; 3] {
        let val0 = (self.p1 as u64) & STORAGE_P1_MASK;
        let val1 = self.p2;
        let val2 = unsafe { self.p3.val };
        [val0, val1, val2]
    }

    const unsafe fn dynamic_unchecked(&self) -> (*const u64, usize) {
        let ptr = unsafe { self.p3.ptr };
        let len = self.p2;
        (ptr, len as usize)
    }

    const fn small(&self) -> Option<i64> {
        if self.is_small() {
            Some(unsafe { self.small_unchecked() })
        } else {
            None
        }
    }

    const fn large(&self) -> Option<[u64; 3]> {
        if self.is_large() {
            Some(unsafe { self.large_unchecked() })
        } else {
            None
        }
    }

    const fn dynamic(&self) -> Option<(*const u64, usize)> {
        if self.is_dynamic() {
            Some(unsafe { self.dynamic_unchecked() })
        } else {
            None
        }
    }

    const fn as_repr(&'_ self) -> (BigIntRepr<'_>, Sign) {
        let sign = self.sign();

        let repr = if let Some(small) = self.small() {
            BigIntRepr::Small(small)
        } else if let Some(large) = self.large() {
            BigIntRepr::Large(large)
        } else if let Some((ptr, len)) = self.dynamic() {
            let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
            BigIntRepr::Dynamic(slice)
        } else {
            unreachable!()
        };

        (repr, sign)
    }
}


impl Drop for BigIntStorage {
    fn drop(&mut self) {
        if self.is_dynamic() {
            let (ptr, len) = unsafe { self.dynamic_unchecked() };

            let ptr = ptr::slice_from_raw_parts(ptr, len);

            let rc = unsafe { Rc::from_raw(ptr) };

            drop(rc);
        }
    }
}

impl Clone for BigIntStorage {
    fn clone(&self) -> Self {
        if let Some((ptr, len)) = self.dynamic() {
            let ptr = ptr::slice_from_raw_parts(ptr, len);
            let rc = unsafe { Rc::from_raw(ptr) };
            let rc_clone = Rc::clone(&rc);
            mem::forget(rc);
            mem::forget(rc_clone);
        }

        Self {
            p1: self.p1,
            p2: self.p2,
            p3: self.p3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Sign {
    Positive = 0,
    Negative = 1,
}



#[derive(Clone, Copy)]
union Ptr {
    val: u64,
    ptr: *const u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BigIntRepr<'a> {
    Small(i64),
    Large([u64; 3]),
    Dynamic(&'a [u64]),
}


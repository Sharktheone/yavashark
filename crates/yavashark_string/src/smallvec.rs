use std::fmt::{Debug, Formatter};
use std::{fmt, mem};
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use crate::uz::{DoubleU4, UsizeSmall, UZ_BYTES};

/// A 23 byte sized Vector that has a length and capacity of 60 bits (7.5bytes) each
#[repr(packed)]
pub struct SmallVec<T> {
    pub(crate) len_cap: SmallVecLenCap,
    pub(crate) ptr: NonNull<T>
}

impl<T> Drop for SmallVec<T> {
    fn drop(&mut self) {
        unsafe {
            let mut vec = self.to_vec_ref();
            ManuallyDrop::drop(&mut vec);
        }

    }
}


impl Clone for SmallVec<u8> {
    fn clone(&self) -> Self {
        let vec = self.slice().to_vec();

        #[allow(clippy::expect_used)]
        Self::new(vec).expect("unreachable")
    }
}

impl Debug for SmallVec<u8> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self.slice(), f)
    }
}

impl<T> Deref for SmallVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.slice()
    }
}

impl DerefMut for SmallVec<u8> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            let len = self.len_cap.len();
            let ptr = self.ptr.as_ptr();
            std::slice::from_raw_parts_mut(ptr, len)
        }
    }
}

impl Default for SmallVec<u8> {
    fn default() -> Self {
        #[allow(clippy::expect_used)]
        Self::new(Vec::new()).expect("unreachable")
    }
}


impl<T> SmallVec<T> {
    pub fn new(mut vec: Vec<T>) -> Option<Self> {
        let len = vec.len();
        let cap = vec.capacity();

        let len_cap = SmallVecLenCap::new(len, cap)?;

        let ptr = NonNull::new(vec.as_mut_ptr())?;

        mem::forget(vec);

        Some(Self {
            len_cap,
            ptr
        })
    }

    unsafe fn to_vec_ref(&self) -> ManuallyDrop<Vec<T>> {
        let len = self.len_cap.len();
        let cap = self.len_cap.cap();

        let ptr = self.ptr.as_ptr();

        ManuallyDrop::new(Vec::from_raw_parts(ptr, len, cap))
    }

    pub fn into_vec(self) -> Vec<T> {
        unsafe {
            let vec = self.to_vec_ref();
            ManuallyDrop::into_inner(vec)
        }
    }

    pub fn slice(&self) -> &[T] {
        unsafe {
            let len = self.len_cap.len();
            let ptr = self.ptr.as_ptr();
            std::slice::from_raw_parts(ptr, len)
        }
    }
}


#[repr(packed)]
pub struct SmallVecLenCap {
    len: UsizeSmall,
    shared: DoubleU4,
    cap: UsizeSmall,
}

impl SmallVecLenCap {
    pub fn new(len: usize, cap: usize) -> Option<Self> {
        if len > 0x7F_FF_FF_FF || cap > 0x7F_FF_FF_FF {
            return None;
        }

        let (len, len_shared) = uz_to_bytes(len);
        let (cap, cap_shared) = uz_to_bytes(cap);

        let shared = DoubleU4::new(len_shared, cap_shared)?;

        Some(Self {
            len: UsizeSmall::from_le_bytes(len),
            shared,
            cap: UsizeSmall::from_le_bytes(cap),
        })
    }

    pub fn len(&self) -> usize {
        let without_shared = self.len.to_usize();

        let shared = self.shared.first();

        without_shared | (shared as usize) << (8 * UZ_BYTES)
    }

    pub fn cap(&self) -> usize {
        let without_shared = self.cap.to_usize();

        let shared = self.shared.second();

        without_shared | (shared as usize) << (8 * UZ_BYTES)
    }
}

pub fn uz_to_bytes(uz: usize) -> ([u8; UZ_BYTES], u8) {
    let bytes_full = uz.to_le_bytes();

    let most_worth = bytes_full[UZ_BYTES];

    let mut res = [0; UZ_BYTES];

    for i in 0..UZ_BYTES {
        res[i] = bytes_full[UZ_BYTES - 1 - i];
    }

    (res, most_worth)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_vec_len_cap() {
        let small_vec_len_cap = SmallVecLenCap::new(0, 0).unwrap();

        assert_eq!(small_vec_len_cap.len(), 0);
        assert_eq!(small_vec_len_cap.cap(), 0);

        let small_vec_len_cap = SmallVecLenCap::new(1, 1).unwrap();

        assert_eq!(small_vec_len_cap.len(), 1);
        assert_eq!(small_vec_len_cap.cap(), 1);

        let small_vec_len_cap = SmallVecLenCap::new(0x7F_FF_FF_FF, 0x7F_FF_FF_FF).unwrap();

        assert_eq!(small_vec_len_cap.len(), 0x7F_FF_FF_FF);
        assert_eq!(small_vec_len_cap.cap(), 0x7F_FF_FF_FF);
    }
}

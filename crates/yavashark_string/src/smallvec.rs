use crate::uz::{DoubleU4, UsizeSmall, UZ_BYTES};
use std::fmt::{Debug, Formatter};
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::{fmt, mem};

/// A 23 byte sized Vector that has a length and capacity of 60 bits (7.5bytes) each
#[repr(packed)]
//TODO: this should be Rc-able, but we can only do this once UniqueRc is stable
pub struct SmallVec<T> {
    pub(crate) len_cap: SmallVecLenCap,
    pub(crate) ptr: NonNull<T>,
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
    pub fn new(vec: Vec<T>) -> Option<Self> {
        let mut vec = ManuallyDrop::new(vec);

        let len = vec.len();
        let cap = vec.capacity();

        let len_cap = SmallVecLenCap::new(len, cap)?;

        let ptr = NonNull::new(vec.as_mut_ptr())?;

        Some(Self { len_cap, ptr })
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

    fn vec_wrapper(&mut self) -> VecWrapper<T> {
        unsafe { VecWrapper(self.to_vec_ref(), self) }
    }

    pub fn slice(&self) -> &[T] {
        unsafe {
            let len = self.len_cap.len();
            let ptr = self.ptr.as_ptr();
            std::slice::from_raw_parts(ptr, len)
        }
    }

    pub fn into_raw_parts(self) -> (NonNull<T>, usize, usize) {
        let this = ManuallyDrop::new(self);

        let ptr = this.ptr;
        let len = this.len_cap.len();
        let cap = this.len_cap.cap();

        (ptr, len, cap)
    }

    pub fn push(&mut self, val: T) {
        let mut wrap = self.vec_wrapper();

        wrap.push(val);
    }
}

impl<T: Clone> SmallVec<T> {
    pub fn extend_from_slice(&mut self, slice: &[T]) {
        let mut wrap = self.vec_wrapper();

        wrap.extend_from_slice(slice);
    }
}

struct VecWrapper<'a, T>(ManuallyDrop<Vec<T>>, &'a mut SmallVec<T>);

impl<T> Deref for VecWrapper<'_, T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for VecWrapper<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Drop for VecWrapper<'_, T> {
    fn drop(&mut self) {
        let len = self.0.len();
        let cap = self.0.capacity();

        #[allow(clippy::expect_used)]
        let len_cap = SmallVecLenCap::new(len, cap).expect("too many items in vec!");

        #[allow(clippy::expect_used)]
        let ptr = NonNull::new(self.0.as_mut_ptr()).expect("unreachable");

        self.1.len_cap = len_cap;
        self.1.ptr = ptr;
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

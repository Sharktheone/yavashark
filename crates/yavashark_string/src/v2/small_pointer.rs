use std::marker::PhantomData;
use std::num::NonZeroUsize;
use std::ops::Deref;
use std::ptr::NonNull;

#[derive(Debug, PartialEq, Eq)]
#[repr(Rust, packed)]
pub struct SmallPointer<T> {
    #[cfg(any(target_pointer_width = "64", target_pointer_width = "32"))]
    p1: u32,
    #[cfg(any(target_pointer_width = "64", target_pointer_width = "16"))]
    p2: u16,

    _marker: PhantomData<NonNull<T>>,
}

impl<T> Clone for SmallPointer<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for SmallPointer<T> {}

impl<T> SmallPointer<T> {
    pub fn new(ptr: NonNull<T>) -> Self {
        let addr = ptr.expose_provenance().get();

        #[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
        let p1 = addr as u32;
        #[cfg(target_pointer_width = "64")]
        let p2 = (addr >> 32) as u16;
        #[cfg(target_pointer_width = "16")]
        let p2 = addr as u16;

        Self {
            #[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
            p1,
            #[cfg(any(target_pointer_width = "16", target_pointer_width = "64"))]
            p2,
            _marker: PhantomData,
        }
    }

    pub fn get(self) -> NonNull<T> {
        #[cfg(target_pointer_width = "16")]
        let addr = self.p2 as usize;

        #[cfg(target_pointer_width = "32")]
        let addr = self.p1 as usize;

        #[cfg(target_pointer_width = "64")]
        let addr = (self.p2 as usize) << 32 | (self.p1 as usize);

        let addr = unsafe { NonZeroUsize::new_unchecked(addr) };

        NonNull::with_exposed_provenance(addr)
    }
}

pub struct Gc<T> {
    ptr: SmallPointer<T>,
}

impl<T> Clone for Gc<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Gc<T> {}

impl<T> Deref for Gc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.get().as_ref() }
    }
}

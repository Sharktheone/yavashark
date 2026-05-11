use std::mem::{MaybeUninit};
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::ptr::{NonNull};

#[repr(C, align(8))]
pub struct NativeWrapper<T: ?Sized> {
    data: T,
}

impl<T> NativeWrapper<T> {
    const DATA_ALIGNMENT_REQUIREMENT: () = assert!(
        align_of::<T>() <= 8,
        "Alignment of T must be <= 8; consider wrapping in a Box<T>"
    );

    pub const fn new_sized(data: T) -> Self {
        let () = Self::DATA_ALIGNMENT_REQUIREMENT;

        Self { data }
    }

    pub unsafe fn initialize_from_ref(this: NonNull<MaybeUninit<Self>>, data: T) {
        unsafe {
            (*(*this.as_ptr()).as_mut_ptr()).data = data;
        }
    }

    pub unsafe fn initialize_from_ref_cb<R>(this: NonNull<MaybeUninit<Self>>, data: impl FnOnce(NonNull<MaybeUninit<T>>) -> R) -> R {
        unsafe {
            let native = &raw mut (*(*this.as_ptr()).as_mut_ptr()).data;


            data(NonNull::new_unchecked(native).cast())
        }
    }
}

impl<T: ?Sized> Deref for NativeWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T: ?Sized> DerefMut for NativeWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T: ?Sized> AsRef<T> for NativeWrapper<T> {
    fn as_ref(&self) -> &T {
        &self.data
    }
}

impl<T: ?Sized> AsMut<T> for NativeWrapper<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

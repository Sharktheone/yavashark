use std::ops::{Deref, DerefMut};

#[repr(C, align(8))]
pub struct NativeWrapper<T: ?Sized> {
    data: T,
}


impl<T> NativeWrapper<T> {

    const DATA_ALIGNMENT_REQUIREMENT: () = assert!(
        align_of::<T>() <= 8,
        "Alignment of T must be <= 8; consider wrapping in a Box<T>"
    );

    pub const fn new(data: T) -> Self {
        let () = Self::DATA_ALIGNMENT_REQUIREMENT;

        Self { data }
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

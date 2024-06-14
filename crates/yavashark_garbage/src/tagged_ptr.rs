use std::ptr::NonNull;

#[cfg(all(not(miri), not(feature = "easy_debug")))]
#[repr(transparent)]
#[derive(Debug)]
pub struct TaggedPtr<T> {
    #[cfg(not(deb))]
    ptr: NonNull<[(); 0]>,
    _marker: std::marker::PhantomData<T>,
}

#[cfg(any(miri, feature = "easy_debug"))]
#[derive(Debug)]
pub struct TaggedPtr<T> {
    ptr: NonNull<T>,
    tag: bool,
    _marker: std::marker::PhantomData<T>,
}


impl<T> Clone for TaggedPtr<T> {
    fn clone(&self) -> Self { *self }
}

impl<T> Copy for TaggedPtr<T> {}


#[allow(clippy::missing_const_for_fn)]
impl<T> TaggedPtr<T> {
    #[cfg(all(not(miri), not(feature = "easy_debug")))]
    const IS_ALIGNED_ENOUGH: bool = {
        let alignment = if std::mem::align_of::<T>() > 2 {
            0
        } else {
            1
        };

        #[allow(clippy::no_effect)]
        [0][alignment];

        true
    };


    /// Mask, so we only keep the lowest bit
    #[cfg(all(not(miri), not(feature = "easy_debug")))]
    const MASK: usize = 0b1;

    /// # Panics
    /// - Panics if the pointer is not aligned enough
    /// - Panics if the pointer is null
    #[cfg(all(not(miri), not(feature = "easy_debug")))]
    pub fn new(ptr: NonNull<T>, tag: bool) -> Self {
        assert!(Self::IS_ALIGNED_ENOUGH);
        let ptr = ptr.as_ptr() as usize;

        assert_eq!(ptr & Self::MASK, 0, "Pointer is not aligned enough");

        let ptr = ptr | usize::from(tag);

        #[allow(clippy::expect_used)]
            let ptr = NonNull::new(ptr as *mut _).expect("Pointer is null");


        Self {
            ptr,
            _marker: std::marker::PhantomData,
        }
    }

    #[cfg(any(miri, feature = "easy_debug"))]
    pub fn new(ptr: NonNull<T>, tag: bool) -> Self {
        Self {
            ptr: ptr.cast(),
            tag,
            _marker: std::marker::PhantomData,
        }
    }

    #[cfg(all(not(miri), not(feature = "easy_debug")))]
    pub(crate) fn tag(&self) -> bool {
        self.ptr.as_ptr() as usize & Self::MASK != 0
    }

    #[cfg(any(miri, feature = "easy_debug"))]
    pub(crate) fn tag(&self) -> bool {
        self.tag
    }


    #[cfg(all(not(miri), not(feature = "easy_debug")))]
    pub(crate) fn ptr(&self) -> NonNull<T> {
        let ptr = self.ptr.as_ptr() as usize & !Self::MASK;
        unsafe { NonNull::new_unchecked(ptr as *mut _) }
    }

    #[cfg(any(miri, feature = "easy_debug"))]
    pub(crate) fn ptr(&self) -> NonNull<T> {
        self.ptr.cast()
    }

    pub fn as_ptr(&self) -> *mut T {
        self.ptr().as_ptr()
    }

    #[cfg(all(not(miri), not(feature = "easy_debug")))]
    pub const fn cast<U>(self) -> TaggedPtr<U> {
        // SAFETY: `self` is a `NonNull` pointer which is necessarily non-null
        TaggedPtr { ptr: self.ptr, _marker: std::marker::PhantomData }
    }

    #[cfg(any(miri, feature = "easy_debug"))]
    pub const fn cast<U>(self) -> TaggedPtr<U> {
        // SAFETY: `self` is a `NonNull` pointer which is necessarily non-null
        TaggedPtr { ptr: self.ptr.cast(), tag: self.tag, _marker: std::marker::PhantomData }
    }
}


impl<T> From<NonNull<T>> for TaggedPtr<T> {
    fn from(value: NonNull<T>) -> Self {
        Self::new(value, false)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tagged_ptr() {
        let ptr = NonNull::new(&mut 1337).unwrap();
        let tagged = TaggedPtr::new(ptr, true);

        assert!(tagged.tag());
        assert_eq!(tagged.ptr(), ptr);
        assert_eq!(unsafe { *tagged.ptr().as_ptr() }, 1337);
    }

    #[test]
    fn test2_tagged_ptr() {
        let ptr = NonNull::new(&mut 1337).unwrap();
        let tagged = TaggedPtr::new(ptr, false);

        assert!(!tagged.tag());
        assert_eq!(tagged.ptr(), ptr);
        assert_eq!(unsafe { *tagged.ptr().as_ptr() }, 1337);
    }
}
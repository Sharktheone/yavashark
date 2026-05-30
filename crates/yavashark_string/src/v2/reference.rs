use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::ptr;
use crate::v2::YSString;

pub struct YSStringRef<'a> {
    pub(crate) inner: ManuallyDrop<YSString>,
    pub(crate) _marker: std::marker::PhantomData<&'a YSString>,
}


impl Deref for YSStringRef<'_> {
    type Target = YSString;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}


impl Clone for YSStringRef<'_> {
    fn clone(&self) -> Self {
        self.copy()
    }
}

impl YSStringRef<'_> {
    pub const fn copy(&self) -> Self {
        unsafe {
            ptr::read(self)
        }
    }

    pub fn to_owned(&self) -> YSString {
        self.deref().clone()
    }

    pub fn slice(&self, start: u32, end: u32) -> Option<Self> {
        if start > end || end > self.len() {
            return None;
        }

        let mut inner = self.copy();

        //TODO

        Some(inner)
    }
}

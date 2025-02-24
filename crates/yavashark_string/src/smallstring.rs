use crate::smallvec::SmallVec;
use std::mem;

pub struct SmallString {
    inner: SmallVec<u8>,
}

impl SmallString {
    pub fn new() -> Option<Self> {
        Some(Self {
            inner: SmallVec::new(Vec::new())?,
        })
    }

    pub fn from_string(mut string: String) -> Option<Self> {
        Some(Self {
            inner: SmallVec::new(string.into_bytes())?,
        })
    }

    pub fn into_string(self) -> String {
        unsafe {
            let str = String::from_raw_parts(
                self.inner.ptr.as_ptr(),
                self.inner.len_cap.len(),
                self.inner.len_cap.cap(),
            );
            
            mem::forget(self.inner); // we transferred ownership to the String
            
            str
        }
    }
}

impl std::ops::Deref for SmallString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        #[allow(clippy::expect_used)]
        std::str::from_utf8(&self.inner).expect(
            "SmallString contained invalid utf8 (impossible since it was created from a String)",
        )
    }
}

impl std::ops::DerefMut for SmallString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[allow(clippy::expect_used)]
        std::str::from_utf8_mut(&mut self.inner).expect(
            "SmallString contained invalid utf8 (impossible since it was created from a String)",
        )
    }
}

use crate::smallvec::SmallVec;
use std::mem;
use std::rc::Rc;

#[derive(Debug, Clone, Default)]
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

    pub fn as_str(&self) -> &str {
        #[allow(clippy::expect_used)]
        std::str::from_utf8(&self.inner).expect(
            "SmallString contained invalid utf8 (impossible since it was created from a String)",
        )
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

    pub fn into_rc(self) -> Rc<str> {
        let vec = self.into_string().into_boxed_str();

        Rc::from(vec)
    }
    
    pub fn into_rc_if_fit(self) -> Result<Rc<str>, Self> {
        let vec = self.into_string();
        
        if vec.capacity() != vec.len() {
            #[allow(clippy::expect_used)]
            return Err(Self::from_string(vec).expect("unreachable"));
        }
        
        Ok(Rc::from(vec.into_boxed_str()))
    }
    
    pub fn copy_rc(&self) -> Rc<str> {
        let vec = self.as_str().to_string().into_boxed_str();

        Rc::from(vec)
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

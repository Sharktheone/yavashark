use crate::smallvec::SmallVec;
use std::fmt::Display;
use std::hash::Hash;
use std::mem;
use std::rc::Rc;

#[derive(Debug, Clone, Default)]
pub struct SmallString {
    inner: SmallVec<u8>,
}

impl Hash for SmallString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let slice = self.as_str();

        slice.hash(state);
    }
}

impl Display for SmallString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl SmallString {
    pub fn new() -> Self {
        #[allow(clippy::expect_used)]
        Self {
            inner: SmallVec::new(Vec::new()).expect("unreachable"),
        }
    }

    pub fn from_string(mut string: String) -> Result<Self, String> {
        Ok(Self {
            inner: SmallVec::new(string.into_bytes()).map_err(|vec| {
                unsafe {
                    // SAFETY: `vec` is a valid Vec<u8> since it was created from a String
                    String::from_utf8_unchecked(vec)
                }
            })?,
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
            let (ptr, len, cap) = self.inner.into_raw_parts();

            String::from_raw_parts(ptr.as_ptr(), len, cap)
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

    pub fn push(&mut self, c: char) {
        self.inner.push(c as u8);
    }

    pub fn push_str(&mut self, s: &str) {
        self.inner.extend_from_slice(s.as_bytes());
    }
    
    pub fn pop(&mut self) {
        self.inner.pop()
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

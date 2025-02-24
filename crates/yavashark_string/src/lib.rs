#![allow(unused)]

mod smallstring;
pub(crate) mod smallvec;
pub(crate) mod uz;

use crate::smallstring::SmallString;
use crate::smallvec::SmallVecLenCap;
use crate::uz::UZ_BYTES;
use std::borrow::Cow;
use std::cell::UnsafeCell;
use std::cmp::Ordering;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

pub struct YSString {
    inner: UnsafeCell<InnerString>,
}

enum InnerString {
    Inline(InlineString),
    Static(&'static str),
    Owned(SmallString),
    Rc(Rc<str>),
    Rope(RopeStr),
}

#[repr(packed)]
#[derive(Debug, Clone, Copy)]
pub struct InlineString {
    len: InlineLen,
    data: [u8; 23],
}

impl PartialEq for InlineString {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.data[0..self.len()].eq(&other.data[0..other.len()])
    }
}

impl Eq for InlineString {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InlineLen {
    Empty = 0,
    Len1,
    Len2,
    Len3,
    Len4,
    Len5,
    Len6,
    Len7,
    Len8,
    Len9,
    Len10,
    Len11,
    Len12,
    Len13,
    Len14,
    Len15,
    Len16,
    Len17,
    Len18,
    Len19,
    Len20,
    Len21,
    Len22,
    Len23,
}

#[derive(Clone)]
struct RopeStr {
    left: Rc<YSString>,
    right: Rc<YSString>,
}

impl InlineString {
    const fn begin(&self) -> [u8; UZ_BYTES] {
        let mut begin = [0; UZ_BYTES];

        begin[0] = self.len as u8;

        begin
    }

    const fn len(&self) -> usize {
        self.len as usize
    }

    fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.data[0..self.len()]) }
    }

    fn as_mut_str(&mut self) -> &mut str {
        let len = self.len();

        unsafe { std::str::from_utf8_unchecked_mut(&mut self.data[0..len]) }
    }

    fn try_from_string(str: String) -> Result<Self, String> {
        let Some(len) = InlineLen::from_usize(str.len()) else {
            return Err(str);
        };

        let mut data = [0; 23];

        if len != InlineLen::Empty {
            data[0..len as usize].copy_from_slice(str.as_bytes());
        }

        Ok(Self { len, data })
    }
}

impl InlineLen {
    #[must_use]
    pub const fn from_usize(len: usize) -> Option<Self> {
        Some(match len {
            0 => Self::Empty,
            1 => Self::Len1,
            2 => Self::Len2,
            3 => Self::Len3,
            4 => Self::Len4,
            5 => Self::Len5,
            6 => Self::Len6,
            7 => Self::Len7,
            8 => Self::Len8,
            9 => Self::Len9,
            10 => Self::Len10,
            11 => Self::Len11,
            12 => Self::Len12,
            13 => Self::Len13,
            14 => Self::Len14,
            15 => Self::Len15,
            16 => Self::Len16,
            17 => Self::Len17,
            18 => Self::Len18,
            19 => Self::Len19,
            20 => Self::Len20,
            21 => Self::Len21,
            22 => Self::Len22,
            23 => Self::Len23,
            _ => return None,
        })
    }
}

impl RopeStr {
    fn len(&self) -> usize {
        self.left.len() + self.right.len()
    }

    fn as_string_opt_rope(&self) -> String {
        let mut str = String::with_capacity(self.len());

        if Rc::strong_count(&self.left) == 1 {
            str.push_str(&self.left.as_str_no_rope_fix()); // if we only have one reference, we can avoid cloning
        } else {
            str.push_str(self.left.as_str());
        }

        if Rc::strong_count(&self.right) == 1 {
            str.push_str(&self.right.as_str_no_rope_fix()); // if we only have one reference, we can avoid cloning
        } else {
            str.push_str(self.right.as_str());
        }

        str
    }

    fn as_string(&self) -> String {
        let mut str = String::with_capacity(self.len());

        str.push_str(self.left.as_str());
        str.push_str(self.right.as_str());

        str
    }
}

impl Default for YSString {
    fn default() -> Self {
        Self::new()
    }
}

impl YSString {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(InnerString::Inline(InlineString {
                len: InlineLen::Empty,
                data: [0; 23],
            })),
        }
    }

    #[must_use]
    pub const fn new_static(str: &'static str) -> Self {
        Self {
            inner: UnsafeCell::new(InnerString::Static(str)),
        }
    }

    #[must_use]
    pub fn from_string(str: String) -> Self {
        let str = match InlineString::try_from_string(str) {
            Ok(inline) => InnerString::Inline(inline),
            Err(str) => InnerString::Owned(SmallString::from_string(str).unwrap_or_default()),
        };

        Self {
            inner: UnsafeCell::new(str),
        }
    }

    #[must_use]
    pub const fn from_rc(rc: Rc<str>) -> Self {
        Self {
            inner: UnsafeCell::new(InnerString::Rc(rc)),
        }
    }

    #[must_use]
    pub const fn from_rope(left: Rc<Self>, right: Rc<Self>) -> Self {
        Self {
            inner: UnsafeCell::new(InnerString::Rope(RopeStr { left, right })),
        }
    }

    #[must_use]
    const fn from_inline(inline: InlineString) -> Self {
        Self {
            inner: UnsafeCell::new(InnerString::Inline(inline)),
        }
    }

    #[must_use]
    const fn from_rope_str(rope: RopeStr) -> Self {
        Self {
            inner: UnsafeCell::new(InnerString::Rope(rope)),
        }
    }

    fn inner(&self) -> &InnerString {
        unsafe { &*self.inner.get() }
    }

    #[allow(clippy::mut_from_ref)]
    unsafe fn inner_mut_ref(&self) -> &mut InnerString {
        unsafe { &mut *self.inner.get() }
    }

    fn inner_mut(&mut self) -> &mut InnerString {
        unsafe { &mut *self.inner.get() }
    }

    pub fn len(&self) -> usize {
        match self.inner() {
            InnerString::Inline(inline) => inline.len(),
            InnerString::Static(static_str) => static_str.len(),
            InnerString::Owned(owned) => owned.len(),
            InnerString::Rc(rc) => rc.len(),
            InnerString::Rope(rope) => rope.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn as_str(&self) -> &str {
        match self.inner() {
            InnerString::Inline(inline) => inline.as_str(),
            InnerString::Static(static_str) => static_str,
            InnerString::Owned(owned) => owned,
            InnerString::Rc(rc) => rc,
            InnerString::Rope(rope) => {
                let str = rope.as_string_opt_rope(); // since we drop the RopeStr afterward we don't need to fix the rope

                let inner = unsafe { self.inner_mut_ref() };

                let str = SmallString::from_string(str).unwrap_or_default();

                *inner = InnerString::Owned(str);

                let InnerString::Owned(str) = inner else {
                    return ""; // unreachable, but don't crash
                };

                str
            }
        }
    }

    fn as_str_no_rope_fix(&self) -> Cow<str> {
        Cow::Borrowed(match self.inner() {
            InnerString::Inline(inline) => inline.as_str(),
            InnerString::Static(static_str) => static_str,
            InnerString::Owned(owned) => owned,
            InnerString::Rc(rc) => rc,
            InnerString::Rope(rope) => return Cow::Owned(rope.as_string_opt_rope()), // we don't need to fix the rope here
        })
    }

    /// This will clone the string without actually cloning it
    fn clone_no_copy(&mut self) -> Self {
        match self.inner_mut() {
            InnerString::Inline(ref inline) => Self::from_inline(*inline),
            InnerString::Static(static_str) => Self::new_static(static_str),
            InnerString::Owned(owned) => {
                let owned = mem::take(owned);

                let rc = owned.into_rc();

                *self.inner_mut() = InnerString::Rc(Rc::clone(&rc));

                Self::from_rc(rc)
            }
            InnerString::Rc(ref rc) => Self::from_rc(Rc::clone(rc)),
            InnerString::Rope(ref rope) => Self::from_rope_str(rope.clone()),
        }
    }

    ///try to use `clone_no_copy` if possible!
    fn clone_ref(&self) -> Self {
        match unsafe { self.inner_mut_ref() } {
            InnerString::Inline(ref inline) => Self::from_inline(*inline),
            InnerString::Static(static_str) => Self::new_static(static_str),
            InnerString::Owned(owned) => {
                let owned_str = mem::take(owned);

                unsafe {
                    let rc = match owned_str.into_rc_if_fit() {
                        // we need to use the into_rc_fit method as the into_rc would create UB!
                        Ok(rc) => rc, // we don't need to shrink, which means potential references will still be valid
                        Err(str) => {
                            *owned = str;

                            let rc = owned.copy_rc();

                            return Self::from_rc(rc);
                        }
                    };
                    *self.inner_mut_ref() = InnerString::Rc(Rc::clone(&rc));
                    Self::from_rc(rc)
                }
            }
            InnerString::Rc(ref rc) => Self::from_rc(Rc::clone(rc)),
            InnerString::Rope(ref rope) => Self::from_rope_str(rope.clone()),
        }
    }

    fn as_mut_str(&mut self) -> &mut str {
        let inner = self.inner_mut();

        let owned = match inner {
            InnerString::Inline(inline) => return inline.as_mut_str(),
            InnerString::Static(static_str) => {
                SmallString::from_string(static_str.to_string()).unwrap_or_default()
            }
            InnerString::Owned(owned) => owned.clone(),
            InnerString::Rc(rc) => SmallString::from_string(rc.to_string()).unwrap_or_default(),
            InnerString::Rope(rope) => {
                SmallString::from_string(rope.as_string_opt_rope()).unwrap_or_default()
            }
        };
        *inner = InnerString::Owned(owned);

        let InnerString::Owned(owned) = inner else {
            unreachable!()
        };

        owned
    }
}

impl Deref for YSString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl DerefMut for YSString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_str()
    }
}

impl PartialEq for YSString {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for YSString {}

impl PartialOrd for YSString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for YSString {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

#[test]
fn str_size() {
    dbg!(size_of::<InnerString>());
    dbg!(align_of::<InnerString>());
    dbg!(size_of::<SmallVecLenCap>());
    dbg!(size_of::<SmallString>());

    dbg!(size_of::<String>());
    dbg!(size_of::<Option<String>>());

    let str = "Hello, World!".to_owned();

    let str = str.into_boxed_str();

    let rcd: Rc<str> = Rc::from(str);

    let rcd2 = rcd.clone();

    dbg!(rcd, rcd2);

    // assert_eq!(std::mem::size_of::<InnerString>(), std::mem::size_of::<String>());
}

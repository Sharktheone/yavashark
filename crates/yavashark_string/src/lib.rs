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
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{Add, AddAssign, Deref, DerefMut};
use std::rc::Rc;

pub struct YSString {
    inner: UnsafeCell<InnerString>,
}

impl Display for YSString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Debug for YSString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Hash for YSString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

enum InnerString {
    Inline(InlineString),
    Static(&'static str),
    Owned(SmallString),
    #[cold]
    #[allow(clippy::box_collection)]
    // we can't use just String here, as the InnerString would not be 24 bytes anymore (size_of::<InnerString> != size_of::<String>)
    BoxedOwned(Box<String>), //This is because SmallString can "only" hold up to 2^60 bytes
    Rc(Rc<str>),
    Rope(RopeStr),
}

#[repr(Rust, packed)]
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
    //TODO: we can theoretically also have the last byte here if the length is 24
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
    inner: Rc<RopeStrInner>,
}

pub struct RopeStrInner {
    left: YSString, //TODO: it would be better to have another enum here, where we don't have the 24 byte limit (so we can use the std String)
    right: YSString,
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

    fn try_from_string(str: &str) -> Option<Self> {
        let len = InlineLen::from_usize(str.len())?;

        let mut data = [0; 23];

        if len != InlineLen::Empty {
            data[0..len as usize].copy_from_slice(str.as_bytes());
        }

        Some(Self { len, data })
    }
    pub fn push(&mut self, ch: char) -> Option<Result<SmallString, String>> {
        let prev_len = self.len();

        let Some(len) = InlineLen::from_usize(prev_len + 1) else {
            let mut string = self.as_str().to_string();
            string.push(ch);

            return Some(SmallString::from_string(string));
        };

        self.data[prev_len] = ch as u8;

        None
    }

    fn push_str(&mut self, str: YSString) -> Option<RopeStr> {
        let prev_len = self.len();
        let new_len = prev_len + str.len();

        let Some(len) = InlineLen::from_usize(new_len) else {
            return Some(RopeStr {
                inner: Rc::new(RopeStrInner {
                    left: YSString::from_inline(*self),
                    right: str,
                }),
            });
        };

        self.data[prev_len..new_len].copy_from_slice(str.as_bytes());
        self.len = len;

        None
    }

    fn pop(&mut self) {
        self.len = InlineLen::from_usize(self.len().saturating_sub(1)).expect("unreachable");
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
        self.inner.left.len() + self.inner.right.len()
    }

    fn left_rc(&self) -> usize {
        match &self.inner.left.inner() {
            InnerString::Rc(rc) => Rc::strong_count(rc),
            InnerString::Rope(rope) => Rc::strong_count(&rope.inner),
            _ => 1,
        }
    }

    fn right_rc(&self) -> usize {
        match &self.inner.right.inner() {
            InnerString::Rc(rc) => Rc::strong_count(rc),
            InnerString::Rope(rope) => Rc::strong_count(&rope.inner),
            _ => 1,
        }
    }

    fn as_string_opt_rope(&self) -> String {
        let mut str = String::with_capacity(self.len());

        if self.left_rc() == 1 {
            str.push_str(&self.inner.left.as_str_no_rope_fix()); // if we only have one reference, we can avoid cloning
        } else {
            str.push_str(self.inner.left.as_str());
        }

        if self.right_rc() == 1 {
            str.push_str(&self.inner.right.as_str_no_rope_fix()); // if we only have one reference, we can avoid cloning
        } else {
            str.push_str(self.inner.right.as_str());
        }

        str
    }

    fn as_string(&self) -> String {
        let mut str = String::with_capacity(self.len());

        str.push_str(self.inner.left.as_str());
        str.push_str(self.inner.right.as_str());

        str
    }

    fn as_ysstring(&self) -> YSString {
        YSString::from_rope_str(self.clone())
    }

    fn push(&self, ch: char) -> Result<SmallString, String> {
        let mut str = self.as_string();

        str.push(ch);

        SmallString::from_string(str)
    }

    fn push_str(&self, str: YSString) -> Self {
        let rope = self.as_ysstring();

        Self {
            inner: Rc::new(RopeStrInner {
                left: rope,
                right: str,
            }),
        }
    }

    pub fn from_elems(left: YSString, right: YSString) -> Self {
        Self {
            inner: Rc::new(RopeStrInner { left, right }),
        }
    }
}

impl Default for YSString {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&'static str> for YSString {
    fn from(str: &'static str) -> Self {
        Self::new_static(str)
    }
}

impl From<String> for YSString {
    fn from(str: String) -> Self {
        Self::from_string(str)
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
        let str = InlineString::try_from_string(&str).map_or_else(
            || match SmallString::from_string(str) {
                Ok(str) => InnerString::Owned(str),
                Err(str) => InnerString::BoxedOwned(Box::new(str)),
            },
            InnerString::Inline,
        );

        Self {
            inner: UnsafeCell::new(str),
        }
    }

    #[must_use]
    pub fn from_ref(str: &str) -> Self {
        let str = InlineString::try_from_string(str).map_or_else(
            || match SmallString::from_string(str.to_string()) {
                Ok(str) => InnerString::Owned(str),
                Err(str) => InnerString::BoxedOwned(Box::new(str)),
            },
            InnerString::Inline,
        );

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
    pub fn from_rope(left: Self, right: Self) -> Self {
        Self {
            inner: UnsafeCell::new(InnerString::Rope(RopeStr {
                inner: Rc::new(RopeStrInner { left, right }),
            })),
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
            InnerString::BoxedOwned(boxed) => boxed.len(),
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
            InnerString::BoxedOwned(boxed) => boxed,
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

    fn as_str_no_rope_fix(&self) -> Cow<'_, str> {
        Cow::Borrowed(match self.inner() {
            InnerString::Inline(inline) => inline.as_str(),
            InnerString::Static(static_str) => static_str,
            InnerString::Owned(owned) => owned,
            InnerString::BoxedOwned(boxed) => boxed,
            InnerString::Rc(rc) => rc,
            InnerString::Rope(rope) => return Cow::Owned(rope.as_string_opt_rope()), // we don't need to fix the rope here
        })
    }

    fn as_mut_str(&mut self) -> &mut str {
        let inner = self.inner_mut();

        let owned = match inner {
            InnerString::Inline(inline) => return inline.as_mut_str(),
            InnerString::Static(static_str) => {
                SmallString::from_string((**static_str).to_string()).unwrap_or_default()
            }
            InnerString::Owned(owned) => owned.clone(),
            InnerString::BoxedOwned(boxed) => return boxed.as_mut_str(),
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

    pub fn push(&mut self, ch: char) {
        let inner = self.inner_mut();

        let new = match inner {
            InnerString::Inline(inline) => inline.push(ch),
            InnerString::Static(static_str) => {
                let mut string = String::with_capacity(static_str.len() + 1);
                string.push_str(static_str);

                string.push(ch);

                Some(SmallString::from_string(string))
            }
            InnerString::Owned(owned) => {
                owned.push(ch);
                None
            }
            InnerString::BoxedOwned(boxed) => {
                boxed.push(ch);
                None
            }
            InnerString::Rc(rc) => {
                let mut string = rc.to_string();
                string.push(ch);

                Some(SmallString::from_string(string))
            }
            InnerString::Rope(rope) => Some(rope.push(ch)),
        };

        if let Some(new) = new {
            match new {
                Ok(new) => *inner = InnerString::Owned(new),
                Err(new) => *inner = InnerString::BoxedOwned(Box::new(new)),
            }
        }
    }

    pub fn push_str(&mut self, str: impl Into<Self>) {
        let inner = self.inner_mut();
        let str = str.into();

        let new = match inner {
            InnerString::Inline(inline) => inline.push_str(str),
            InnerString::Static(static_str) => {
                let left = Self::new_static(static_str);
                Some(RopeStr::from_elems(left, str))
            }
            InnerString::Owned(owned) => {
                let rc = owned.clone().into_rc();

                let left = Self::from_rc(rc);

                Some(RopeStr::from_elems(left, str))
            }
            InnerString::BoxedOwned(boxed) => {
                let left = Self::from_string((**boxed).clone());
                Some(RopeStr::from_elems(left, str))
            }
            InnerString::Rc(rc) => {
                let left = Self::from_rc(Rc::clone(rc));
                Some(RopeStr::from_elems(left, str))
            }
            InnerString::Rope(rope) => Some(rope.push_str(str)),
        };

        if let Some(new) = new {
            *inner = InnerString::Rope(new);
        }
    }

    fn pop(&mut self) {
        match self.inner.get_mut() {
            InnerString::Inline(inline) => inline.pop(),
            InnerString::Static(static_str) => {
                if !static_str.is_empty() {
                    *static_str = &static_str[..static_str.len() - 1];
                }
            }
            InnerString::Owned(owned) => owned.pop(),
            InnerString::BoxedOwned(boxed) => {
                boxed.pop();
            }
            InnerString::Rc(rc) => {
                let mut str = rc.to_string();
                str.pop();

                let str = Self::from_string(str);

                self.inner = str.inner;
            }

            InnerString::Rope(rope) => {
                let mut str = rope.as_string();
                str.pop();

                let str = Self::from_string(str);

                self.inner = str.inner;
            }
        }
    }
}

impl Deref for YSString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl Clone for YSString {
    fn clone(&self) -> Self {
        match self.inner() {
            InnerString::Inline(ref inline) => Self::from_inline(*inline),
            InnerString::Static(static_str) => Self::new_static(static_str),
            InnerString::Owned(owned) => Self::from_rc(Rc::from(owned.as_str())), //TODO: once UniqueRc is stable, we can actually clone the string without copying it
            InnerString::BoxedOwned(boxed) => Self::from_string((**boxed).clone()), //TODO: make this non-clone, TODO: we need a second method where we DON'T move the string into a inline string
            InnerString::Rc(ref rc) => Self::from_rc(Rc::clone(rc)),
            InnerString::Rope(ref rope) => Self::from_rope_str(rope.clone()),
        }
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

impl PartialEq<str> for YSString {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
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

impl Add<Self> for YSString {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self::Output {
        self.push_str(rhs);

        self
    }
}

impl Add<&Self> for YSString {
    type Output = Self;

    fn add(mut self, rhs: &Self) -> Self::Output {
        self.push_str(rhs.clone());
        self
    }
}

impl<T: Into<Self>> AddAssign<T> for YSString {
    fn add_assign(&mut self, rhs: T) {
        self.push_str(rhs.into());
    }
}

pub trait ToYSString {
    fn to_ys_string(&self) -> YSString;
}

impl ToYSString for &'static str {
    fn to_ys_string(&self) -> YSString {
        (*self).into()
    }
}

impl ToYSString for String {
    fn to_ys_string(&self) -> YSString {
        YSString::from_ref(self)
    }
}

impl ToYSString for bool {
    fn to_ys_string(&self) -> YSString {
        if *self {
            YSString::new_static("true")
        } else {
            YSString::new_static("false")
        }
    }
}

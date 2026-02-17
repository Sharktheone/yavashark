//! `YSString` - A JavaScript-compatible string type with dual UTF-8/UTF-16 storage.
//!
//! This module provides a string type that supports WTF-16 semantics (JavaScript strings),
//! while optimizing memory usage for ASCII-only strings by storing them as UTF-8.
//!
//! # Storage Strategy
//!
//! - **ASCII-only strings** (all bytes < 128): Stored as UTF-8 for memory efficiency.
//!   In this case, `byte_len == utf16_len`, so length is O(1).
//! - **Non-ASCII strings**: Stored as UTF-16 code units for correct JavaScript semantics.
//! - **Strings with lone surrogates**: Must use UTF-16 storage (can't be represented in UTF-8).

#![allow(unused)]

mod codepoint;
mod const_string;
mod iter;
mod smallstring;
pub(crate) mod smallvec;
mod utf16;
pub(crate) mod uz;

use crate::codepoint::{decode_surrogate_pair, is_high_surrogate, is_low_surrogate, is_surrogate};
use crate::iter::{CodePoints, CodeUnits};
use crate::smallstring::SmallString;
use crate::smallvec::SmallVecLenCap;
use crate::utf16::InlineUtf16String;
use crate::uz::UZ_BYTES;
use std::borrow::Cow;
use std::cell::UnsafeCell;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{Add, AddAssign, Deref, DerefMut};
use std::rc::Rc;
use thin_vec::ThinVec;

pub use codepoint::CodePoint;
pub use const_string::ConstString;

/// A JavaScript-compatible string with dual UTF-8/UTF-16 storage.
///
/// This type provides correct JavaScript string semantics:
/// - `len()` returns the number of UTF-16 code units
/// - Indexing is by UTF-16 code unit position
/// - Supports lone/unpaired surrogates (WTF-16)
///
/// For memory efficiency, ASCII-only strings are stored as UTF-8.
pub struct YSString {
    inner: UnsafeCell<InnerString>,
}

impl Display for YSString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str_lossy())
    }
}

impl Debug for YSString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str_lossy())
    }
}

impl Hash for YSString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash by UTF-16 code units for consistency
        for unit in self.code_units() {
            unit.hash(state);
        }
    }
}

/// Internal storage for `YSString`.
///
/// Uses UTF-8 storage for ASCII-only strings (`byte_len` == `utf16_len`),
/// and UTF-16 storage for everything else.
enum InnerString {
    // UTF-8 variants (ASCII-only, byte_len == utf16_len)
    /// Inline UTF-8 string (up to 23 ASCII bytes)
    InlineUtf8(InlineString),
    /// Static UTF-8 string reference
    Static(&'static str),
    /// Heap-allocated UTF-8 string
    OwnedUtf8(SmallString),
    /// Reference-counted UTF-8 string
    RcUtf8(Rc<str>),
    /// Boxed UTF-8 string (for strings > 2^60 bytes)
    #[cold]
    #[allow(clippy::box_collection)]
    BoxedUtf8(Box<String>),

    // UTF-16 variants (non-ASCII or lone surrogates)
    /// Inline UTF-16 string (up to 11 code units)
    InlineUtf16(InlineUtf16String),
    /// Heap-allocated UTF-16 string
    OwnedUtf16(ThinVec<u16>),
    /// Reference-counted UTF-16 string
    RcUtf16(Rc<[u16]>),

    // Lazy concatenation
    /// Rope string for lazy concatenation
    Rope(RopeStr),
}

/// Inline UTF-8 string storage (up to 23 bytes).
#[repr(C, packed)]
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

/// Length enum for inline strings (0-23 bytes).
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

/// Rope string for lazy concatenation.
#[derive(Clone)]
struct RopeStr {
    inner: Rc<RopeStrInner>,
}

/// Inner data for rope strings.
pub struct RopeStrInner {
    left: YSString,
    right: YSString,
}

// =============================================================================
// InlineString implementation
// =============================================================================

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

    fn as_bytes(&self) -> &[u8] {
        &self.data[0..self.len()]
    }

    fn as_mut_str(&mut self) -> &mut str {
        let len = self.len();
        unsafe { std::str::from_utf8_unchecked_mut(&mut self.data[0..len]) }
    }

    /// Try to create an inline string from a UTF-8 string.
    /// Returns None if the string is too long or contains non-ASCII characters.
    fn try_from_ascii(s: &str) -> Option<Self> {
        if !s.is_ascii() {
            return None;
        }

        let len = InlineLen::from_usize(s.len())?;
        let mut data = [0; 23];

        if len != InlineLen::Empty {
            data[0..len as usize].copy_from_slice(s.as_bytes());
        }

        Some(Self { len, data })
    }

    /// Try to create an inline string, allowing non-ASCII UTF-8.
    /// Only used internally when we know the source is ASCII.
    fn try_from_string(s: &str) -> Option<Self> {
        let len = InlineLen::from_usize(s.len())?;
        let mut data = [0; 23];

        if len != InlineLen::Empty {
            data[0..len as usize].copy_from_slice(s.as_bytes());
        }

        Some(Self { len, data })
    }

    const fn push_ascii(&mut self, ch: u8) -> bool {
        let prev_len = self.len();
        if let Some(len) = InlineLen::from_usize(prev_len + 1) {
            self.data[prev_len] = ch;
            self.len = len;
            true
        } else {
            false
        }
    }

    #[allow(clippy::expect_used)]
    const fn pop(&mut self) {
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

// =============================================================================
// RopeStr implementation
// =============================================================================

impl RopeStr {
    fn len(&self) -> usize {
        self.inner.left.len() + self.inner.right.len()
    }

    fn is_utf8(&self) -> bool {
        self.inner.left.is_utf8_storage() && self.inner.right.is_utf8_storage()
    }

    /// Flatten to UTF-16 code units.
    fn to_utf16_vec(&self) -> ThinVec<u16> {
        let mut result = ThinVec::with_capacity(self.len());
        result.extend(self.inner.left.code_units());
        result.extend(self.inner.right.code_units());
        result
    }

    /// Flatten to String (only valid if both sides are UTF-8).
    fn to_string(&self) -> String {
        let mut result = String::with_capacity(self.len());
        if let Some(s) = self.inner.left.as_str() {
            result.push_str(s);
        } else {
            result.push_str(&self.inner.left.as_str_lossy());
        }
        if let Some(s) = self.inner.right.as_str() {
            result.push_str(s);
        } else {
            result.push_str(&self.inner.right.as_str_lossy());
        }
        result
    }

    fn as_ysstring(&self) -> YSString {
        YSString::from_rope_str(self.clone())
    }

    pub fn from_elems(left: YSString, right: YSString) -> Self {
        Self {
            inner: Rc::new(RopeStrInner { left, right }),
        }
    }
}

// =============================================================================
// YSString implementation
// =============================================================================

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
    // -------------------------------------------------------------------------
    // Constructors
    // -------------------------------------------------------------------------

    /// Creates an empty string.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(InnerString::InlineUtf8(InlineString {
                len: InlineLen::Empty,
                data: [0; 23],
            })),
        }
    }

    /// Creates a string from a static ASCII string reference.
    ///
    /// This is a const fn that assumes the string is ASCII.
    /// If the string contains non-ASCII characters, they will be stored as-is
    /// but may cause issues when the string is accessed.
    ///
    /// For non-ASCII static strings, use `new_static` instead.
    #[must_use]
    pub const fn new_static_ascii(s: &'static str) -> Self {
        Self {
            inner: UnsafeCell::new(InnerString::Static(s)),
        }
    }

    /// Creates a string from a static string reference.
    ///
    /// If the string is ASCII-only, it's stored as UTF-8.
    /// Otherwise, it's converted to UTF-16.
    #[must_use]
    pub fn new_static(s: &'static str) -> Self {
        if s.is_ascii() {
            Self {
                inner: UnsafeCell::new(InnerString::Static(s)),
            }
        } else {
            // Convert to UTF-16
            Self::from_utf16_iter(s.encode_utf16())
        }
    }

    /// Creates a string from an owned String.
    ///
    /// If the string is ASCII-only, it's stored as UTF-8.
    /// Otherwise, it's converted to UTF-16.
    #[must_use]
    pub fn from_string(s: String) -> Self {
        if s.is_ascii() {
            // Try inline first
            if let Some(inline) = InlineString::try_from_string(&s) {
                return Self {
                    inner: UnsafeCell::new(InnerString::InlineUtf8(inline)),
                };
            }

            // Fall back to heap
            match SmallString::from_string(s) {
                Ok(small) => Self {
                    inner: UnsafeCell::new(InnerString::OwnedUtf8(small)),
                },
                Err(s) => Self {
                    inner: UnsafeCell::new(InnerString::BoxedUtf8(Box::new(s))),
                },
            }
        } else {
            // Convert to UTF-16
            Self::from_utf16_iter(s.encode_utf16())
        }
    }

    /// Creates a string from a string reference.
    #[must_use]
    pub fn from_ref(s: &str) -> Self {
        if s.is_ascii() {
            // Try inline first
            if let Some(inline) = InlineString::try_from_string(s) {
                return Self {
                    inner: UnsafeCell::new(InnerString::InlineUtf8(inline)),
                };
            }

            // Fall back to heap
            match SmallString::from_string(s.to_string()) {
                Ok(small) => Self {
                    inner: UnsafeCell::new(InnerString::OwnedUtf8(small)),
                },
                Err(s) => Self {
                    inner: UnsafeCell::new(InnerString::BoxedUtf8(Box::new(s))),
                },
            }
        } else {
            // Convert to UTF-16
            Self::from_utf16_iter(s.encode_utf16())
        }
    }

    /// Creates a string from a reference-counted string.
    #[must_use]
    pub fn from_rc(rc: Rc<str>) -> Self {
        if rc.is_ascii() {
            Self {
                inner: UnsafeCell::new(InnerString::RcUtf8(rc)),
            }
        } else {
            Self::from_utf16_iter(rc.encode_utf16())
        }
    }

    /// Creates a string from UTF-16 code units.
    ///
    /// This can include lone surrogates (WTF-16).
    #[must_use]
    pub fn from_utf16(units: &[u16]) -> Self {
        // Check if it's ASCII-representable
        if units.iter().all(|&u| u < 128) {
            let ascii: String = units.iter().map(|&u| u as u8 as char).collect();
            Self::from_string(ascii)
        } else {
            // Try inline first
            if let Some(inline) = InlineUtf16String::try_from_slice(units) {
                return Self {
                    inner: UnsafeCell::new(InnerString::InlineUtf16(inline)),
                };
            }

            // Fall back to heap
            Self {
                inner: UnsafeCell::new(InnerString::OwnedUtf16(units.into())),
            }
        }
    }

    /// Creates a string from a UTF-16 iterator.
    fn from_utf16_iter(iter: impl Iterator<Item = u16>) -> Self {
        let units: ThinVec<u16> = iter.collect();

        // Check if it's ASCII-representable
        if units.iter().all(|&u| u < 128) {
            let ascii: String = units.iter().map(|&u| u as u8 as char).collect();
            Self::from_string(ascii)
        } else {
            // Try inline first
            if let Some(inline) = InlineUtf16String::try_from_slice(&units) {
                return Self {
                    inner: UnsafeCell::new(InnerString::InlineUtf16(inline)),
                };
            }

            Self {
                inner: UnsafeCell::new(InnerString::OwnedUtf16(units)),
            }
        }
    }

    /// Creates a string from a single UTF-16 code unit.
    #[must_use]
    pub const fn from_code_unit(unit: u16) -> Self {
        if unit < 128 {
            // ASCII
            let mut data = [0u8; 23];
            data[0] = unit as u8;
            Self {
                inner: UnsafeCell::new(InnerString::InlineUtf8(InlineString {
                    len: InlineLen::Len1,
                    data,
                })),
            }
        } else {
            Self {
                inner: UnsafeCell::new(InnerString::InlineUtf16(
                    InlineUtf16String::from_code_unit(unit),
                )),
            }
        }
    }

    /// Creates a string from a `CodePoint`.
    #[must_use]
    pub const fn from_code_point(cp: CodePoint) -> Self {
        match cp {
            CodePoint::Unicode(c) => {
                if c.is_ascii() {
                    let mut data = [0u8; 23];
                    data[0] = c as u8;
                    Self {
                        inner: UnsafeCell::new(InnerString::InlineUtf8(InlineString {
                            len: InlineLen::Len1,
                            data,
                        })),
                    }
                } else {
                    Self {
                        inner: UnsafeCell::new(InnerString::InlineUtf16(
                            InlineUtf16String::from_char(c),
                        )),
                    }
                }
            }
            CodePoint::UnpairedSurrogate(s) => Self {
                inner: UnsafeCell::new(InnerString::InlineUtf16(
                    InlineUtf16String::from_code_unit(s),
                )),
            },
        }
    }

    /// Creates a rope string from two strings.
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
            inner: UnsafeCell::new(InnerString::InlineUtf8(inline)),
        }
    }

    #[must_use]
    const fn from_rope_str(rope: RopeStr) -> Self {
        Self {
            inner: UnsafeCell::new(InnerString::Rope(rope)),
        }
    }

    // -------------------------------------------------------------------------
    // Internal accessors
    // -------------------------------------------------------------------------

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

    /// Returns true if the string is stored as UTF-8.
    #[must_use]
    pub fn is_utf8_storage(&self) -> bool {
        match self.inner() {
            InnerString::InlineUtf8(_)
            | InnerString::Static(_)
            | InnerString::OwnedUtf8(_)
            | InnerString::RcUtf8(_)
            | InnerString::BoxedUtf8(_) => true,
            InnerString::InlineUtf16(_) | InnerString::OwnedUtf16(_) | InnerString::RcUtf16(_) => {
                false
            }
            InnerString::Rope(rope) => rope.is_utf8(),
        }
    }

    // -------------------------------------------------------------------------
    // Length and emptiness
    // -------------------------------------------------------------------------

    /// Returns the number of UTF-16 code units in the string.
    ///
    /// This is O(1) for all storage variants because:
    /// - UTF-8 storage is only used for ASCII strings where `byte_len` == `utf16_len`
    /// - UTF-16 storage directly stores code units
    #[must_use]
    pub fn len(&self) -> usize {
        match self.inner() {
            // UTF-8 variants (ASCII-only, byte_len == utf16_len)
            InnerString::InlineUtf8(inline) => inline.len(),
            InnerString::Static(s) => s.len(),
            InnerString::OwnedUtf8(s) => s.len(),
            InnerString::RcUtf8(s) => s.len(),
            InnerString::BoxedUtf8(s) => s.len(),

            // UTF-16 variants
            InnerString::InlineUtf16(inline) => inline.len(),
            InnerString::OwnedUtf16(v) => v.len(),
            InnerString::RcUtf16(v) => v.len(),

            // Rope
            InnerString::Rope(rope) => rope.len(),
        }
    }

    /// Returns true if the string is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    // -------------------------------------------------------------------------
    // String access
    // -------------------------------------------------------------------------

    /// Returns the string as a `&str` if it's stored as UTF-8.
    ///
    /// Returns `None` if the string is stored as UTF-16 (non-ASCII or contains surrogates).
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self.inner() {
            InnerString::InlineUtf8(inline) => Some(inline.as_str()),
            InnerString::Static(s) => Some(s),
            InnerString::OwnedUtf8(s) => Some(s),
            InnerString::RcUtf8(s) => Some(s),
            InnerString::BoxedUtf8(s) => Some(s),

            // UTF-16 variants can't return &str directly
            InnerString::InlineUtf16(_) | InnerString::OwnedUtf16(_) | InnerString::RcUtf16(_) => {
                None
            }

            // Rope needs to be flattened first
            InnerString::Rope(rope) => {
                if rope.is_utf8() {
                    // Flatten to UTF-8
                    let s = rope.to_string();
                    let inner = unsafe { self.inner_mut_ref() };

                    match SmallString::from_string(s) {
                        Ok(small) => *inner = InnerString::OwnedUtf8(small),
                        Err(s) => *inner = InnerString::BoxedUtf8(Box::new(s)),
                    }

                    match inner {
                        InnerString::OwnedUtf8(s) => Some(s.as_str()),
                        InnerString::BoxedUtf8(s) => Some(s.as_str()),
                        _ => None,
                    }
                } else {
                    None
                }
            }
        }
    }

    /// Returns the string as a `Cow<str>`, replacing invalid UTF-8 with U+FFFD.
    ///
    /// For UTF-8 stored strings, this is a borrowed reference.
    /// For UTF-16 stored strings, this performs a lossy conversion.
    #[must_use]
    pub fn as_str_lossy(&self) -> Cow<'_, str> {
        match self.inner() {
            InnerString::InlineUtf8(inline) => Cow::Borrowed(inline.as_str()),
            InnerString::Static(s) => Cow::Borrowed(s),
            InnerString::OwnedUtf8(s) => Cow::Borrowed(s),
            InnerString::RcUtf8(s) => Cow::Borrowed(s),
            InnerString::BoxedUtf8(s) => Cow::Borrowed(s),

            InnerString::InlineUtf16(inline) => Cow::Owned(inline.to_string_lossy()),
            InnerString::OwnedUtf16(v) => Cow::Owned(String::from_utf16_lossy(v)),
            InnerString::RcUtf16(v) => Cow::Owned(String::from_utf16_lossy(v)),

            InnerString::Rope(rope) => Cow::Owned(rope.to_string()),
        }
    }

    /// Returns the UTF-16 code units as a slice, if stored as UTF-16.
    #[must_use]
    pub fn as_utf16(&self) -> Option<&[u16]> {
        match self.inner() {
            InnerString::InlineUtf16(inline) => Some(inline.as_slice()),
            InnerString::OwnedUtf16(v) => Some(v),
            InnerString::RcUtf16(v) => Some(v),
            _ => None,
        }
    }

    // -------------------------------------------------------------------------
    // Code unit and code point access
    // -------------------------------------------------------------------------

    /// Returns the UTF-16 code unit at the given index.
    ///
    /// Returns `None` if the index is out of bounds.
    #[must_use]
    pub fn code_unit_at(&self, index: usize) -> Option<u16> {
        if index >= self.len() {
            return None;
        }

        match self.inner() {
            // UTF-8 variants (ASCII-only)
            InnerString::InlineUtf8(inline) => inline.as_bytes().get(index).map(|&b| u16::from(b)),
            InnerString::Static(s) => s.as_bytes().get(index).map(|&b| u16::from(b)),
            InnerString::OwnedUtf8(s) => s.as_bytes().get(index).map(|&b| u16::from(b)),
            InnerString::RcUtf8(s) => s.as_bytes().get(index).map(|&b| u16::from(b)),
            InnerString::BoxedUtf8(s) => s.as_bytes().get(index).map(|&b| u16::from(b)),

            // UTF-16 variants
            InnerString::InlineUtf16(inline) => inline.get(index),
            InnerString::OwnedUtf16(v) => v.get(index).copied(),
            InnerString::RcUtf16(v) => v.get(index).copied(),

            // Rope - flatten first
            InnerString::Rope(rope) => {
                // TODO: Could optimize to avoid full flattening
                let units = rope.to_utf16_vec();
                units.get(index).copied()
            }
        }
    }

    /// Returns the code point at the given UTF-16 index.
    ///
    /// If the index points to a high surrogate followed by a low surrogate,
    /// returns the combined code point. Otherwise returns the code unit as a code point.
    #[must_use]
    pub fn code_point_at(&self, index: usize) -> Option<CodePoint> {
        let unit = self.code_unit_at(index)?;

        if is_high_surrogate(unit) {
            // Check if next unit is a low surrogate
            if let Some(next) = self.code_unit_at(index + 1) {
                if is_low_surrogate(next) {
                    return Some(CodePoint::Unicode(decode_surrogate_pair(unit, next)));
                }
            }
            // Unpaired high surrogate
            Some(CodePoint::UnpairedSurrogate(unit))
        } else if is_low_surrogate(unit) {
            // Unpaired low surrogate
            Some(CodePoint::UnpairedSurrogate(unit))
        } else {
            // BMP character
            Some(CodePoint::Unicode(unsafe {
                char::from_u32_unchecked(u32::from(unit))
            }))
        }
    }

    /// Returns an iterator over UTF-16 code units.
    pub fn code_units(&self) -> CodeUnits<'_> {
        match self.inner() {
            InnerString::InlineUtf8(inline) => CodeUnits::Utf8(inline.as_str().bytes()),
            InnerString::Static(s) => CodeUnits::Utf8(s.bytes()),
            InnerString::OwnedUtf8(s) => CodeUnits::Utf8(s.bytes()),
            InnerString::RcUtf8(s) => CodeUnits::Utf8(s.bytes()),
            InnerString::BoxedUtf8(s) => CodeUnits::Utf8(s.bytes()),

            InnerString::InlineUtf16(inline) => CodeUnits::Utf16(inline.as_slice().iter().copied()),
            InnerString::OwnedUtf16(v) => CodeUnits::Utf16(v.iter().copied()),
            InnerString::RcUtf16(v) => CodeUnits::Utf16(v.iter().copied()),

            InnerString::Rope(rope) => {
                // Need to flatten rope for iteration
                let units = rope.to_utf16_vec();
                // This is inefficient, but ropes should be flattened before heavy iteration
                let inner = unsafe { self.inner_mut_ref() };
                *inner = InnerString::OwnedUtf16(units);

                if let InnerString::OwnedUtf16(v) = inner {
                    CodeUnits::Utf16(v.iter().copied())
                } else {
                    CodeUnits::Utf16([].iter().copied()) // Unreachable
                }
            }
        }
    }

    /// Returns an iterator over code points.
    ///
    /// Surrogate pairs are combined into single code points.
    /// Unpaired surrogates are yielded as `CodePoint::UnpairedSurrogate`.
    pub fn code_points(&self) -> CodePoints<'_> {
        CodePoints::new(self.code_units())
    }

    // -------------------------------------------------------------------------
    // Well-formedness
    // -------------------------------------------------------------------------

    /// Returns true if the string is well-formed (contains no unpaired surrogates).
    ///
    /// UTF-8 stored strings are always well-formed.
    #[must_use]
    pub fn is_well_formed(&self) -> bool {
        match self.inner() {
            // UTF-8 storage can't have surrogates
            InnerString::InlineUtf8(_)
            | InnerString::Static(_)
            | InnerString::OwnedUtf8(_)
            | InnerString::RcUtf8(_)
            | InnerString::BoxedUtf8(_) => true,

            // Check UTF-16 for unpaired surrogates
            InnerString::InlineUtf16(inline) => is_utf16_well_formed(inline.as_slice()),
            InnerString::OwnedUtf16(v) => is_utf16_well_formed(v),
            InnerString::RcUtf16(v) => is_utf16_well_formed(v),

            // Rope: check both sides
            InnerString::Rope(rope) => {
                rope.inner.left.is_well_formed() && rope.inner.right.is_well_formed()
            }
        }
    }

    /// Converts the string to a well-formed string by replacing unpaired surrogates with U+FFFD.
    #[must_use]
    pub fn to_well_formed(&self) -> Self {
        if self.is_well_formed() {
            return self.clone();
        }

        // Convert to well-formed UTF-16
        let units: ThinVec<u16> = self
            .code_points()
            .flat_map(|cp| match cp {
                CodePoint::Unicode(c) => {
                    let mut buf = [0u16; 2];
                    let encoded = c.encode_utf16(&mut buf);
                    encoded.to_vec()
                }
                CodePoint::UnpairedSurrogate(_) => vec![0xFFFD], // Replacement character
            })
            .collect();

        Self::from_utf16(&units)
    }

    // -------------------------------------------------------------------------
    // Mutation
    // -------------------------------------------------------------------------

    /// Pushes a character to the end of the string.
    pub fn push(&mut self, ch: char) {
        if ch.is_ascii() && self.is_utf8_storage() {
            // Can stay as UTF-8
            let inner = self.inner_mut();
            match inner {
                InnerString::InlineUtf8(inline) => {
                    if inline.push_ascii(ch as u8) {
                        return;
                    }
                    // Need to upgrade to heap
                    let mut s = inline.as_str().to_string();
                    s.push(ch);
                    match SmallString::from_string(s) {
                        Ok(small) => *inner = InnerString::OwnedUtf8(small),
                        Err(s) => *inner = InnerString::BoxedUtf8(Box::new(s)),
                    }
                }
                InnerString::Static(s) => {
                    let mut string = (**s).to_string();
                    string.push(ch);
                    match SmallString::from_string(string) {
                        Ok(small) => *inner = InnerString::OwnedUtf8(small),
                        Err(s) => *inner = InnerString::BoxedUtf8(Box::new(s)),
                    }
                }
                InnerString::OwnedUtf8(s) => {
                    s.push(ch);
                }
                InnerString::RcUtf8(rc) => {
                    let mut s = rc.to_string();
                    s.push(ch);
                    match SmallString::from_string(s) {
                        Ok(small) => *inner = InnerString::OwnedUtf8(small),
                        Err(s) => *inner = InnerString::BoxedUtf8(Box::new(s)),
                    }
                }
                InnerString::BoxedUtf8(s) => {
                    s.push(ch);
                }
                _ => {} // unreachable
            }
        } else {
            // Need to use UTF-16 storage
            self.push_utf16(ch);
        }
    }

    /// Push a character using UTF-16 storage.
    fn push_utf16(&mut self, ch: char) {
        let inner = self.inner_mut();

        // Convert current contents to UTF-16 if needed
        let mut units: ThinVec<u16> = match inner {
            InnerString::InlineUtf8(inline) => inline.as_str().encode_utf16().collect(),
            InnerString::Static(s) => s.encode_utf16().collect(),
            InnerString::OwnedUtf8(s) => s.encode_utf16().collect(),
            InnerString::RcUtf8(s) => s.encode_utf16().collect(),
            InnerString::BoxedUtf8(s) => s.encode_utf16().collect(),
            InnerString::InlineUtf16(inline) => inline.as_slice().into(),
            InnerString::OwnedUtf16(v) => std::mem::take(v),
            InnerString::RcUtf16(v) => v.to_vec().into(),
            InnerString::Rope(rope) => rope.to_utf16_vec(),
        };

        // Encode the character
        let mut buf = [0u16; 2];
        let encoded = ch.encode_utf16(&mut buf);
        units.extend_from_slice(encoded);

        *inner = InnerString::OwnedUtf16(units);
    }

    /// Pushes a string to the end of this string.
    pub fn push_str(&mut self, s: impl Into<Self>) {
        let s = s.into();
        if s.is_empty() {
            return;
        }

        let inner = self.inner_mut();

        // Create a rope for lazy concatenation
        let left = std::mem::replace(
            inner,
            InnerString::InlineUtf8(InlineString {
                len: InlineLen::Empty,
                data: [0; 23],
            }),
        );

        let left = Self {
            inner: UnsafeCell::new(left),
        };

        *inner = InnerString::Rope(RopeStr::from_elems(left, s));
    }

    // -------------------------------------------------------------------------
    // Convenience methods for migration compatibility
    // -------------------------------------------------------------------------

    /// Returns true if the string contains the given substring.
    #[must_use]
    pub fn contains(&self, pattern: &str) -> bool {
        // For ASCII-only strings, use direct str::contains
        if let Some(s) = self.as_str() {
            return s.contains(pattern);
        }

        // For UTF-16, convert to string and check
        self.as_str_lossy().contains(pattern)
    }

    /// Returns true if the string starts with the given prefix.
    #[must_use]
    pub fn starts_with(&self, prefix: &str) -> bool {
        if let Some(s) = self.as_str() {
            return s.starts_with(prefix);
        }

        self.as_str_lossy().starts_with(prefix)
    }

    /// Returns true if the string ends with the given suffix.
    #[must_use]
    pub fn ends_with(&self, suffix: &str) -> bool {
        if let Some(s) = self.as_str() {
            return s.ends_with(suffix);
        }

        self.as_str_lossy().ends_with(suffix)
    }

    /// Returns a string with leading and trailing whitespace removed.
    #[must_use]
    pub fn trim(&self) -> Cow<'_, str> {
        if let Some(s) = self.as_str() {
            return Cow::Borrowed(s.trim());
        }

        Cow::Owned(self.as_str_lossy().trim().to_string())
    }

    /// Returns an iterator over characters.
    /// For UTF-16 storage, this performs lossy conversion.
    pub fn chars(&self) -> std::str::Chars<'_> {
        // This is a bit awkward for UTF-16, but we need to return Chars
        // For UTF-8 storage, this works directly
        if let Some(s) = self.as_str() {
            return s.chars();
        }

        // For UTF-16, we need to flatten first
        // This is a limitation - we can't easily return Chars for UTF-16
        // Flatten the rope/UTF-16 to UTF-8 first
        let inner = unsafe { self.inner_mut_ref() };
        let s = self.as_str_lossy().into_owned();
        match SmallString::from_string(s) {
            Ok(small) => *inner = InnerString::OwnedUtf8(small),
            Err(s) => *inner = InnerString::BoxedUtf8(Box::new(s)),
        }

        match inner {
            InnerString::OwnedUtf8(s) => s.chars(),
            InnerString::BoxedUtf8(s) => s.chars(),
            _ => "".chars(), // Unreachable
        }
    }

    /// Returns the bytes of the string (only valid for UTF-8 storage).
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self.inner() {
            InnerString::InlineUtf8(inline) => Some(inline.as_bytes()),
            InnerString::Static(s) => Some(s.as_bytes()),
            InnerString::OwnedUtf8(s) => Some(s.as_bytes()),
            InnerString::RcUtf8(s) => Some(s.as_bytes()),
            InnerString::BoxedUtf8(s) => Some(s.as_bytes()),
            _ => None,
        }
    }

    /// Get a substring by byte range (UTF-8 only).
    pub fn get<R: std::ops::RangeBounds<usize>>(&self, range: R) -> Option<&str> {
        let start = match range.start_bound() {
            std::ops::Bound::Included(&n) => n,
            std::ops::Bound::Excluded(&n) => n + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            std::ops::Bound::Included(&n) => n + 1,
            std::ops::Bound::Excluded(&n) => n,
            std::ops::Bound::Unbounded => self.len(),
        };
        self.as_str()?.get(start..end)
    }

    /// Returns true if the given index is a char boundary (UTF-8 only).
    #[must_use]
    pub fn is_char_boundary(&self, index: usize) -> bool {
        if let Some(s) = self.as_str() {
            return s.is_char_boundary(index);
        }
        // For UTF-16 storage, every index is a valid code unit boundary
        index <= self.len()
    }

    /// Parse the string into a type that implements `FromStr`.
    pub fn parse<F: std::str::FromStr>(&self) -> Result<F, F::Err> {
        self.as_str_lossy().parse()
    }

    /// Pushes a UTF-16 code unit to the end of the string.
    pub fn push_code_unit(&mut self, unit: u16) {
        if unit < 128 && self.is_utf8_storage() {
            self.push(unit as u8 as char);
        } else {
            let inner = self.inner_mut();

            // Convert current contents to UTF-16 if needed
            let mut units: ThinVec<u16> = match inner {
                InnerString::InlineUtf8(inline) => inline.as_str().encode_utf16().collect(),
                InnerString::Static(s) => s.encode_utf16().collect(),
                InnerString::OwnedUtf8(s) => s.encode_utf16().collect(),
                InnerString::RcUtf8(s) => s.encode_utf16().collect(),
                InnerString::BoxedUtf8(s) => s.encode_utf16().collect(),
                InnerString::InlineUtf16(inline) => inline.as_slice().into(),
                InnerString::OwnedUtf16(v) => std::mem::take(v),
                InnerString::RcUtf16(v) => v.to_vec().into(),
                InnerString::Rope(rope) => rope.to_utf16_vec(),
            };

            units.push(unit);
            *inner = InnerString::OwnedUtf16(units);
        }
    }
}

// =============================================================================
// Helper functions
// =============================================================================

/// Check if a UTF-16 slice is well-formed (no unpaired surrogates).
fn is_utf16_well_formed(units: &[u16]) -> bool {
    let mut i = 0;
    while i < units.len() {
        let unit = units[i];
        if is_high_surrogate(unit) {
            // Must be followed by a low surrogate
            if i + 1 >= units.len() || !is_low_surrogate(units[i + 1]) {
                return false;
            }
            i += 2;
        } else if is_low_surrogate(unit) {
            // Unpaired low surrogate
            return false;
        } else {
            i += 1;
        }
    }
    true
}

// =============================================================================
// Trait implementations
// =============================================================================

impl Clone for YSString {
    fn clone(&self) -> Self {
        match self.inner() {
            InnerString::InlineUtf8(inline) => Self::from_inline(*inline),
            InnerString::Static(s) => Self {
                inner: UnsafeCell::new(InnerString::Static(s)),
            },
            InnerString::OwnedUtf8(s) => Self {
                inner: UnsafeCell::new(InnerString::RcUtf8(Rc::from(s.as_str()))),
            },
            InnerString::RcUtf8(rc) => Self {
                inner: UnsafeCell::new(InnerString::RcUtf8(Rc::clone(rc))),
            },
            InnerString::BoxedUtf8(s) => Self::from_string((**s).clone()),

            InnerString::InlineUtf16(inline) => Self {
                inner: UnsafeCell::new(InnerString::InlineUtf16(*inline)),
            },
            InnerString::OwnedUtf16(v) => Self {
                inner: UnsafeCell::new(InnerString::RcUtf16(Rc::from(v.as_slice()))),
            },
            InnerString::RcUtf16(rc) => Self {
                inner: UnsafeCell::new(InnerString::RcUtf16(Rc::clone(rc))),
            },

            InnerString::Rope(rope) => Self::from_rope_str(rope.clone()),
        }
    }
}

impl PartialEq for YSString {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        // Compare by code units
        self.code_units().eq(other.code_units())
    }
}

impl PartialEq<str> for YSString {
    fn eq(&self, other: &str) -> bool {
        // Compare by code units
        self.code_units().eq(other.encode_utf16())
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
        // Compare by code units (JavaScript string comparison)
        self.code_units().cmp(other.code_units())
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

// =============================================================================
// ToYSString trait
// =============================================================================

/// Trait for converting types to `YSString`.
pub trait ToYSString {
    fn to_ys_string(&self) -> YSString;
}

impl ToYSString for &'static str {
    fn to_ys_string(&self) -> YSString {
        YSString::new_static(self)
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

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_len() {
        let s = YSString::from("hello");
        assert_eq!(s.len(), 5);
        assert!(s.is_utf8_storage());
    }

    #[test]
    fn test_emoji_len() {
        let s = YSString::from_ref("😀");
        assert_eq!(s.len(), 2); // Surrogate pair = 2 code units
        assert!(!s.is_utf8_storage());
    }

    #[test]
    fn test_mixed_len() {
        let s = YSString::from_ref("hi😀");
        assert_eq!(s.len(), 4); // 2 ASCII + 2 surrogates
    }

    #[test]
    fn test_lone_surrogate() {
        let s = YSString::from_utf16(&[0xD800]);
        assert_eq!(s.len(), 1);
        assert!(!s.is_well_formed());
        assert!(s.as_str().is_none());
    }

    #[test]
    fn test_code_unit_at() {
        let s = YSString::from_ref("A😀B");
        assert_eq!(s.code_unit_at(0), Some(0x0041)); // 'A'
        assert_eq!(s.code_unit_at(1), Some(0xD83D)); // High surrogate
        assert_eq!(s.code_unit_at(2), Some(0xDE00)); // Low surrogate
        assert_eq!(s.code_unit_at(3), Some(0x0042)); // 'B'
        assert_eq!(s.code_unit_at(4), None);
    }

    #[test]
    fn test_code_point_at() {
        let s = YSString::from_ref("A😀B");
        assert_eq!(s.code_point_at(0), Some(CodePoint::Unicode('A')));
        assert_eq!(s.code_point_at(1), Some(CodePoint::Unicode('😀')));
        assert_eq!(s.code_point_at(3), Some(CodePoint::Unicode('B')));
    }

    #[test]
    fn test_code_point_at_unpaired() {
        let s = YSString::from_utf16(&[0x0041, 0xD800, 0x0042]);
        assert_eq!(s.code_point_at(0), Some(CodePoint::Unicode('A')));
        assert_eq!(
            s.code_point_at(1),
            Some(CodePoint::UnpairedSurrogate(0xD800))
        );
        assert_eq!(s.code_point_at(2), Some(CodePoint::Unicode('B')));
    }

    #[test]
    fn test_is_well_formed() {
        assert!(YSString::from("hello").is_well_formed());
        assert!(YSString::from_ref("😀").is_well_formed());
        assert!(!YSString::from_utf16(&[0xD800]).is_well_formed());
        assert!(!YSString::from_utf16(&[0xDC00]).is_well_formed());
    }

    #[test]
    fn test_to_well_formed() {
        let s = YSString::from_utf16(&[0x0041, 0xD800, 0x0042]);
        let well = s.to_well_formed();
        assert!(well.is_well_formed());
        assert_eq!(well.as_str_lossy(), "A\u{FFFD}B");
    }

    #[test]
    fn test_push_ascii() {
        let mut s = YSString::from("hel");
        s.push('l');
        s.push('o');
        assert_eq!(s.len(), 5);
        assert!(s.is_utf8_storage());
        assert_eq!(s.as_str(), Some("hello"));
    }

    #[test]
    fn test_push_non_ascii() {
        let mut s = YSString::from("hi");
        s.push('😀');
        assert_eq!(s.len(), 4);
        assert!(!s.is_utf8_storage());
    }

    #[test]
    fn test_concat() {
        let a = YSString::from("hello");
        let b = YSString::from(" world");
        let c = a + b;
        assert_eq!(c.len(), 11);
        assert_eq!(c.as_str_lossy(), "hello world");
    }

    #[test]
    fn test_equality() {
        let a = YSString::from("hello");
        let b = YSString::from_ref("hello");
        assert_eq!(a, b);

        let c = YSString::from_ref("😀");
        let d = YSString::from_utf16(&[0xD83D, 0xDE00]);
        assert_eq!(c, d);
    }
}

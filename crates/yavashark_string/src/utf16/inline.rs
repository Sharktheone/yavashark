use std::fmt::{self, Debug};

/// Maximum number of UTF-16 code units that can be stored inline.
/// With 1 byte for length and 22 bytes for data, we can fit 11 u16 values.
pub const INLINE_UTF16_CAPACITY: usize = 11;

/// An inline UTF-16 string that stores up to 11 code units without heap allocation.
///
/// Memory layout (23 bytes total):
/// - 1 byte: length (0-11)
/// - 22 bytes: data (11 × u16)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct InlineUtf16String {
    len: u8,
    data: [u16; INLINE_UTF16_CAPACITY],
}

impl InlineUtf16String {
    /// Creates an empty inline UTF-16 string.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            len: 0,
            data: [0; INLINE_UTF16_CAPACITY],
        }
    }

    /// Creates an inline UTF-16 string from a slice of code units.
    ///
    /// Returns `None` if the slice is too long to fit inline.
    #[inline]
    #[must_use]
    pub fn try_from_slice(units: &[u16]) -> Option<Self> {
        if units.len() > INLINE_UTF16_CAPACITY {
            return None;
        }

        let mut data = [0u16; INLINE_UTF16_CAPACITY];
        data[..units.len()].copy_from_slice(units);

        Some(Self {
            len: units.len() as u8,
            data,
        })
    }

    /// Creates an inline UTF-16 string from a single code unit.
    #[inline]
    #[must_use]
    pub const fn from_code_unit(unit: u16) -> Self {
        let mut data = [0u16; INLINE_UTF16_CAPACITY];
        data[0] = unit;
        Self { len: 1, data }
    }

    /// Creates an inline UTF-16 string from a char.
    ///
    /// Returns `None` if the char requires a surrogate pair and would exceed capacity.
    #[inline]
    #[must_use]
    pub const fn from_char(c: char) -> Self {
        let mut data = [0u16; INLINE_UTF16_CAPACITY];
        let encoded = c.encode_utf16(&mut data);
        Self {
            len: encoded.len() as u8,
            data,
        }
    }

    /// Returns the number of UTF-16 code units in the string.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len as usize
    }

    /// Returns `true` if the string is empty.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the code units as a slice.
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[u16] {
        &self.data[..self.len as usize]
    }

    /// Returns the code units as a mutable slice.
    #[inline]
    #[must_use]
    pub fn as_mut_slice(&mut self) -> &mut [u16] {
        let len = self.len as usize;
        &mut self.data[..len]
    }

    /// Gets the code unit at the given index.
    #[inline]
    #[must_use]
    pub const fn get(&self, index: usize) -> Option<u16> {
        if index < self.len as usize {
            Some(self.data[index])
        } else {
            None
        }
    }

    /// Pushes a code unit to the end of the string.
    ///
    /// Returns `false` if the string is full.
    #[inline]
    pub const fn push(&mut self, unit: u16) -> bool {
        if (self.len as usize) < INLINE_UTF16_CAPACITY {
            self.data[self.len as usize] = unit;
            self.len += 1;
            true
        } else {
            false
        }
    }

    /// Pushes a char to the end of the string.
    ///
    /// Returns the number of code units added, or 0 if there wasn't enough space.
    #[inline]
    pub fn push_char(&mut self, c: char) -> usize {
        let mut buf = [0u16; 2];
        let encoded = c.encode_utf16(&mut buf);
        let needed = encoded.len();

        if self.len as usize + needed <= INLINE_UTF16_CAPACITY {
            for &unit in encoded.iter() {
                self.data[self.len as usize] = unit;
                self.len += 1;
            }
            needed
        } else {
            0
        }
    }

    /// Returns the remaining capacity.
    #[inline]
    #[must_use]
    pub const fn remaining_capacity(&self) -> usize {
        INLINE_UTF16_CAPACITY - self.len as usize
    }

    /// Checks if all code units are ASCII (< 128).
    #[inline]
    #[must_use]
    pub fn is_ascii(&self) -> bool {
        self.as_slice().iter().all(|&u| u < 128)
    }

    /// Converts to a Rust String, replacing unpaired surrogates with U+FFFD.
    #[must_use]
    pub fn to_string_lossy(&self) -> String {
        String::from_utf16_lossy(self.as_slice())
    }

    /// Attempts to convert to a Rust String.
    ///
    /// Returns `None` if the string contains unpaired surrogates.
    #[must_use]
    pub fn to_string(&self) -> Option<String> {
        String::from_utf16(self.as_slice()).ok()
    }
}

impl Default for InlineUtf16String {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for InlineUtf16String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InlineUtf16String({:?})", self.to_string_lossy())
    }
}

impl PartialEq for InlineUtf16String {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl Eq for InlineUtf16String {}

impl PartialEq<[u16]> for InlineUtf16String {
    fn eq(&self, other: &[u16]) -> bool {
        self.as_slice() == other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let s = InlineUtf16String::new();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn test_from_code_unit() {
        let s = InlineUtf16String::from_code_unit(0x0041);
        assert_eq!(s.len(), 1);
        assert_eq!(s.get(0), Some(0x0041));
    }

    #[test]
    fn test_from_char_bmp() {
        let s = InlineUtf16String::from_char('A');
        assert_eq!(s.len(), 1);
        assert_eq!(s.as_slice(), &[0x0041]);
    }

    #[test]
    fn test_from_char_supplementary() {
        let s = InlineUtf16String::from_char('😀');
        assert_eq!(s.len(), 2);
        assert_eq!(s.as_slice(), &[0xD83D, 0xDE00]);
    }

    #[test]
    fn test_try_from_slice() {
        let units = [0x0048, 0x0065, 0x006C, 0x006C, 0x006F]; // "Hello"
        let s = InlineUtf16String::try_from_slice(&units).unwrap();
        assert_eq!(s.len(), 5);
        assert_eq!(s.as_slice(), &units);
    }

    #[test]
    fn test_try_from_slice_too_long() {
        let units = [0u16; 12];
        assert!(InlineUtf16String::try_from_slice(&units).is_none());
    }

    #[test]
    fn test_push() {
        let mut s = InlineUtf16String::new();
        assert!(s.push(0x0041));
        assert!(s.push(0x0042));
        assert_eq!(s.len(), 2);
        assert_eq!(s.as_slice(), &[0x0041, 0x0042]);
    }

    #[test]
    fn test_push_full() {
        let mut s = InlineUtf16String::try_from_slice(&[0u16; 11]).unwrap();
        assert!(!s.push(0x0041));
    }

    #[test]
    fn test_is_ascii() {
        let ascii = InlineUtf16String::try_from_slice(&[0x0041, 0x0042, 0x007F]).unwrap();
        assert!(ascii.is_ascii());

        let non_ascii = InlineUtf16String::try_from_slice(&[0x0041, 0x0100]).unwrap();
        assert!(!non_ascii.is_ascii());
    }

    #[test]
    fn test_to_string_lossy() {
        // Normal string
        let s = InlineUtf16String::try_from_slice(&[0x0048, 0x0069]).unwrap(); // "Hi"
        assert_eq!(s.to_string_lossy(), "Hi");

        // String with unpaired surrogate
        let s = InlineUtf16String::try_from_slice(&[0x0041, 0xD800, 0x0042]).unwrap();
        assert_eq!(s.to_string_lossy(), "A\u{FFFD}B");
    }

    #[test]
    fn test_to_string() {
        // Valid string
        let s = InlineUtf16String::try_from_slice(&[0x0048, 0x0069]).unwrap();
        assert_eq!(s.to_string(), Some("Hi".to_string()));

        // String with emoji (valid surrogate pair)
        let s = InlineUtf16String::try_from_slice(&[0xD83D, 0xDE00]).unwrap();
        assert_eq!(s.to_string(), Some("😀".to_string()));

        // String with unpaired surrogate
        let s = InlineUtf16String::try_from_slice(&[0x0041, 0xD800]).unwrap();
        assert!(s.to_string().is_none());
    }
}

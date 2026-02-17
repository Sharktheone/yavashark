use std::fmt::{self, Display, Write};

/// Represents a Unicode codepoint within a JavaScript string.
///
/// This can be either a valid Unicode scalar value (like Rust's `char`),
/// or an unpaired surrogate (which is valid in JavaScript/WTF-16 but not in UTF-8/UTF-16).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum CodePoint {
    /// A valid Unicode scalar value (U+0000..U+D7FF or U+E000..U+10FFFF)
    Unicode(char),

    /// An unpaired/lone surrogate (U+D800..U+DFFF)
    UnpairedSurrogate(u16),
}

impl CodePoint {
    /// High surrogate range start (U+D800)
    pub const HIGH_SURROGATE_START: u16 = 0xD800;
    /// High surrogate range end (U+DBFF)
    pub const HIGH_SURROGATE_END: u16 = 0xDBFF;
    /// Low surrogate range start (U+DC00)
    pub const LOW_SURROGATE_START: u16 = 0xDC00;
    /// Low surrogate range end (U+DFFF)
    pub const LOW_SURROGATE_END: u16 = 0xDFFF;

    /// Creates a `CodePoint` from a UTF-16 code unit.
    ///
    /// If the code unit is a surrogate, it becomes an `UnpairedSurrogate`.
    /// Otherwise, it becomes a `Unicode` char.
    #[inline]
    #[must_use]
    pub const fn from_code_unit(unit: u16) -> Self {
        if unit >= Self::HIGH_SURROGATE_START && unit <= Self::LOW_SURROGATE_END {
            Self::UnpairedSurrogate(unit)
        } else {
            // SAFETY: unit is not a surrogate, so it's a valid char
            Self::Unicode(unsafe { char::from_u32_unchecked(unit as u32) })
        }
    }

    /// Creates a `CodePoint` from a u32 value.
    ///
    /// Returns `None` if the value is not a valid Unicode code point (> 0x10FFFF).
    #[inline]
    #[must_use]
    pub const fn from_u32(value: u32) -> Option<Self> {
        if value > 0x10FFFF {
            return None;
        }

        // Check if it's a surrogate
        if value >= Self::HIGH_SURROGATE_START as u32 && value <= Self::LOW_SURROGATE_END as u32 {
            Some(Self::UnpairedSurrogate(value as u16))
        } else {
            // SAFETY: value is a valid Unicode scalar value
            Some(Self::Unicode(unsafe { char::from_u32_unchecked(value) }))
        }
    }

    /// Returns the code point as a u32.
    #[inline]
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        match self {
            Self::Unicode(c) => c as u32,
            Self::UnpairedSurrogate(s) => s as u32,
        }
    }

    /// Returns the code point as a char, if it's a valid Unicode scalar value.
    ///
    /// Returns `None` for unpaired surrogates.
    #[inline]
    #[must_use]
    pub const fn as_char(self) -> Option<char> {
        match self {
            Self::Unicode(c) => Some(c),
            Self::UnpairedSurrogate(_) => None,
        }
    }

    /// Returns the number of UTF-16 code units needed to encode this code point.
    ///
    /// - BMP characters (U+0000..U+FFFF): 1 code unit
    /// - Supplementary characters (U+10000..U+10FFFF): 2 code units (surrogate pair)
    /// - Unpaired surrogates: 1 code unit
    #[inline]
    #[must_use]
    pub const fn len_utf16(self) -> usize {
        match self {
            Self::Unicode(c) => c.len_utf16(),
            Self::UnpairedSurrogate(_) => 1,
        }
    }

    /// Returns `true` if this is an unpaired surrogate.
    #[inline]
    #[must_use]
    pub const fn is_surrogate(self) -> bool {
        matches!(self, Self::UnpairedSurrogate(_))
    }

    /// Returns `true` if this is a high surrogate (U+D800..U+DBFF).
    #[inline]
    #[must_use]
    pub const fn is_high_surrogate(self) -> bool {
        match self {
            Self::UnpairedSurrogate(s) => {
                s >= Self::HIGH_SURROGATE_START && s <= Self::HIGH_SURROGATE_END
            }
            Self::Unicode(_) => false,
        }
    }

    /// Returns `true` if this is a low surrogate (U+DC00..U+DFFF).
    #[inline]
    #[must_use]
    pub const fn is_low_surrogate(self) -> bool {
        match self {
            Self::UnpairedSurrogate(s) => {
                s >= Self::LOW_SURROGATE_START && s <= Self::LOW_SURROGATE_END
            }
            Self::Unicode(_) => false,
        }
    }

    /// Encodes this code point as UTF-16 into the provided buffer.
    ///
    /// Returns the subslice of the buffer that contains the encoded character.
    ///
    /// # Panics
    ///
    /// Panics if the buffer is not large enough. A buffer of length 2 is always sufficient.
    #[inline]
    #[must_use]
    pub fn encode_utf16(self, dst: &mut [u16]) -> &mut [u16] {
        match self {
            Self::Unicode(c) => c.encode_utf16(dst),
            Self::UnpairedSurrogate(s) => {
                dst[0] = s;
                &mut dst[..1]
            }
        }
    }
}

impl Display for CodePoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unicode(c) => f.write_char(*c),
            Self::UnpairedSurrogate(s) => write!(f, "\\u{s:04X}"),
        }
    }
}

impl From<char> for CodePoint {
    #[inline]
    fn from(c: char) -> Self {
        Self::Unicode(c)
    }
}

/// Check if a u16 value is a high surrogate
#[inline]
#[must_use]
pub const fn is_high_surrogate(unit: u16) -> bool {
    unit >= CodePoint::HIGH_SURROGATE_START && unit <= CodePoint::HIGH_SURROGATE_END
}

/// Check if a u16 value is a low surrogate
#[inline]
#[must_use]
pub const fn is_low_surrogate(unit: u16) -> bool {
    unit >= CodePoint::LOW_SURROGATE_START && unit <= CodePoint::LOW_SURROGATE_END
}

/// Check if a u16 value is any surrogate (high or low)
#[inline]
#[must_use]
pub const fn is_surrogate(unit: u16) -> bool {
    unit >= CodePoint::HIGH_SURROGATE_START && unit <= CodePoint::LOW_SURROGATE_END
}

/// Decode a surrogate pair into a Unicode code point.
///
/// # Panics
///
/// Panics if `high` is not a high surrogate or `low` is not a low surrogate.
#[inline]
#[must_use]
pub const fn decode_surrogate_pair(high: u16, low: u16) -> char {
    debug_assert!(is_high_surrogate(high));
    debug_assert!(is_low_surrogate(low));

    let high = (high - CodePoint::HIGH_SURROGATE_START) as u32;
    let low = (low - CodePoint::LOW_SURROGATE_START) as u32;
    let code_point = 0x10000 + (high << 10) + low;

    // SAFETY: We decoded a valid surrogate pair, so the result is a valid Unicode scalar value
    unsafe { char::from_u32_unchecked(code_point) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_code_unit_ascii() {
        let cp = CodePoint::from_code_unit(0x0041); // 'A'
        assert_eq!(cp, CodePoint::Unicode('A'));
        assert_eq!(cp.as_u32(), 0x41);
        assert_eq!(cp.as_char(), Some('A'));
        assert!(!cp.is_surrogate());
    }

    #[test]
    fn test_from_code_unit_bmp() {
        let cp = CodePoint::from_code_unit(0x4E2D); // '中'
        assert_eq!(cp, CodePoint::Unicode('中'));
        assert_eq!(cp.len_utf16(), 1);
    }

    #[test]
    fn test_from_code_unit_high_surrogate() {
        let cp = CodePoint::from_code_unit(0xD83D);
        assert!(cp.is_surrogate());
        assert!(cp.is_high_surrogate());
        assert!(!cp.is_low_surrogate());
        assert_eq!(cp.as_char(), None);
    }

    #[test]
    fn test_from_code_unit_low_surrogate() {
        let cp = CodePoint::from_code_unit(0xDE00);
        assert!(cp.is_surrogate());
        assert!(!cp.is_high_surrogate());
        assert!(cp.is_low_surrogate());
    }

    #[test]
    fn test_from_u32_supplementary() {
        let cp = CodePoint::from_u32(0x1F600).unwrap(); // 😀
        assert_eq!(cp, CodePoint::Unicode('😀'));
        assert_eq!(cp.len_utf16(), 2);
    }

    #[test]
    fn test_from_u32_invalid() {
        assert!(CodePoint::from_u32(0x110000).is_none());
    }

    #[test]
    fn test_decode_surrogate_pair() {
        let c = decode_surrogate_pair(0xD83D, 0xDE00);
        assert_eq!(c, '😀');
    }

    #[test]
    fn test_encode_utf16() {
        let mut buf = [0u16; 2];

        // BMP character
        let cp = CodePoint::Unicode('A');
        let encoded = cp.encode_utf16(&mut buf);
        assert_eq!(encoded, &[0x0041]);

        // Supplementary character
        let cp = CodePoint::Unicode('😀');
        let encoded = cp.encode_utf16(&mut buf);
        assert_eq!(encoded, &[0xD83D, 0xDE00]);

        // Unpaired surrogate
        let cp = CodePoint::UnpairedSurrogate(0xD800);
        let encoded = cp.encode_utf16(&mut buf);
        assert_eq!(encoded, &[0xD800]);
    }
}

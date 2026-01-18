//! Iterators for YSString code units and code points.

use crate::codepoint::{decode_surrogate_pair, is_high_surrogate, is_low_surrogate, CodePoint};

/// Iterator over UTF-16 code units of a string.
#[derive(Clone, Debug)]
pub enum CodeUnits<'a> {
    /// Iterator over UTF-8 bytes (ASCII-only, each byte is a code unit)
    Utf8(std::str::Bytes<'a>),
    /// Iterator over UTF-16 code units
    Utf16(std::iter::Copied<std::slice::Iter<'a, u16>>),
}

impl<'a> Iterator for CodeUnits<'a> {
    type Item = u16;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Utf8(iter) => iter.next().map(u16::from),
            Self::Utf16(iter) => iter.next(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Utf8(iter) => iter.size_hint(),
            Self::Utf16(iter) => iter.size_hint(),
        }
    }
}

impl ExactSizeIterator for CodeUnits<'_> {
    #[inline]
    fn len(&self) -> usize {
        match self {
            Self::Utf8(iter) => iter.len(),
            Self::Utf16(iter) => iter.len(),
        }
    }
}

impl DoubleEndedIterator for CodeUnits<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::Utf8(iter) => iter.next_back().map(u16::from),
            Self::Utf16(iter) => iter.next_back(),
        }
    }
}

/// Iterator over code points of a string.
///
/// This iterator combines surrogate pairs into single code points,
/// but yields unpaired surrogates as `CodePoint::UnpairedSurrogate`.
#[derive(Clone, Debug)]
pub struct CodePoints<'a> {
    inner: CodeUnits<'a>,
    /// Buffered code unit for peeking
    peeked: Option<u16>,
}

impl<'a> CodePoints<'a> {
    /// Creates a new code points iterator from a code units iterator.
    #[inline]
    pub fn new(inner: CodeUnits<'a>) -> Self {
        Self {
            inner,
            peeked: None,
        }
    }
}

impl Iterator for CodePoints<'_> {
    type Item = CodePoint;

    fn next(&mut self) -> Option<Self::Item> {
        let unit = self.peeked.take().or_else(|| self.inner.next())?;

        if is_high_surrogate(unit) {
            // Check if the next unit is a low surrogate
            if let Some(next_unit) = self.inner.next() {
                if is_low_surrogate(next_unit) {
                    // Decode the surrogate pair
                    return Some(CodePoint::Unicode(decode_surrogate_pair(unit, next_unit)));
                }
                // Not a low surrogate, buffer it for next iteration
                self.peeked = Some(next_unit);
            }
            // Unpaired high surrogate
            Some(CodePoint::UnpairedSurrogate(unit))
        } else if is_low_surrogate(unit) {
            // Unpaired low surrogate
            Some(CodePoint::UnpairedSurrogate(unit))
        } else {
            // BMP character (not a surrogate)
            // SAFETY: unit is not a surrogate, so it's a valid char
            Some(CodePoint::Unicode(unsafe {
                char::from_u32_unchecked(unit as u32)
            }))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.inner.size_hint();
        // At minimum 1 code point per 2 code units (surrogate pairs)
        // At maximum 1 code point per code unit
        (0, upper)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_units_utf8() {
        let s = "Hello";
        let iter = CodeUnits::Utf8(s.bytes());
        let units: Vec<u16> = iter.collect();
        assert_eq!(units, vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]);
    }

    #[test]
    fn test_code_units_utf16() {
        let data = [0x48u16, 0x69, 0xD83D, 0xDE00]; // "Hi😀"
        let iter = CodeUnits::Utf16(data.iter().copied());
        let units: Vec<u16> = iter.collect();
        assert_eq!(units, vec![0x48, 0x69, 0xD83D, 0xDE00]);
    }

    #[test]
    fn test_code_points_ascii() {
        let s = "Hi";
        let iter = CodePoints::new(CodeUnits::Utf8(s.bytes()));
        let points: Vec<CodePoint> = iter.collect();
        assert_eq!(
            points,
            vec![CodePoint::Unicode('H'), CodePoint::Unicode('i')]
        );
    }

    #[test]
    fn test_code_points_with_emoji() {
        let data = [0x48u16, 0x69, 0xD83D, 0xDE00]; // "Hi😀"
        let iter = CodePoints::new(CodeUnits::Utf16(data.iter().copied()));
        let points: Vec<CodePoint> = iter.collect();
        assert_eq!(
            points,
            vec![
                CodePoint::Unicode('H'),
                CodePoint::Unicode('i'),
                CodePoint::Unicode('😀')
            ]
        );
    }

    #[test]
    fn test_code_points_unpaired_high_surrogate() {
        let data = [0x41u16, 0xD800, 0x42]; // "A<high>B"
        let iter = CodePoints::new(CodeUnits::Utf16(data.iter().copied()));
        let points: Vec<CodePoint> = iter.collect();
        assert_eq!(
            points,
            vec![
                CodePoint::Unicode('A'),
                CodePoint::UnpairedSurrogate(0xD800),
                CodePoint::Unicode('B')
            ]
        );
    }

    #[test]
    fn test_code_points_unpaired_low_surrogate() {
        let data = [0x41u16, 0xDC00, 0x42]; // "A<low>B"
        let iter = CodePoints::new(CodeUnits::Utf16(data.iter().copied()));
        let points: Vec<CodePoint> = iter.collect();
        assert_eq!(
            points,
            vec![
                CodePoint::Unicode('A'),
                CodePoint::UnpairedSurrogate(0xDC00),
                CodePoint::Unicode('B')
            ]
        );
    }
}

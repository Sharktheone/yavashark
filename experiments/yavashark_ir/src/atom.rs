use std::fmt::{Debug, Formatter};

const INLINE_CAPACITY: usize = 7;
const INTERNED: u8 = u8::MAX;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Atom {
    len: u8,
    data: [u8; INLINE_CAPACITY],
}

impl Atom {
    #[must_use]
    pub fn inline(s: &str) -> Option<Self> {
        Self::inline_units(s.bytes().map(u16::from), s.len())
    }

    pub fn inline_bytes(bytes: &[u8]) -> Option<Self> {
        let len = bytes.len();

        if len > INLINE_CAPACITY {
            return None;
        }

        let mut data = [0; INLINE_CAPACITY];

        //TODO(perf): this can be optimized by two overlapping 32bit loads plus a slow path instead of the heavy variable width load here!
        data[..len].copy_from_slice(bytes);

        Some(Self {
            len: len as u8,
            data,
        })
    }

    fn inline_units(units: impl Iterator<Item = u16>, len: usize) -> Option<Self> {
        if len > INLINE_CAPACITY {
            return None;
        }

        let mut data = [0; INLINE_CAPACITY];

        for (byte, unit) in data.iter_mut().zip(units) {
            //TODO: We should start using Latin1 instead of plain ASCII
            if unit >= 0x80 {
                return None;
            }

            *byte = unit as u8;
        }

        Some(Self {
            len: len as u8,
            data,
        })
    }

    const fn interned(index: u32) -> Self {
        let [a, b, c, d] = index.to_le_bytes();

        Self {
            len: INTERNED,
            data: [a, b, c, d, 0, 0, 0],
        }
    }

    #[must_use]
    pub fn as_inline(&self) -> Option<&str> {
        let bytes = self.data.get(..self.len as usize)?;

        std::str::from_utf8(bytes).ok()
    }

    const fn index(self) -> Option<usize> {
        if self.len != INTERNED {
            return None;
        }

        let [a, b, c, d, ..] = self.data;

        Some(u32::from_le_bytes([a, b, c, d]) as usize)
    }
}

impl Debug for Atom {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.as_inline() {
            Some(s) => write!(f, "{s:?}"),
            None => write!(f, "#{}", self.index().unwrap_or_default()),
        }
    }
}

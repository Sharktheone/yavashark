use std::fmt::{Debug, Formatter};

const INLINE_CAPACITY: usize = 7;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Atom {
    len: Len,
    data: [u8; INLINE_CAPACITY],
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum Len {
    Empty = 0,
    L1 = 1,
    L2 = 2,
    L3 = 3,
    L4 = 4,
    L5 = 5,
    L6 = 6,
    L7 = 7,
    Interned = 255,
}

impl Len {
    const fn new(len: usize) -> Option<Self> {
        match len {
            0 => Some(Self::Empty),
            1 => Some(Self::L1),
            2 => Some(Self::L2),
            3 => Some(Self::L3),
            4 => Some(Self::L4),
            5 => Some(Self::L5),
            6 => Some(Self::L6),
            7 => Some(Self::L7),
            _ => None,
        }
    }
}

impl Atom {
    #[must_use]
    pub fn inline(s: &str) -> Option<Self> {
        Self::inline_units(s.bytes().map(u16::from), s.len())
    }

    pub fn inline_bytes(bytes: &[u8]) -> Option<Self> {
        let len = bytes.len();

        let l = Len::new(len)?;

        let mut data = [0; INLINE_CAPACITY];

        //TODO(perf): this can be optimized by two overlapping 32bit loads plus a slow path instead of the heavy variable width load here!
        data[..len].copy_from_slice(bytes);

        Some(Self { len: l, data })
    }

    pub fn inline_bigint(value: u64) -> Option<Self> {
        let bytes = value.to_le_bytes();
        let len = bytes.iter().rposition(|&b| b != 0).map_or(0, |pos| pos + 1);

        Self::inline_bytes(&bytes[..len])
    }

    fn inline_units(units: impl Iterator<Item = u16>, len: usize) -> Option<Self> {
        let l = Len::new(len)?;

        let mut data = [0; INLINE_CAPACITY];

        for (byte, unit) in data.iter_mut().zip(units) {
            //TODO: We should start using Latin1 instead of plain ASCII
            if unit >= 0x80 {
                return None;
            }

            *byte = unit as u8;
        }

        Some(Self { len: l, data })
    }

    const fn interned(index: u32) -> Self {
        let [a, b, c, d] = index.to_le_bytes();

        Self {
            len: Len::Interned,
            data: [a, b, c, d, 0, 0, 0],
        }
    }

    #[must_use]
    pub fn as_inline(&self) -> Option<&str> {
        let bytes = self.data.get(..self.len as usize)?;

        std::str::from_utf8(bytes).ok()
    }

    fn index(self) -> Option<usize> {
        if self.len != Len::Interned {
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

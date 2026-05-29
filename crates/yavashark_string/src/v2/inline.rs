use std::mem::size_of;
use std::ops::Deref;

pub struct InlineAscii {
    len: InlineLen,
    bytes: [u8; Self::CAPACITY],
}

#[repr(Rust, packed)]
pub struct InlineWtf16 {
    len: InlineLenWtf,
    bytes: [u16; Self::CAPACITY],
}

impl InlineAscii {
    const CAPACITY: usize = 23;

    pub fn new() -> Self {
        Self {
            len: InlineLen::Empty,
            bytes: [0; Self::CAPACITY],
        }
    }

    pub fn try_from_str(s: &str) -> Option<Self> {
        if s.len() > Self::CAPACITY {
            return None;
        }

        let mut bytes = [0; Self::CAPACITY];
        bytes[..s.len()].copy_from_slice(s.as_bytes());

        Some(Self {
            len: InlineLen::from_u32(s.len() as u32)?,
            bytes,
        })
    }

    pub fn len(&self) -> u32 {
        self.len.to_u32()
    }

    pub fn slice(self, start: u32, end: u32) -> Option<Self> {
        if start > end || end > self.len.to_u32() {
            return None;
        }

        let mut bytes = [0; Self::CAPACITY];
        let len = InlineLen::from_u32(end - start)?;

        let end = end as usize;
        let start = start as usize;

        bytes[..(end - start)].copy_from_slice(&self.bytes[start..end]);

        Some(Self { len, bytes })
    }
}

impl AsRef<str> for InlineAscii {
    fn as_ref(&self) -> &str {
        let len = self.len.to_u32() as usize;

        unsafe { std::str::from_utf8_unchecked(&self.bytes[..len]) }
    }
}

impl Deref for InlineAscii {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl InlineWtf16 {
    const CAPACITY: usize = 11;

    pub fn new() -> Self {
        Self {
            len: InlineLenWtf::Empty,
            bytes: [0; Self::CAPACITY],
        }
    }

    pub fn try_from_slice(units: &[u16]) -> Option<Self> {
        if units.len() > Self::CAPACITY {
            return None;
        }

        let mut bytes = [0; Self::CAPACITY];
        bytes[..units.len()].copy_from_slice(units);

        Some(Self {
            len: InlineLenWtf::from_u32(units.len() as u32)?,
            bytes,
        })
    }

    pub fn len(&self) -> u32 {
        self.len.to_u32()
    }

    pub fn slice(self, start: u32, end: u32) -> Option<Self> {
        if start > end || end > self.len.to_u32() {
            return None;
        }

        let mut bytes = [0; Self::CAPACITY];
        let sbytes = self.bytes;
        let len = InlineLenWtf::from_u32(end - start)?;

        let end = end as usize;
        let start = start as usize;

        bytes[..(end - start)].copy_from_slice(&sbytes[start..end]);

        Some(Self { len, bytes })
    }
}

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InlineLenWtf {
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
}

impl InlineLen {
    pub fn from_u32(len: u32) -> Option<Self> {
        match len {
            0 => Some(Self::Empty),
            1 => Some(Self::Len1),
            2 => Some(Self::Len2),
            3 => Some(Self::Len3),
            4 => Some(Self::Len4),
            5 => Some(Self::Len5),
            6 => Some(Self::Len6),
            7 => Some(Self::Len7),
            8 => Some(Self::Len8),
            9 => Some(Self::Len9),
            10 => Some(Self::Len10),
            11 => Some(Self::Len11),
            12 => Some(Self::Len12),
            13 => Some(Self::Len13),
            14 => Some(Self::Len14),
            15 => Some(Self::Len15),
            16 => Some(Self::Len16),
            17 => Some(Self::Len17),
            18 => Some(Self::Len18),
            19 => Some(Self::Len19),
            20 => Some(Self::Len20),
            21 => Some(Self::Len21),
            22 => Some(Self::Len22),
            23 => Some(Self::Len23),
            _ => None,
        }
    }

    pub fn to_u32(self) -> u32 {
        self as u32
    }
}

impl InlineLenWtf {
    pub fn from_u32(len: u32) -> Option<Self> {
        match len {
            0 => Some(Self::Empty),
            1 => Some(Self::Len1),
            2 => Some(Self::Len2),
            3 => Some(Self::Len3),
            4 => Some(Self::Len4),
            5 => Some(Self::Len5),
            6 => Some(Self::Len6),
            7 => Some(Self::Len7),
            8 => Some(Self::Len8),
            9 => Some(Self::Len9),
            10 => Some(Self::Len10),
            11 => Some(Self::Len11),
            _ => None,
        }
    }

    pub fn to_u32(self) -> u32 {
        self as u32
    }
}

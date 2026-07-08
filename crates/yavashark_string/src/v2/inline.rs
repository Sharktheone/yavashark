use std::mem::size_of;
use std::ops::Deref;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InlineAscii {
    len: InlineLen,
    bytes: [u8; Self::CAPACITY],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(Rust, packed)]
pub struct InlineWtf16 {
    len: InlineLenWtf,
    bytes: [u16; Self::CAPACITY],
}

impl InlineAscii {
    pub const CAPACITY: usize = 31;

    pub const fn new() -> Self {
        Self {
            len: InlineLen::Empty,
            bytes: [0; Self::CAPACITY],
        }
    }

    pub fn from_bytes(bytes: [u8; Self::CAPACITY], len: u32) -> Self {
        let len = InlineLen::from_u32(len).unwrap_or(InlineLen::Len31);

        Self { len, bytes }
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

    pub const fn len(&self) -> u32 {
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
    pub const CAPACITY: usize = 15;

    pub const fn new() -> Self {
        Self {
            len: InlineLenWtf::Empty,
            bytes: [0; Self::CAPACITY],
        }
    }

    pub fn from_bytes(bytes: [u16; Self::CAPACITY], len: u32) -> Self {
        let len = InlineLenWtf::from_u32(len).unwrap_or(InlineLenWtf::Len15);

        Self { len, bytes }
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

    pub const fn len(&self) -> u32 {
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
    Len24,
    Len25,
    Len26,
    Len27,
    Len28,
    Len29,
    Len30,
    Len31,
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
    Len12,
    Len13,
    Len14,
    Len15,
}

impl InlineLen {
    pub const fn from_u32(len: u32) -> Option<Self> {
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
            24 => Some(Self::Len24),
            25 => Some(Self::Len25),
            26 => Some(Self::Len26),
            27 => Some(Self::Len27),
            28 => Some(Self::Len28),
            29 => Some(Self::Len29),
            30 => Some(Self::Len30),
            31 => Some(Self::Len31),
            _ => None,
        }
    }

    pub const fn to_u32(self) -> u32 {
        self as u32
    }
}

impl InlineLenWtf {
    pub const fn from_u32(len: u32) -> Option<Self> {
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
            _ => None,
        }
    }

    pub const fn to_u32(self) -> u32 {
        self as u32
    }
}

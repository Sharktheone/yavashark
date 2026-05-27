pub struct InlineAscii {
    len: InlineLen,
    bytes: [u8; 23],
}

#[repr(Rust, packed)]
pub struct InlineWtf16 {
    len: InlineLenWtf,
    bytes: [u16; 11],
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
    pub fn from_usize(len: usize) -> Option<Self> {
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

    pub fn to_usize(self) -> usize {
        self as usize
    }
}

impl InlineLenWtf {
    pub fn from_usize(len: usize) -> Option<Self> {
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

    pub fn to_usize(self) -> usize {
        self as usize
    }
}

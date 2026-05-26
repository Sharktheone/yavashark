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

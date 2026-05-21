use std::cell::UnsafeCell;

pub struct YSString {
    inner: UnsafeCell<Inner>,
}

enum Inner {
    Heap(HeapString),
    InlineAscii(InlineAscii),
    InlineWtf16(InlineWtf16),
    Rope(RopeString),
}

enum Type {
    Ascii,
    Wtf16,
}

enum Storage {
    Inline,
    Rc,
    Rope,
    Static,
}

#[repr(Rust, packed)]
struct HeapString {
    ptr: *const (), //actual ptr = ptr - ptr_offset
    len: u32,
    ptr_offset: u32,
    len_offset: u32, //len + len_offset = capacity
    ty: Type,
    storage: Storage,
}

struct InlineAscii {
    len: InlineLen,
    bytes: [u8; 23],
}

#[repr(Rust, packed)]
struct InlineWtf16 {
    len: InlineLen,
    bytes: [u16; 11],
}

struct RopeString {
    ptr1: *const YSString,
    ptr2: *const YSString
}

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

// TODO: string v3 should store the string data inline to the definition
//       this will allow for faster access to the string data and less memory allocations
// There will be one pointer and which has an tag to what the string actually is. That can be
// - InlineAscii
// - InlineWtf16
// - Slice
// - Rope
// - External
// The data will be unsized, since all strings will be now stored inline.

// There should also be an optimization which the interpreter can do when it sees that a string is being mutated while not being shared.

use std::mem::ManuallyDrop;

type Gc<T> = *mut T;

pub struct YSString {}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct StringHeader {
    ty: Type,
}

#[derive(Debug, Copy, Clone)]
enum Type {
    Ascii,
    Wtf16,
    Slice,
    Rope,
    RopeSlice,
    External,
}

#[repr(C)]
#[derive(Debug)]
struct AsciiString {
    header: StringHeader,
    len: u32,
    data: [u8],
}

#[repr(C)]
#[derive(Debug)]
struct Wtf16String {
    header: StringHeader,
    len: u32,
    data: [u16],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct SliceString {
    header: StringHeader,
    start: u32,
    len: u32,
    ptr: Gc<YSString>,
}

#[repr(C)]
#[derive(Debug)]
struct RopeString {
    header: StringHeader,
    elems: u32,
    ropes: [Gc<YSString>],
}

#[derive(Debug, Copy, Clone)]
struct RopeSliceElem {
    start: u32,
    len: u32,
    ptr: Gc<YSString>,
}

#[repr(C)]
#[derive(Debug)]
struct RopeSliceString {
    header: StringHeader,
    elems: u32,
    ropes: [RopeSliceElem], //TODO: We need to test if more than 2 ropes actually is better for perf
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct ExternalString {
    header: StringHeader,
    len: u32,
    data: *const u8,
    drop: Option<unsafe extern "C" fn(*const u8)>,
}

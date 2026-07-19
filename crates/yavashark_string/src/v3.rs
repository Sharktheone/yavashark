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

use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;

type Gc<T> = *mut T;

pub struct YSString {
    data: NonNull<StringHeader>,
    phantom: PhantomData<StringHeader>,
}


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
    phantom: PhantomData<[u8]>,
}

#[repr(C)]
#[derive(Debug)]
struct Wtf16String {
    header: StringHeader,
    len: u32,
    phantom: PhantomData<[u16]>,
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
    left: Gc<YSString>,
    right: Gc<YSString>,
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
    left: RopeSliceElem,
    right: RopeSliceElem,
}

#[repr(C)]
#[derive(Debug, Clone)]
struct ExternalString {
    header: StringHeader,
    len: u32,
    data: *const u8,
    drop: Option<unsafe extern "C" fn(*const u8, u32)>,
}

impl Drop for ExternalString {
    fn drop(&mut self) {
        if let Some(drop) = self.drop {
            unsafe {
                drop(self.data, self.len);
            }
        }
    }
}

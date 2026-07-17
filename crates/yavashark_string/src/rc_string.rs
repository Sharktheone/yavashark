use std::alloc::Layout;
use std::marker::PhantomData;
use std::ptr::NonNull;

#[repr(C)]
struct Header {
    count: usize,
    capacity: u32,
    init_to: u32,
}


struct RcAsciiString {
    header: NonNull<Header>,
    len: u32,
    phantom: PhantomData<[u8]>,
}


struct RcWtf16String {
    header: NonNull<Header>,
    len: u32,
    phantom: PhantomData<[u16]>,
}

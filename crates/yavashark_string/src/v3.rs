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

use std::alloc::Layout;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::ptr::NonNull;
use std::{alloc, slice};

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
    cap: u32,
    len: u32,
    phantom: PhantomData<[u8]>,
}

impl AsciiString {
    fn layout(cap: u32) -> Layout {
        let layout = Layout::new::<Self>();

        #[allow(clippy::expect_used)]
        layout
            .extend(Layout::array::<u8>(cap as usize).expect("cannot happen"))
            .expect("cannot happen")
            .0
            .pad_to_align()
    }

    fn with_capacity(cap: u32) -> Option<NonNull<Self>> {
        let layout = Self::layout(cap);

        let ptr = unsafe {
            #[allow(clippy::cast_ptr_alignment)]
            alloc::alloc(layout).cast::<Self>()
        };

        let ptr = NonNull::new(ptr)?;

        unsafe {
            ptr.write(Self {
                header: StringHeader { ty: Type::Ascii },
                cap,
                len: 0,
                phantom: PhantomData,
            });
        }

        Some(ptr)
    }

    fn new_with_extra(str: &str, extra: u32) -> Option<NonNull<Self>> {
        let cap = str
            .len()
            .saturating_add(extra as usize)
            .min(u32::MAX as usize) as u32;

        let slf = Self::with_capacity(cap)?;

        unsafe {
            Self::write(slf, str);
        }

        Some(slf)
    }

    fn new(str: &str) -> Option<NonNull<Self>> {
        Self::new_with_extra(str, 0)
    }

    unsafe fn get_data_ptr(slf: NonNull<Self>) -> &'static mut [u8] {
        unsafe {
            let ptr = slf.offset(1).cast::<u8>();

            slice::from_raw_parts_mut(ptr.as_ptr(), (*slf.as_ptr()).cap as usize)
        }
    }

    unsafe fn get_data_ptr_ref(slf: NonNull<Self>) -> &'static [u8] {
        unsafe {
            let ptr = slf.offset(1).cast::<u8>();

            slice::from_raw_parts(ptr.as_ptr(), (*slf.as_ptr()).cap as usize)
        }
    }

    unsafe fn write(slf: NonNull<Self>, str: &str) {
        unsafe {
            let offset = (*slf.as_ptr()).len as usize;

            let data = Self::get_data_ptr(slf);

            data[offset..offset + str.len()].copy_from_slice(str.as_bytes());
        }
    }

    unsafe fn drop(slf: NonNull<Self>) {
        unsafe {
            let layout = Self::layout((*slf.as_ptr()).cap);

            alloc::dealloc(slf.cast().as_ptr(), layout);
        }
    }
}

impl Deref for AsciiString {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        unsafe {
            let slice = Self::get_data_ptr_ref(NonNull::from_ref(self));

            str::from_utf8_unchecked(slice)
        }
    }
}

#[repr(C)]
#[derive(Debug)]
struct Wtf16String {
    header: StringHeader,
    cap: u32,
    len: u32,
    phantom: PhantomData<[u16]>,
}

impl Wtf16String {
    fn layout(cap: u32) -> Layout {
        let layout = Layout::new::<Self>();

        #[allow(clippy::expect_used)]
        layout
            .extend(Layout::array::<u16>(cap as usize).expect("cannot happen"))
            .expect("cannot happen")
            .0
            .pad_to_align()
    }

    fn with_capacity(cap: u32) -> Option<NonNull<Self>> {
        let layout = Self::layout(cap);

        let ptr = unsafe {
            #[allow(clippy::cast_ptr_alignment)]
            alloc::alloc(layout).cast::<Self>()
        };

        let ptr = NonNull::new(ptr)?;

        unsafe {
            ptr.write(Self {
                header: StringHeader { ty: Type::Wtf16 },
                cap,
                len: 0,
                phantom: PhantomData,
            });
        }

        Some(ptr)
    }

    fn new_with_extra(str: &[u16], extra: u32) -> Option<NonNull<Self>> {
        let cap = str
            .len()
            .saturating_add(extra as usize)
            .min(u32::MAX as usize) as u32;

        let slf = Self::with_capacity(cap)?;

        unsafe {
            Self::write(slf, str);
        }

        Some(slf)
    }

    fn new(str: &[u16]) -> Option<NonNull<Self>> {
        Self::new_with_extra(str, 0)
    }

    unsafe fn get_data_ptr(slf: NonNull<Self>) -> &'static mut [u16] {
        unsafe {
            let ptr = slf.offset(1).cast::<u16>();

            slice::from_raw_parts_mut(ptr.as_ptr(), (*slf.as_ptr()).cap as usize)
        }
    }

    unsafe fn get_data_ptr_ref(slf: NonNull<Self>) -> &'static [u16] {
        unsafe {
            let ptr = slf.offset(1).cast::<u16>();

            slice::from_raw_parts(ptr.as_ptr(), (*slf.as_ptr()).cap as usize)
        }
    }

    unsafe fn write(slf: NonNull<Self>, str: &[u16]) {
        unsafe {
            let offset = (*slf.as_ptr()).len as usize;

            let data = Self::get_data_ptr(slf);

            data[offset..offset + str.len()].copy_from_slice(str);
        }
    }

    unsafe fn drop(slf: NonNull<Self>) {
        unsafe {
            let layout = Self::layout((*slf.as_ptr()).cap);

            alloc::dealloc(slf.cast().as_ptr(), layout);
        }
    }
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

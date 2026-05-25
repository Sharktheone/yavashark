use std::{cell::UnsafeCell, ptr::NonNull, rc::Rc};

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
    Rc,
    Static,
}

#[repr(Rust, packed)]
struct HeapString {
    ptr: NonNull<()>, //actual ptr = ptr - ptr_offset
    len: u32,
    ptr_offset: u32,
    len_offset: u32, //len + len_offset = capacity
    ty: Type,
    storage: Storage,
}



impl HeapString {
    fn from_static_ascii(s: &'static str) -> Self {
        Self {
            ptr: NonNull::from(s).cast(),
            len: s.len() as u32,
            ptr_offset: 0,
            len_offset: 0,
            ty: Type::Ascii,
            storage: Storage::Static,
        }
    }

    fn from_static_wtf16(s: &'static [u16]) -> Self {
        Self {
            ptr: NonNull::from(s).cast(), 
            len: s.len() as u32,
            ptr_offset: 0,
            len_offset: 0,
            ty: Type::Wtf16,
            storage: Storage::Static,
        }
    }

    fn from_rc_ascii(s: Rc<str>) -> Self {
        let len = s.len() as u32;

        let ptr = Rc::into_raw(s);

        
        
        Self {
            // SAFETY: Rc::into:raw always returns a non-null and aligned pointer
            ptr: unsafe { NonNull::new_unchecked(ptr.cast_mut().cast()) },
            len,
            ptr_offset: 0,
            len_offset: 0,
            ty: Type::Ascii,
            storage: Storage::Rc,
        }
    }

    fn from_rc_wtf16(s: Rc<[u16]>) -> Self {
        let len = s.len() as u32;

        let ptr = Rc::into_raw(s);

        Self {
            // SAFETY: Rc::into:raw always returns a non-null and aligned pointer
            ptr: unsafe { NonNull::new_unchecked(ptr.cast_mut().cast()) },
            len,
            ptr_offset: 0,
            len_offset: 0,
            ty: Type::Wtf16,
            storage: Storage::Rc,
        }
    }

    const fn get_base_ptr(&self) -> NonNull<()> {
        // SAFETY: ptr is always valid and properly aligned, and ptr_offset is always <= u32::MAX
        unsafe { self.ptr.sub(self.ptr_offset as usize) }
    }

    const fn storage_len(&self) -> usize {
        // SAFETY: len_offset is always <= u32::MAX
        (self.len + self.len_offset) as usize
    }

    


    
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

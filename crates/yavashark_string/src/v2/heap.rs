use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use std::rc::Rc;
use std::slice;
use crate::v2::{StringRef, Type};



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Storage {
    Rc,
    Static,
}

#[repr(Rust, packed)]
pub struct HeapString {
    ptr: NonNull<()>, //actual ptr = ptr - ptr_offset
    len: u32,
    ptr_offset: u32,
    len_offset: u32, //len + len_offset = capacity
    ty: Type,
    storage: Storage,
}


#[derive(Debug, Clone, PartialEq, Eq)]
enum HeapStringStorage {
    StaticAscii(&'static str),
    StaticWtf16(&'static [u16]),
    RcAscii(ManuallyDrop<Rc<str>>),
    RcWtf16(ManuallyDrop<Rc<[u16]>>),
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


    const fn as_ptr(&self) -> NonNull<()> {
        self.ptr
    }

    const fn as_ref(&'_ self) -> StringRef<'_> {
        match self.ty {
            Type::Ascii => {
                let str = unsafe {
                    // SAFETY: ptr is valid and properly aligned, and len is correct
                    let slice = slice::from_raw_parts(self.ptr.as_ptr().cast(), self.len as usize);

                    //SAFETY: slice is valid UTF-8
                    str::from_utf8_unchecked(slice)
                };

                StringRef::Ascii(str)
            },
            Type::Wtf16 => {
                // SAFETY: base_ptr is valid and properly aligned, and len is correct
                let slice = unsafe { slice::from_raw_parts(self.ptr.as_ptr().cast(), self.len as usize) };
                StringRef::Wtf16(slice)
            },
        }
    }


    fn storage(&self) -> HeapStringStorage {
        let ptr = self.get_base_ptr().as_ptr();

        match (self.ty, self.storage) {
            (Type::Ascii, Storage::Static) => {
                // SAFETY: ptr is valid and properly aligned, and len is correct
                let slice = unsafe { slice::from_raw_parts(ptr.cast(), self.len as usize) };
                let s = unsafe { str::from_utf8_unchecked(slice) };
                HeapStringStorage::StaticAscii(s)
            }
            (Type::Wtf16, Storage::Static) => {
                // SAFETY: ptr is valid and properly aligned, and len is correct
                let slice = unsafe { slice::from_raw_parts(ptr.cast(), self.len as usize) };
                HeapStringStorage::StaticWtf16(slice)
            },
            (Type::Ascii, Storage::Rc) => {
                let rc = unsafe {
                    let slice = slice::from_raw_parts(ptr.cast(), self.len as usize);
                    let str = str::from_utf8_unchecked(slice);


                    Rc::from_raw(&raw const *str)
                };

                HeapStringStorage::RcAscii(ManuallyDrop::new(rc))
            }
            (Type::Wtf16, Storage::Rc) => {
                let rc = unsafe {
                    let slice = slice::from_raw_parts(ptr.cast(), self.len as usize);

                    Rc::from_raw(&raw const *slice)
                };

                HeapStringStorage::RcWtf16(ManuallyDrop::new(rc))
            }
        }
    }
}

impl Drop for HeapString {
    fn drop(&mut self) {
        if Storage::Rc == self.storage {
            // SAFETY: get_base_ptr returns a valid pointer, and len is correct
            let ptr = unsafe { self.get_base_ptr().as_ptr() };

            match self.ty {
                Type::Ascii => {
                    let slice = unsafe { slice::from_raw_parts(ptr.cast(), self.len as usize) };
                    let str = unsafe { str::from_utf8_unchecked(slice) };
                    let _rc: Rc<str> = unsafe { Rc::from_raw(&raw const *str) };
                }
                Type::Wtf16 => {
                    let slice = unsafe { slice::from_raw_parts(ptr.cast(), self.len as usize) };
                    let _rc: Rc<[u16]> = unsafe { Rc::from_raw(&raw const *slice) };
                }
            }
        }
    }
}

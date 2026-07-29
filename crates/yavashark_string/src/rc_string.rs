use std::alloc::Layout;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr;
use std::ptr::NonNull;

#[repr(C)]
struct Header {
    count: usize,
    capacity: u32,
    init_to: u32,
}

#[repr(Rust, packed)]
pub struct RcAsciiString {
    header: NonNull<Header>,
    len: u32,
    phantom: PhantomData<[u8]>,
}

#[repr(Rust, packed)]
pub struct RcWtf16String {
    header: NonNull<Header>,
    len: u32,
    phantom: PhantomData<[u16]>,
}

impl RcAsciiString {
    pub fn with_capacity(capacity: u32) -> Self {
        let header = Header::alloc_u8(capacity);

        Self {
            header,
            len: 0,
            phantom: PhantomData,
        }
    }

    pub fn new_with_extra(str: &str, extra: u32) -> Self {
        let len = str.len().min(u32::MAX as usize) as u32;

        let header = Header::alloc_u8((len).saturating_add(extra));
        let mut rc_string = Self {
            header,
            len,
            phantom: PhantomData,
        };

        unsafe {
            (*header.as_ptr()).init_to = len;
        }

        unsafe {
            let ptr = Header::get_data_u8(header);

            ptr::copy_nonoverlapping(str.as_ptr(), ptr, len as usize);
        }

        rc_string
    }

    pub fn new(str: &str) -> Self {
        Self::new_with_extra(str, 0)
    }

    pub fn extend(&self, str: &str) -> Option<Self> {
        let cap = unsafe { (*self.header.as_ptr()).capacity };
        let init_to = unsafe { (*self.header.as_ptr()).init_to };

        if init_to != self.len {
            // there already is something behind us

            let additional =
                unsafe { &Header::data_slice_u8(self.header)[self.len as usize..init_to as usize] };

            if additional.starts_with(str.as_bytes()) {
                let mut new = self.clone();

                new.len += str.len() as u32;

                return Some(new);
            }

            return None;
        }

        let remaining = cap - init_to;

        if str.len() > remaining as usize {
            return None;
        }

        let mut data = Header::get_data_u8(self.header);

        let it = init_to as usize;
        let to = it + str.len();

        if to >= self.len as usize {
            return None;
        }

        unsafe {
            ptr::copy_nonoverlapping(str.as_ptr(), data.add(it), str.len());

            (*self.header.as_ptr()).init_to += str.len() as u32;
        }

        let mut new = self.clone();

        new.len += str.len() as u32;

        Some(new)
    }
}

impl Drop for RcAsciiString {
    fn drop(&mut self) {
        unsafe {
            (*self.header.as_ptr()).count = (*self.header.as_ptr()).count.saturating_sub(1);
        }

        unsafe {
            if (*self.header.as_ptr()).count == 0 {
                Header::drop_u8(self.header);
            }
        }
    }
}

impl Clone for RcAsciiString {
    fn clone(&self) -> Self {
        unsafe {
            assert_ne!((*self.header.as_ptr()).count, usize::MAX, "RcAsciiString count overflow");

            (*self.header.as_ptr()).count += 1;
        }

        Self {
            header: self.header,
            len: self.len,
            phantom: PhantomData,
        }
    }
}

impl Deref for RcAsciiString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        unsafe {
            let len = self.len as usize;
            let data_slice = Header::data_slice_u8_to(self.header, len);
            std::str::from_utf8_unchecked(data_slice)
        }
    }
}

impl RcWtf16String {
    pub fn with_capacity(capacity: u32) -> Self {
        let header = Header::alloc_u16(capacity);

        Self {
            header,
            len: 0,
            phantom: PhantomData,
        }
    }

    pub fn new_with_extra(str: &[u16], extra: u32) -> Self {
        let len = str.len().min(u32::MAX as usize) as u32;

        let header = Header::alloc_u16((len).saturating_add(extra));
        let mut rc_string = Self {
            header,
            len,
            phantom: PhantomData,
        };

        unsafe {
            (*header.as_ptr()).init_to = str.len() as u32;

            let data = Header::get_data_u16(header);

            ptr::copy_nonoverlapping(str.as_ptr(), data, len as usize);
        }

        rc_string
    }

    pub fn new(str: &[u16]) -> Self {
        Self::new_with_extra(str, 0)
    }

    pub fn extend(&self, str: &[u16]) -> Option<Self> {
        let cap = unsafe { (*self.header.as_ptr()).capacity };
        let init_to = unsafe { (*self.header.as_ptr()).init_to };

        if init_to != self.len {
            // there already is something behind us

            let additional = unsafe {
                &Header::data_slice_u16(self.header)[self.len as usize..init_to as usize]
            };

            if additional.starts_with(str) {
                let mut new = self.clone();

                new.len += str.len() as u32;

                return Some(new);
            }

            return None;
        }

        let remaining = cap - init_to;

        if str.len() > remaining as usize {
            return None;
        }

        unsafe {
            let mut data = Header::get_data_u16(self.header);

            let it = init_to as usize;
            let to = it + str.len();

            if to >= self.len as usize {
                return None;
            }

            ptr::copy_nonoverlapping(str.as_ptr(), data.add(it), str.len());

            (*self.header.as_ptr()).init_to += str.len() as u32;
        }

        let mut new = self.clone();

        new.len += str.len() as u32;

        Some(new)
    }
}

impl Drop for RcWtf16String {
    fn drop(&mut self) {
        unsafe {
            (*self.header.as_ptr()).count = (*self.header.as_ptr()).count.saturating_sub(1);
        }

        unsafe {
            if (*self.header.as_ptr()).count == 0 {
                Header::drop_u16(self.header);
            }
        }
    }
}

impl Clone for RcWtf16String {
    fn clone(&self) -> Self {
        unsafe {
            assert_ne!((*self.header.as_ptr()).count, usize::MAX, "RcWtf16String count overflow");

            (*self.header.as_ptr()).count += 1;
        }

        Self {
            header: self.header,
            len: self.len,
            phantom: PhantomData,
        }
    }
}

impl Deref for RcWtf16String {
    type Target = [u16];

    fn deref(&self) -> &Self::Target {
        unsafe {
            let len = self.len as usize;
            Header::data_slice_u16_to(self.header, len)
        }
    }
}

impl Header {
    fn layout<T>(cap: u32) -> Layout {
        #[allow(clippy::expect_used)]
        Layout::new::<Self>()
            .extend(Layout::array::<T>(cap as usize).expect("layout failed"))
            .expect("layout failed")
            .0
            .pad_to_align()
    }

    unsafe fn alloc<T>(capacity: u32) -> NonNull<Self> {
        let layout = Self::layout::<T>(capacity);

        #[allow(clippy::cast_ptr_alignment)]
        let ptr = unsafe { std::alloc::alloc(layout).cast::<Self>() };

        let Some(ptr) = NonNull::new(ptr) else {
            std::alloc::handle_alloc_error(layout);
        };

        unsafe {
            std::ptr::write(
                ptr.as_ptr(),
                Self {
                    count: 1,
                    capacity,
                    init_to: 0,
                },
            );
        }

        ptr
    }

    fn alloc_u8(capacity: u32) -> NonNull<Self> {
        unsafe { Self::alloc::<u8>(capacity) }
    }

    fn alloc_u16(capacity: u32) -> NonNull<Self> {
        unsafe { Self::alloc::<u16>(capacity) }
    }

    const fn get_data_u8(ptr: NonNull<Self>) -> *mut u8 {
        unsafe { ptr.as_ptr().add(1).cast::<u8>() }
    }

    const fn get_data_u16(ptr: NonNull<Self>) -> *mut u16 {
        unsafe { ptr.as_ptr().add(1).cast::<u16>() }
    }

    unsafe fn data_slice_u8_mut<'a>(ptr: NonNull<Self>) -> &'a mut [u8] {
        unsafe {
            let cap = (*ptr.as_ptr()).capacity as usize;

            let data_ptr = Self::get_data_u8(ptr);
            std::slice::from_raw_parts_mut(data_ptr, cap)
        }
    }

    unsafe fn data_slice_u16_mut<'a>(ptr: NonNull<Self>) -> &'a mut [u16] {
        unsafe {
            let cap = (*ptr.as_ptr()).capacity as usize;

            let data_ptr = Self::get_data_u16(ptr);
            std::slice::from_raw_parts_mut(data_ptr, cap)
        }
    }

    unsafe fn data_slice_u8<'a>(ptr: NonNull<Self>) -> &'a [u8] {
        unsafe {
            let cap = (*ptr.as_ptr()).capacity as usize;

            let data_ptr = Self::get_data_u8(ptr);
            std::slice::from_raw_parts(data_ptr, cap)
        }
    }

    unsafe fn data_slice_u16<'a>(ptr: NonNull<Self>) -> &'a [u16] {
        unsafe {
            let cap = (*ptr.as_ptr()).capacity as usize;

            let data_ptr = Self::get_data_u16(ptr);
            std::slice::from_raw_parts(data_ptr, cap)
        }
    }

    unsafe fn data_slice_u8_to<'a>(ptr: NonNull<Self>, to: usize) -> &'a [u8] {
        unsafe {
            let cap = (*ptr.as_ptr()).capacity as usize;
            debug_assert!(to <= cap, "to is greater than capacity");

            let data_ptr = Self::get_data_u8(ptr);
            std::slice::from_raw_parts(data_ptr, to)
        }
    }

    unsafe fn data_slice_u16_to<'a>(ptr: NonNull<Self>, to: usize) -> &'a [u16] {
        unsafe {
            let cap = (*ptr.as_ptr()).capacity as usize;
            debug_assert!(to <= cap, "to is greater than capacity");

            let data_ptr = Self::get_data_u16(ptr);
            std::slice::from_raw_parts(data_ptr, to)
        }
    }

    unsafe fn drop<T>(ptr: NonNull<Self>) {
        let capacity = unsafe { (*ptr.as_ptr()).capacity };

        let layout = Self::layout::<T>(capacity);

        unsafe {
            std::alloc::dealloc(ptr.as_ptr().cast::<u8>(), layout);
        }
    }

    unsafe fn drop_u8(ptr: NonNull<Self>) {
        unsafe { Self::drop::<u8>(ptr) }
    }

    unsafe fn drop_u16(ptr: NonNull<Self>) {
        unsafe { Self::drop::<u16>(ptr) }
    }
}

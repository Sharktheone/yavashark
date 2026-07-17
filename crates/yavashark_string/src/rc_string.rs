use std::alloc::Layout;
use std::marker::PhantomData;
use std::ops::Deref;
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
        let header = Header::alloc_u8((str.len() as u32).saturating_add(extra));
        let mut rc_string = Self {
            header,
            len: str.len() as u32,
            phantom: PhantomData,
        };

        unsafe {
            (*header.as_ptr()).init_to = str.len() as u32;
        }

        unsafe {
            let data_slice = Header::data_slice_u8_mut(header);
            data_slice[..str.len().min(u32::MAX as usize)].copy_from_slice(str.as_bytes());
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

        unsafe {
            let mut data = Header::data_slice_u8_mut(self.header);

            let it = init_to as usize;

            data[it..(it + str.len())].copy_from_slice(str.as_bytes());

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
            (*self.header.as_ptr()).count -= 1;
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
            let data_slice = Header::data_slice_u8(self.header);
            let len = self.len as usize;
            std::str::from_utf8_unchecked(&data_slice[..len])
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
        let header = Header::alloc_u16((str.len() as u32).saturating_add(extra));
        let mut rc_string = Self {
            header,
            len: str.len() as u32,
            phantom: PhantomData,
        };

        unsafe {
            (*header.as_ptr()).init_to = str.len() as u32;
        }

        unsafe {
            let data_slice = Header::data_slice_u16_mut(header);
            data_slice[..str.len().min(u32::MAX as usize)].copy_from_slice(str);
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
            let mut data = Header::data_slice_u16_mut(self.header);

            let it = init_to as usize;

            data[it..(it + str.len())].copy_from_slice(str);

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
            (*self.header.as_ptr()).count -= 1;
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
            let data_slice = Header::data_slice_u16(self.header);

            &data_slice[..(self.len as usize)]
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

    fn get_data_u8(ptr: NonNull<Self>) -> *mut u8 {
        unsafe { ptr.as_ptr().add(1).cast::<u8>() }
    }

    fn get_data_u16(ptr: NonNull<Self>) -> *mut u16 {
        unsafe { ptr.as_ptr().add(1).cast::<u16>() }
    }

    unsafe fn data_slice_u8_mut(ptr: NonNull<Self>) -> &'static mut [u8] {
        unsafe {
            let cap = (*ptr.as_ptr()).capacity as usize;

            let data_ptr = Self::get_data_u8(ptr);
            std::slice::from_raw_parts_mut(data_ptr, cap)
        }
    }

    unsafe fn data_slice_u16_mut(ptr: NonNull<Self>) -> &'static mut [u16] {
        unsafe {
            let cap = (*ptr.as_ptr()).capacity as usize;

            let data_ptr = Self::get_data_u16(ptr);
            std::slice::from_raw_parts_mut(data_ptr, cap)
        }
    }

    unsafe fn data_slice_u8(ptr: NonNull<Self>) -> &'static [u8] {
        unsafe {
            let cap = (*ptr.as_ptr()).capacity as usize;

            let data_ptr = Self::get_data_u8(ptr);
            std::slice::from_raw_parts(data_ptr, cap)
        }
    }

    unsafe fn data_slice_u16(ptr: NonNull<Self>) -> &'static [u16] {
        unsafe {
            let cap = (*ptr.as_ptr()).capacity as usize;

            let data_ptr = Self::get_data_u16(ptr);
            std::slice::from_raw_parts(data_ptr, cap)
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

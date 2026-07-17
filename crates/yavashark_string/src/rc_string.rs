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




impl Header {
    unsafe fn alloc<T>(capacity: u32) -> NonNull<Self> {
        #[allow(clippy::expect_used)]
        let layout = Layout::new::<Self>()
            .extend(Layout::array::<T>(capacity as usize).expect("layout failed"))
            .expect("layout failed")
            .0
            .pad_to_align();


        #[allow(clippy::cast_ptr_alignment)]
        let ptr = unsafe { std::alloc::alloc(layout).cast::<Self>() };

        let Some(ptr) = NonNull::new(ptr) else {
            std::alloc::handle_alloc_error(layout);
        };

        unsafe {
            std::ptr::write(ptr.as_ptr(), Self {
                count: 1,
                capacity,
                init_to: 0,
            });
        }

        ptr
    }

    fn alloc_u8(capacity: u32) -> NonNull<Self> {
        unsafe { Self::alloc::<u8>(capacity) }
    }

    fn alloc_u16(capacity: u32) -> NonNull<Self> {
        unsafe { Self::alloc::<u16>(capacity) }
    }
}
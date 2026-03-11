use std::alloc::Layout;
use std::mem::MaybeUninit;
use std::ptr::NonNull;
use crate::object::inline::{ButterFly, Value};

#[repr(C)]
pub struct PropertyMap<T: ?Sized> {
    _marker: std::marker::PhantomData<T>,
    size: TaggedSize,
    data: [u8],
}

impl<T> PropertyMap<T> {
    pub fn sized_layout(size: u32) -> Layout {
        Self::unsized_layout(size, Layout::new::<T>())
    }

    pub fn get_native_ptr(&self) -> *mut T {
        let properties_size = align(self.size.get() as usize * size_of::<T>(), align_of::<T>());

        unsafe { self.data.as_ptr().add(properties_size) as *mut T }
    }

    pub fn get_native(&self) -> &T {
        unsafe { &*self.get_native_ptr() }
    }

    pub fn get_native_mut(&mut self) -> &mut T {
        unsafe { &mut *self.get_native_ptr() }
    }

    pub unsafe fn initialize(this: *mut Self, size: u32, native: T) {
        (*this).size = TaggedSize::new(size, false);

        (*this).get_uninitialized_properties().fill(MaybeUninit::new(Value::hole()));

        let native_ptr = (*this).get_native_ptr();

        std::ptr::write(native_ptr, native);
    }



}

impl<T: ?Sized> PropertyMap<T> {
    fn get_uninitialized_properties(&mut self) -> &mut [MaybeUninit<Value>] {
        unsafe {
            std::slice::from_raw_parts_mut(self.data.as_mut_ptr() as *mut MaybeUninit<Value>, self.size.get() as usize)
        }
    }

    pub fn get_properties_mut(&mut self) -> &mut [Value] {
        unsafe {
            std::slice::from_raw_parts_mut(self.data.as_mut_ptr() as *mut Value, self.size.get() as usize)
        }
    }

    pub fn get_properties(&self) -> &[Value] {
        unsafe {
            std::slice::from_raw_parts(self.data.as_ptr() as *const Value, self.size.get() as usize)
        }
    }

    pub fn get_butterfly(&self) -> Option<NonNull<ButterFly>> {
        if self.size.is_tagged() {
            let ptr = unsafe {
                self.get_unchecked(self.size.get())
                    .unsafe_assume_pointer()
                    .cast::<ButterFly>()
            };

            Some(ptr)
        } else {
            None
        }
    }

    pub fn unsized_layout(size: u32, native: Layout) -> std::alloc::Layout {
        let properties = Layout::array::<Value>(size as usize)
            .expect("Invalid layout for property map");

        properties.extend(native)
            .expect("Invalid layout for property map")
            .0
    }



    pub unsafe fn initialize_unsized(this: *mut Self, size: u32) {
        (*this).size.set_size(size);
        //TODO
    }

    pub fn get(&self, index: u32) -> Option<&Value> {
        if index < self.size.get() {
            Some(self.get_unchecked(index))
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, index: u32) -> Option<&mut Value> {
        if index < self.size.get() {
            Some(self.get_unchecked_mut(index))
        } else {
            None
        }
    }
    
    pub fn get_unchecked(&self, index: u32) -> &Value {
        unsafe {
            self.get_ptr(index).as_ref()
        }
    }

    pub fn get_unchecked_mut(&mut self, index: u32) -> &mut Value {
        unsafe {
            self.get_ptr(index).as_mut()
        }
    }

    unsafe fn get_ptr(&self, index: u32) -> NonNull<Value> {
        let ptr  = self.data.as_ptr().add(index as usize * size_of::<Value>()) as *mut Value;

        NonNull::new_unchecked(ptr)
    }
}


// Uses the most significant bit as a tag
struct TaggedSize(u32);


impl TaggedSize {
    const MAX_SIZE: u32 = 0x7FFF_FFFF; // 31 bits for size

    fn new(size: u32, tag: bool) -> Self {
        assert!(size <= Self::MAX_SIZE, "Size exceeds maximum allowed");
        let tagged = if tag { size | 0x8000_0000 } else { size };
        TaggedSize(tagged)
    }

    fn get(&self) -> u32 {
        self.0 & Self::MAX_SIZE
    }

    fn is_tagged(&self) -> bool {
        (self.0 & 0x8000_0000) != 0
    }

    fn set_tag(&mut self, tag: bool) {
        if tag {
            self.0 |= 0x8000_0000;
        } else {
            self.0 &= 0x7FFF_FFFF;
        }
    }

    fn set_size(&mut self, size: u32) {
        assert!(size <= Self::MAX_SIZE, "Size exceeds maximum allowed");
        self.0 = (self.0 & 0x8000_0000) | size;
    }
}

fn align(size: usize, align: usize) -> usize {
    (size + align - 1) & !(align - 1)
}


use crate::object::inline::{ButterFly, Value};
use std::alloc::Layout;
use std::mem::MaybeUninit;
use std::ptr::NonNull;

#[repr(C)]
pub struct PropertyMap<T: ?Sized> {
    _marker: std::marker::PhantomData<T>,
    state: MapState,
    data: [u8],
}

struct MapState {
    size: u32,
    extensible: bool,
    has_butterfly: bool,
}

impl<T> PropertyMap<T> {
    pub fn sized_layout(size: u32) -> Layout {
        Self::unsized_layout(size, Layout::new::<T>())
    }

    pub fn get_native_ptr(&self) -> *mut T {
        let properties_size = align(self.state.size as usize * size_of::<T>(), align_of::<T>());

        unsafe { self.data.as_ptr().add(properties_size) as *mut T }
    }

    pub fn get_native(&self) -> &T {
        unsafe { &*self.get_native_ptr() }
    }

    pub fn get_native_mut(&mut self) -> &mut T {
        unsafe { &mut *self.get_native_ptr() }
    }

    pub unsafe fn initialize(this: *mut Self, size: u32, native: T) {
        let state = MapState {
            size,
            extensible: true,
            has_butterfly: false,
        };

        (*this).state = state;

        (*this)
            .get_uninitialized_properties()
            .fill(MaybeUninit::new(Value::hole()));

        let native_ptr = (*this).get_native_ptr();

        std::ptr::write(native_ptr, native);
    }
}

impl<T: ?Sized> PropertyMap<T> {
    fn get_uninitialized_properties(&mut self) -> &mut [MaybeUninit<Value>] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.data.as_mut_ptr() as *mut MaybeUninit<Value>,
                self.state.size as usize,
            )
        }
    }

    pub fn get_properties_mut(&mut self) -> &mut [Value] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.data.as_mut_ptr() as *mut Value,
                self.state.size as usize,
            )
        }
    }

    pub fn get_properties(&self) -> &[Value] {
        unsafe {
            std::slice::from_raw_parts(self.data.as_ptr() as *const Value, self.state.size as usize)
        }
    }

    pub fn get_butterfly(&self) -> Option<NonNull<ButterFly>> {
        if self.state.has_butterfly {
            let ptr = unsafe {
                self.get_unchecked(self.state.size)
                    .unsafe_assume_pointer()
                    .cast::<ButterFly>()
            };

            Some(ptr)
        } else {
            None
        }
    }

    pub fn unsized_layout(size: u32, native: Layout) -> std::alloc::Layout {
        let properties =
            Layout::array::<Value>(size as usize).expect("Invalid layout for property map");

        properties
            .extend(native)
            .expect("Invalid layout for property map")
            .0
    }

    pub unsafe fn initialize_unsized(this: *mut Self, size: u32) {
        let state = MapState {
            size,
            extensible: true,
            has_butterfly: false,
        };

        (*this).state = state;
    }

    pub fn get(&self, index: u32) -> Option<&Value> {
        if index < self.state.size {
            Some(self.get_unchecked(index))
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, index: u32) -> Option<&mut Value> {
        if index < self.state.size {
            Some(self.get_unchecked_mut(index))
        } else {
            None
        }
    }

    pub fn get_unchecked(&self, index: u32) -> &Value {
        unsafe { self.get_ptr(index).as_ref() }
    }

    pub fn get_unchecked_mut(&mut self, index: u32) -> &mut Value {
        unsafe { self.get_ptr(index).as_mut() }
    }

    unsafe fn get_ptr(&self, index: u32) -> NonNull<Value> {
        let ptr = self.data.as_ptr().add(index as usize * size_of::<Value>()) as *mut Value;

        NonNull::new_unchecked(ptr)
    }
}

const fn align(size: usize, align: usize) -> usize {
    (size + align - 1) & !(align - 1)
}

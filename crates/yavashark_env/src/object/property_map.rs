#![allow(dead_code)]
use crate::object::inline::{ButterFly, Value};
use std::alloc::Layout;
use std::mem::MaybeUninit;
use std::ptr::NonNull;

#[repr(C)]
#[repr(align(8))]
pub struct PropertyMap<T: ?Sized> {
    _marker: std::marker::PhantomData<T>,
    state: MapState,
    data: OpaqueData,
}

#[repr(align(8))]
pub struct OpaqueData;

#[repr(align(8))]
pub struct Data([u8]);

#[repr(align(8))]
struct MapState {
    size: u32,
    extensible: bool,
    has_butterfly: bool,
    _pad: [u8; 2],
}

impl<T> PropertyMap<T> {
    pub fn sized_layout(size: u32) -> Layout {
        Self::unsized_layout(size, Layout::new::<T>())
    }

    pub const fn size_from_alloc_bytes(bytes: usize) -> u32 {
        let header = size_of::<Self>();

        let native_size = size_of::<T>();
        let native_align = align_of::<T>();

        0
    }

    pub const fn get_native_ptr(&self) -> NonNull<T> {
        let properties_size = align(self.state.size as usize * size_of::<T>(), align_of::<T>());

        let ptr = NonNull::from_ref(&self.data).cast::<u8>();

        unsafe { ptr.byte_add(properties_size).cast::<T>() }
    }

    pub const fn get_native(&self) -> &T {
        unsafe { self.get_native_ptr().as_ref() }
    }

    pub const fn get_native_mut(&mut self) -> &mut T {
        unsafe { self.get_native_ptr().as_mut() }
    }

    pub unsafe fn initialize(this: *mut MaybeUninit<Self>, size: u32, native: T) {
        let this = (*this).as_mut_ptr();
        let state = MapState {
            size,
            extensible: true,
            has_butterfly: false,
            _pad: [0; _],
        };

        (*this).state = state;

        (*this)
            .get_uninitialized_properties()
            .fill(MaybeUninit::new(Value::hole()));

        let native_ptr = (*this).get_native_ptr();

        std::ptr::write(native_ptr.as_ptr(), native);
    }

    pub unsafe fn initialize_native_cb(
        this: NonNull<Self>,
        size: u32,
        native: impl FnOnce(NonNull<T>),
    ) {
        let state = MapState {
            size,
            extensible: true,
            has_butterfly: false,
            _pad: [0; _],
        };

        (*this.as_ptr()).state = state;

        (*this.as_ptr())
            .get_uninitialized_properties()
            .fill(MaybeUninit::new(Value::hole()));

        let native_ptr = (*this.as_ptr()).get_native_ptr();

        native(native_ptr);
    }
}

impl<T: ?Sized> PropertyMap<T> {
    const fn get_uninitialized_properties(&mut self) -> &mut [MaybeUninit<Value>] {
        unsafe {
            std::slice::from_raw_parts_mut(
                (&raw mut self.data).cast::<MaybeUninit<Value>>(),
                self.state.size as usize,
            )
        }
    }

    pub const fn get_properties_mut(&mut self) -> &mut [Value] {
        unsafe {
            std::slice::from_raw_parts_mut(
                (&raw mut self.data).cast::<Value>(),
                self.state.size as usize,
            )
        }
    }

    pub const fn get_properties(&self) -> &[Value] {
        unsafe {
            std::slice::from_raw_parts(
                (&raw const self.data).cast::<Value>(),
                self.state.size as usize,
            )
        }
    }

    pub const fn get_butterfly(&self) -> Option<NonNull<ButterFly>> {
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

    pub fn unsized_layout(size: u32, native: Layout) -> Layout {
        let layout = Layout::new::<Self>();

        layout
            .extend(Layout::array::<Value>(size as usize).expect("Invalid layout for property map"))
            .expect("Invalid layout for property map")
            .0
            .extend(native)
            .expect("Invalid layout for property map")
            .0
    }

    pub const unsafe fn initialize_unsized(this: *mut Self, size: u32) {
        let state = MapState {
            size,
            extensible: true,
            has_butterfly: false,
            _pad: [0; _],
        };

        (*this).state = state;
    }

    pub const fn get(&self, index: u32) -> Option<&Value> {
        if index < self.state.size {
            Some(self.get_unchecked(index))
        } else {
            None
        }
    }

    pub const fn get_mut(&mut self, index: u32) -> Option<&mut Value> {
        if index < self.state.size {
            Some(self.get_unchecked_mut(index))
        } else {
            None
        }
    }

    pub const fn get_unchecked(&self, index: u32) -> &Value {
        unsafe { self.get_ptr(index).as_ref() }
    }

    pub const fn get_unchecked_mut(&mut self, index: u32) -> &mut Value {
        unsafe { self.get_ptr(index).as_mut() }
    }

    const unsafe fn get_ptr(&self, index: u32) -> NonNull<Value> {
        NonNull::from_ref(&self.data)
            .byte_add(index as usize * size_of::<Value>())
            .cast()
    }
}

const fn align(size: usize, align: usize) -> usize {
    (size + align - 1) & !(align - 1)
}

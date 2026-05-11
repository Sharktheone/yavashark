#![allow(unused)]
use crate::object::inline::{ButterFly, Value};
use std::alloc::Layout;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ptr::NonNull;
use crate::object::native_wrapper::NativeWrapper;



/// A property map is a contiguous block of memory that holds a fixed number of properties (as Values) and a native struct of type T.
/// Internally, the `PropertyMap` is broken up into N 64-bit slots for different purposes.
/// The layout of the property map is as follows:
/// - Slot 0: `MapState` (8 bytes)
/// - Slots 1..N: Properties (each 8 bytes, total size determined by `MapState.size`)
/// - Slot N+1: Butterfly pointer
/// - Slot N+2..N+2+M: Native struct of type T (size and alignment determined by T)
///
/// As a diagram this looks as follows:
/// [ MapState | Property 0 | Property 1 | ... | Property N-1 | Butterfly Pointer (if present) | Native struct (T) ]
#[repr(C)]
#[repr(align(8))]
pub struct PropertyMap<T: ?Sized> {
    state: MapState,
    data: OpaqueData<T>,
}

#[repr(C, align(8))]
#[derive(Default)]
pub struct OpaqueData<T: ?Sized> {
    _value: PhantomData<[Value]>,
    _butterfly: PhantomData<*mut ButterFly>,
    _inner: PhantomData<NativeWrapper<T>>,
}


#[repr(align(8))]
pub struct Data<T: ?Sized> {
    _value: PhantomData<[Value]>,
    _butterfly: PhantomData<*mut ButterFly>,
    _inner: PhantomData<NativeWrapper<T>>,
    inner: [u8], // layout is the same as the two markers.
}



#[repr(align(8))]
struct MapState {
    size: u32,
    extensible: bool,
    has_butterfly: bool,
    _pad: [u8; 2],
}

const _: () = assert!(size_of::<MapState>() == 8, "MapState must be 8 bytes in size");
const _: () = assert!(align_of::<MapState>() == 8, "MapState must be 8 bytes aligned");

impl<T: ?Sized> PropertyMap<T> {
    const NUM_INTERNAL_SLOTS: usize = 1; // MapState takes one slot.
    const NUM_BUTTERFLY_SLOTS: usize = 1; // Butterfly pointer takes one slot.
}

impl<T> PropertyMap<T> {
    pub fn sized_layout(size: u32) -> Layout {
        Self::layout(size, Layout::new::<NativeWrapper<T>>(), false)
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

    pub unsafe fn initialize(this: NonNull<MaybeUninit<Self>>, size: u32, native: T) {
        let this = unsafe {
            (*this.as_ptr()).as_mut_ptr()
        };
        let state = MapState {
            size,
            extensible: true,
            has_butterfly: false,
            _pad: [0; _],
        };

        unsafe {
            (*this).state = state;

            (*this)
                .get_uninitialized_properties()
                .fill(MaybeUninit::new(Value::hole()));

            let native_ptr = (*this).get_native_ptr();

            std::ptr::write(native_ptr.as_ptr(), native);
        }
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

        let native_ptr = unsafe {
            (*this.as_ptr()).state = state;

            (*this.as_ptr())
                .get_uninitialized_properties()
                .fill(MaybeUninit::new(Value::hole()));

            (*this.as_ptr()).get_native_ptr()
        };

        native(native_ptr);

    }

    pub fn from_alloc_size(this: NonNull<MaybeUninit<Self>>, capacity: usize, native_size: usize, butterfly: bool) -> NonNull<MaybeUninit<Self>> {
        let slots = capacity / 8;

        let native_slots = native_size.div_ceil(8);
        let num_slots = slots - Self::NUM_INTERNAL_SLOTS -
            if butterfly { Self::NUM_BUTTERFLY_SLOTS } else { 0 } - native_slots;


        unsafe {
            (*(*this.as_ptr()).as_mut_ptr()).state = MapState {
                size: num_slots as u32,
                extensible: true,
                has_butterfly: butterfly,
                _pad: [0; _],
            };

            (*(*this.as_ptr()).as_mut_ptr())
                .get_uninitialized_properties()
                .fill(MaybeUninit::new(Value::hole()));
        }


        this
    }

    pub fn from_alloc_size_native_cb(
        this: NonNull<MaybeUninit<Self>>,
        capacity: usize,
        native_size: usize,
        butterfly: bool,
        native: impl FnOnce(NonNull<MaybeUninit<T>>),
    ) -> NonNull<Self> {
        let mut this = Self::from_alloc_size(this, capacity, native_size, butterfly);

        let native_ptr = unsafe {
            (*(*this.as_ptr()).as_mut_ptr()).get_native_ptr()
                .cast::<MaybeUninit<T>>()
        };

        native(native_ptr);

        unsafe {
            NonNull::from(this.as_mut().assume_init_mut())
        }
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

    pub fn layout(size: u32, native: Layout, butterfly: bool) -> Layout {
        let num_slots = size as usize + Self::NUM_INTERNAL_SLOTS +
            if butterfly { Self::NUM_BUTTERFLY_SLOTS } else { 0 };

        let native_slots = native.size().div_ceil(8);

        let total_slots = num_slots + native_slots;

        Layout::from_size_align(
            total_slots * 8,
            8,
        ).expect("Invalid layout for PropertyMap")
    }

    pub const unsafe fn initialize_unsized(this: *mut Self, size: u32) {
        let state = MapState {
            size,
            extensible: true,
            has_butterfly: false,
            _pad: [0; _],
        };

        unsafe { (*this).state = state; }
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
        unsafe {
            NonNull::from_ref(&self.data)
                .byte_add(index as usize * size_of::<Value>())
                .cast()
        }
    }
}

const fn align(size: usize, align: usize) -> usize {
    (size + align - 1) & !(align - 1)
}

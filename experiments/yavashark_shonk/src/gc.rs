use std::ops::Deref;
use std::ptr::NonNull;

trait Tracer {
    fn trace<T: Trace>(&mut self, value: &T);
}

trait GCable: Trace {
    type Type;

    fn move_to(self, location: NonNull<()>) {
        unsafe {
            let ptr = NonNull::from(&self).cast::<Self::Type>();
            std::ptr::copy_nonoverlapping(ptr.as_ptr(), location.as_ptr() as *mut Self::Type, 1);
        }
    }
}


trait Trace {
    fn trace(&self, gc: &mut Tracer);
}



type Value = u64;
struct Butterfly {
    phantom: std::marker::PhantomData<[Value]>,
}


struct Shape {
    extensible: bool,
    has_butterfly: bool,
    size: u32,
}

struct Object {
    phantom: std::marker::PhantomData<(*mut Butterfly, [Value])>,
}


impl GCable for Object {
    type Type = Shape;
}

impl Trace for Object {
    fn trace(&self, gc: &mut Tracer) {
        // Implement tracing logic for Object
    }
}

enum Color {
    White = 0,
    Gray = 1,
    Black = 2,
}

#[repr(C)]
struct GcData<T: GCable> {
    data: T::Type,
    collect: fn(NonNull<()>),
    drop: fn(NonNull<()>),
}

#[repr(C)]
struct GcBox<T: GCable> {
    data: NonNull<GcData<T>>,
    value: T,
}

struct Gc<T: GCable> {
    ptr: NonNull<GcBox<T>>,
}

impl<T: GCable> Deref for Gc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: GCable> Gc<T> {
    fn new(data: T::Type) -> Self {
        let boxed = Box::new(data);
        let ptr = NonNull::from(Box::leak(boxed));
        Gc { ptr }


    }

    fn get_data(gc: Self) -> &T::Type {
        unsafe { self }
    }

}
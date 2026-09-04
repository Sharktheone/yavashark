use std::alloc;
use std::alloc::Layout;
use std::ptr::NonNull;

fn alloca_type<T>(f: impl FnOnce(&mut T) -> T) -> T {
    let layout = Layout::new::<T>();

    alloca(layout, |ptr| {
        let value = unsafe { ptr.cast::<T>().as_mut() };
        f(value)
    })
}

fn alloca_array<T>(len: usize, f: impl FnOnce(&mut [T]) -> T) -> T {
    let layout = Layout::array::<T>(len).expect("Failed to create layout for array");

    alloca(layout, |ptr| {
        let slice = unsafe { std::slice::from_raw_parts_mut(ptr.cast::<T>().as_ptr(), len) };
        f(slice)
    })
}


fn alloca<T>(layout: Layout, f: impl FnOnce(NonNull<()>) -> T) -> T {

    // we just mock alloca for now, the real one will be written in C
    let mem = unsafe {
        alloc::alloc(layout)
    };

    let Some(ptr) = NonNull::new(mem) else {
        panic!("alloca failed");
    };

    let res = f(ptr.cast());

    unsafe {
        alloc::dealloc(mem, layout);
    }

    res
}

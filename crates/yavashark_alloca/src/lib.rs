use std::alloc::{self, Layout};
use std::mem::MaybeUninit;
use std::ptr::NonNull;


const MAX_STACK_BYTES: usize = 16 * 1024;


pub enum AllocaError {
    Layout,
    TooLarge
}

pub fn alloca_array<T, R>(len: usize, f: impl FnOnce(&mut [MaybeUninit<T>]) -> R) -> Result<R, AllocaError> {
    let Ok(layout) = Layout::array::<T>(len) else {
        return Err(AllocaError::Layout)
    };

    alloca(layout, |ptr| {
        let slice = unsafe { std::slice::from_raw_parts_mut(ptr.cast().as_ptr(), len) };
        f(slice)
    })
}


pub fn alloca<R>(layout: Layout, f: impl FnOnce(NonNull<()>) -> R) -> Result<R, AllocaError> {
    if layout.size() == 0 {
        let ptr = std::ptr::without_provenance_mut::<()>(layout.align());
        return Ok(f(NonNull::new(ptr).ok_or(AllocaError::Layout)?));
    }

    #[cfg(has_c_alloca)]
    {

        let size = layout.size().checked_add(layout.align() - 1).ok_or(AllocaError::TooLarge)?;

        if size <= MAX_STACK_BYTES {
            Ok(stack::alloca(size, layout.align(), f))
        } else {
            Err(AllocaError::TooLarge)
        }

    }

    #[cfg(not(has_c_alloca))] {
        crate::heap_alloca(layout, f)
    }
}

fn heap_alloca<R>(layout: Layout, f: impl FnOnce(NonNull<()>) -> R) -> Result<R, AllocaError> {
    struct Allocation {
        ptr: NonNull<u8>,
        layout: Layout,
    }
    impl Drop for Allocation {
        fn drop(&mut self) {
            // SAFETY: This guard owns the allocation with its original layout.
            unsafe { alloc::dealloc(self.ptr.as_ptr(), self.layout) };
        }
    }

    // SAFETY: alloca handles zero-sized layouts before reaching this backend.
    let ptr = unsafe { alloc::alloc(layout) };

    let ptr = NonNull::new(ptr).ok_or(AllocaError::Layout)?;

    let allocation = Allocation { ptr, layout };

    Ok(f(allocation.ptr.cast()))
}

#[cfg(has_c_alloca)]
mod stack {
    use std::ffi::c_void;
    use std::mem::{ManuallyDrop, MaybeUninit};
    use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
    use std::ptr::NonNull;

    unsafe extern "C-unwind" {
        fn yavashark_with_alloca(
            size: usize,
            callback: unsafe extern "C-unwind" fn(*mut c_void, *mut c_void),
            context: *mut c_void,
        );
    }

    struct Context<F, R> {
        callback: ManuallyDrop<F>,
        result: MaybeUninit<std::thread::Result<R>>,
        align: usize,
    }

    unsafe extern "C-unwind" fn invoke<F, R>(buffer: *mut c_void, context: *mut c_void)
    where
        F: FnOnce(NonNull<()>) -> R,
    {
        // SAFETY: The synchronous C helper forwards our live Context unchanged.
        let context = unsafe { &mut *context.cast::<Context<F, R>>() };
        context.result = MaybeUninit::new(catch_unwind(AssertUnwindSafe(|| {
            let callback = unsafe {
                ManuallyDrop::take(&mut context.callback)
            };

            let ptr = buffer.cast::<u8>();
            let offset = ptr.align_offset(context.align);


            // SAFETY: The allocation includes align - 1 bytes of padding.
            let ptr = unsafe { ptr.add(offset) };


            let ptr = NonNull::new(ptr)
                .expect("C alloca returned null");


            callback(ptr.cast())
        })));
    }

    pub fn alloca<F, R>(size: usize, align: usize, callback: F) -> R
    where
        F: FnOnce(NonNull<()>) -> R,
    {
        let mut context = Context {
            callback: ManuallyDrop::new(callback),
            result: MaybeUninit::uninit(),
            align,
        };
        // SAFETY: size is positive and bounded. The helper calls invoke exactly
        // once while its buffer and this context are live. invoke catches panics.
        unsafe {
            yavashark_with_alloca(
                size,
                invoke::<F, R>,
                std::ptr::from_mut(&mut context).cast(),
            );
        }
        match unsafe { context.result.assume_init() } {
            Ok(result) => result,
            Err(payload) => resume_unwind(payload),
        }
    }
}
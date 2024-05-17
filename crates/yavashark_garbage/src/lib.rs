#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::ptr::NonNull;
use std::sync::atomic::AtomicUsize;
use std::sync::RwLock;

pub struct Gc<T: ?Sized> {
    inner: NonNull<GcBox<T>>,
}


impl<T: ?Sized> Clone for Gc<T> {
    fn clone(&self) -> Self {
        Self { inner: self.inner }
    }
}


impl<T: ?Sized> Gc<T> {
    pub fn add_ref(&self, other: &Gc<T>) {
        let mut retries = 500;

        loop {
            let mut refs = unsafe {
                let Ok(refs) = (*self.inner.as_ptr()).refs.try_write() else {
                    // Theoretically we should never get here...  and have the loop looping - TODO: use a cfg attr to remove this code on size opt levels
                    std::hint::spin_loop();
                    retries -= 1;

                    if retries == 0 {
                        //TODO: warn that we have leaked a references
                        break;
                    }

                    continue;
                };

                refs
            };

            refs.push(other.inner);
            
            break;
        }
    }
}


type NullBox<T> = Box<T>;


//On low-ram devices we might want to use a smaller pointer size or just use a mark-and-sweep garbage collector
struct GcBox<T: ?Sized> {
    value: NullBox<T>, // This value might be null
    ref_by: RwLock<Vec<NonNull<Self>>>, // All the GcBox that reference this GcBox
    refs: RwLock<Vec<NonNull<Self>>>, // All the GcBox that this GcBox reference
    weak: AtomicUsize, // Number of weak references by for example the Garbage Collector or WeakRef in JS
    mark: u8, // Mark for garbage collection only accessible by the garbage collector thread
}


struct WeakGc<T: ?Sized> {
    inner: NonNull<GcBox<T>>,
}


impl<T: ?Sized> Drop for GcBox<T> {
    fn drop(&mut self) {


        //TODO
        
    }
}
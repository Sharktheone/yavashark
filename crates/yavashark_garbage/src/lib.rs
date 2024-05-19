#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::RwLock;

use log::warn;
use rand::random;

use spin_lock::SpinLock;

pub(crate) mod spin_lock;


pub struct Gc<T: ?Sized> {
    inner: NonNull<GcBox<T>>,
}


impl<T: ?Sized> Clone for Gc<T> {
    fn clone(&self) -> Self {
        Self { inner: self.inner }
    }
}


impl<T: ?Sized> Gc<T> {
    pub fn add_ref(&self, other: &Self) {
        unsafe {
            let Some(mut lock) = (*self.inner.as_ptr()).refs.spin_write()  else {
                warn!("Failed to add reference to a GcBox");
                return;
            };

            lock.push(other.inner);
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
        self.mark = 0;
        while self.mark == 0 {
            self.mark = random();
        }

        if let Some(ref_by) = self.ref_by.spin_read() {
            assert!(ref_by.is_empty(), "Cannot drop a GcBox that is still referenced");
        } else {
            warn!("Failed to proof that all references to a GcBox have been dropped - this might be bad"); //TODO: should we also panic here?
        }
        
        if self.weak.load(Ordering::Relaxed) != 0 {
            warn!("Dropping a GcBox that still has weak references - this might be bad");
        }

        if let Some(refs) = self.refs.spin_read() {
            for r in &*refs {
                unsafe {
                    let Some(mut lock) = (*r.as_ptr()).ref_by.spin_write() else {
                        warn!("Failed to remove reference from a GcBox - leaking memory");
                        continue;
                    };
                    lock.retain(|x| (*x.as_ptr()).mark != self.mark);
                }
            }
        } else {
            warn!("Failed to remove all references from a GcBox - leaking memory");
        }
    }
}

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::ptr;
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
            (*self.inner.as_ptr()).strong.fetch_add(1, Ordering::Relaxed);
            let Some(mut lock) = (*self.inner.as_ptr()).refs.spin_write()  else {
                warn!("Failed to add reference to a GcBox");
                return;
            };

            lock.push(other.inner);
        }
    }

    pub fn remove_ref(&self, other: &Self) {
        unsafe {
            (*self.inner.as_ptr()).strong.fetch_sub(1, Ordering::Relaxed);
            let Some(mut lock) = (*self.inner.as_ptr()).refs.spin_write()  else {
                warn!("Failed to remove reference from a GcBox");
                return;
            };

            lock.retain(|x| x != &other.inner);
        }
    }
}


type MaybeNull<T> = NonNull<T>;


//On low-ram devices we might want to use a smaller pointer size or just use a mark-and-sweep garbage collector
struct GcBox<T: ?Sized> {
    value: MaybeNull<T>, // This value might be null
    ref_by: RwLock<Vec<NonNull<Self>>>, // All the GcBox that reference this GcBox
    refs: RwLock<Vec<NonNull<Self>>>, // All the GcBox that this GcBox reference
    weak: AtomicUsize, // Number of weak references by for example the Garbage Collector or WeakRef in JS
    strong: AtomicUsize, // Number of strong references
    mark: u8, // Mark for garbage collection only accessible by the garbage collector thread
}


struct WeakGc<T: ?Sized> {
    inner: NonNull<GcBox<T>>,
}


impl<T: ?Sized> GcBox<T> {
    const MARKED: u8 = 0b1000_0000;
    const HAS_ROOT: u8 = 0b0100_0000;
    const HAS_NO_ROOT: u8 = 0b0010_0000;

    fn walk_graph(&mut self) {
        self.mark();
        let Some(read) = self.refs.spin_read() else {
            warn!("Failed to read references from a GcBox - leaking memory");
            return;
        };

        let mut unmark = Vec::new();


        for r in &*read {
            unsafe {
                if (*r.as_ptr()).mark != Self::MARKED && (*r.as_ptr()).walk_from_dead(&mut unmark) {
                    unmark.push(*r);
                }
            }
        }
        
        for r in unmark {
            unsafe {
                (*r.as_ptr()).unmark();
            }
        }
    }

    /// Returns true if the `GcBox` needs to be unmarked later on
    fn walk_from_dead(&self, unmark: &mut Vec<NonNull<Self>>) -> bool {
        todo!("Implement walk_from_dead")
    }

    fn mark(&mut self) {
        self.mark = Self::MARKED;
    }

    fn unmark(&mut self) {
        self.mark = 0;
    }

    fn nuke(this: *mut Self) {
        unsafe {
            let _ = Box::from_raw((*this).value.as_ptr());
        }
    }

    fn you_have_root(&mut self) -> bool {
        if self.mark & Self::HAS_NO_ROOT != 0 {
            return false; // We already know that we don't have a root
        }


        if self.mark & Self::HAS_ROOT == 0 {
            self.mark |= Self::HAS_ROOT;
            let Some(refs) = self.refs.spin_read() else {
                warn!("Failed to read references from a GcBox - leaking memory");
                return true; // we say that we have a root, so we leak memory and don't drop memory that we might still need
            };

            for r in &*refs {
                unsafe {
                    let root = (*r.as_ptr()).you_have_root();
                    if root {
                        return true; // We have a root
                    }
                }
            }

            return false; // We don't have a root
        }

        true // We have a root
    }
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


impl<T: ?Sized> Drop for Gc<T> {
    fn drop(&mut self) {
        unsafe {
            if (*self.inner.as_ptr()).strong.fetch_sub(1, Ordering::Relaxed) == 0 {
                //we can drop the GcBox's value
                let _ = Box::from_raw(self.inner.as_ptr());

                if (*self.inner.as_ptr()).weak.load(Ordering::Relaxed) == 0 {
                    //we can drop the complete GcBox
                    let _ = Box::from_raw(self.inner.as_ptr());
                }
            }

            (*self.inner.as_ptr()).walk_graph();
        }
    }
}
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::RwLock;

use bitflags::bitflags;
use log::warn;
use rand::random;

use spin_lock::SpinLock;

pub(crate) mod spin_lock;

pub struct Gc<T: ?Sized> {
    inner: NonNull<GcBox<T>>,
}

impl<T: ?Sized> Clone for Gc<T> {
    fn clone(&self) -> Self {
        unsafe {
            (*self.inner.as_ptr())
                .strong
                .fetch_add(1, Ordering::Relaxed);
        }

        Self { inner: self.inner }
    }
}

impl<T: ?Sized> Deref for Gc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(*self.inner.as_ptr()).value.as_ptr() }
    }
}

impl<T: ?Sized> Gc<T> {
    pub fn add_ref(&self, other: &Self) {
        unsafe {
            let Some(mut lock) = (*self.inner.as_ptr()).refs.spin_write() else {
                warn!("Failed to add reference to a GcBox");
                return;
            };

            lock.push(other.inner);
        }
    }

    fn add_ref_by(&self, other: &Self) {
        unsafe {
            let Some(mut lock) = (*self.inner.as_ptr()).ref_by.spin_write() else {
                warn!("Failed to add reference to a GcBox");
                return;
            };

            lock.push(other.inner);
        }
    }

    pub fn remove_ref(&self, other: &Self) {
        unsafe {
            let Some(mut lock) = (*self.inner.as_ptr()).refs.spin_write() else {
                warn!("Failed to remove reference from a GcBox");
                return;
            };

            lock.retain(|x| x != &other.inner);
        }
    }

    fn remove_ref_by(&self, other: &Self) {
        unsafe {
            let Some(mut lock) = (*self.inner.as_ptr()).ref_by.spin_write() else {
                warn!("Failed to remove reference from a GcBox");
                return;
            };

            lock.retain(|x| x != &other.inner);
        }
    }
}

impl<T> Gc<T> {
    pub fn new(value: T) -> Self {
        let value = Box::new(value);
        let value = unsafe { NonNull::new_unchecked(Box::into_raw(value)) };

        let gc_box = GcBox {
            value,
            ref_by: RwLock::new(Vec::new()),
            refs: RwLock::new(Vec::new()),
            weak: AtomicUsize::new(0),
            strong: AtomicUsize::new(1),
            flags: Flags::new(),
        };

        let gc_box = Box::new(gc_box);
        let gc_box = unsafe { NonNull::new_unchecked(Box::into_raw(gc_box)) };

        Self { inner: gc_box }
    }
}

type MaybeNull<T> = NonNull<T>;

//On low-ram devices we might want to use a smaller pointer size or just use a mark-and-sweep garbage collector
struct GcBox<T: ?Sized> {
    value: MaybeNull<T>,                // This value might be null
    ref_by: RwLock<Vec<NonNull<Self>>>, // All the GcBox that reference this GcBox
    refs: RwLock<Vec<NonNull<Self>>>,   // All the GcBox that this GcBox reference
    weak: AtomicUsize, // Number of weak references by for example the Garbage Collector or WeakRef in JS
    strong: AtomicUsize, // Number of strong references
    flags: Flags, // Mark for garbage collection only accessible by the garbage collector thread
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Flags(u8);

impl Flags {
    const MARKED: u8 = 0b0000_0001;
    const NONE_ROOT: u8 = 0b0000_0000;
    const HAS_ROOT: u8 = 0b0000_0010;
    const HAS_NO_ROOT: u8 = 0b0000_0100;
    const ROOT_PENDING: u8 = 0b0000_0110;

    fn new() -> Self {
        Self(0)
    }

    fn set_marked(&mut self) {
        self.0 |= Self::MARKED;
    }

    fn set_none_root(&mut self) {
        self.0 &= !Self::HAS_ROOT;
        self.0 &= !Self::HAS_NO_ROOT;
    }

    fn set_has_root(&mut self) {
        self.0 |= Self::HAS_ROOT;
    }

    fn set_has_no_root(&mut self) {
        self.0 |= Self::HAS_NO_ROOT;
    }

    fn set_root_pending(&mut self) {
        self.0 |= Self::ROOT_PENDING;
    }

    const fn is_marked(&self) -> bool {
        self.0 & Self::MARKED != 0
    }

    const fn is_none_root(&self) -> bool {
        self.0 & Self::HAS_ROOT == 0 && self.0 & Self::HAS_NO_ROOT == 0
    }

    const fn is_has_root(&self) -> bool {
        self.0 & Self::HAS_ROOT != 0 && self.0 & Self::HAS_NO_ROOT == 0
    }

    const fn is_has_no_root(&self) -> bool {
        self.0 & Self::HAS_NO_ROOT != 0 && self.0 & Self::HAS_ROOT == 0
    }

    const fn is_root_pending(&self) -> bool {
        self.0 & Self::ROOT_PENDING != 0
    }

    ///Sets the unused bits to a random value (highest 4)
    fn random(&mut self) {
        self.0 = (self.0 & 0b0000_1111) | (random::<u8>() << 4);
    }

    fn reset_random(&mut self) {
        self.0 &= 0b0000_1111;
    }

    fn reset(&mut self) {
        self.0 = 0;
    }
}

#[derive(Debug, PartialEq, Eq)]
enum RootStatus {
    None,
    HasRoot,
    HasNoRoot,
    RootPending,
}

pub struct WeakGc<T: ?Sized> {
    inner: NonNull<GcBox<T>>,
}

impl<T: ?Sized> GcBox<T> {
    fn shake_tree(this_ptr: NonNull<Self>) {
        let mut unmark = Vec::new();
        unsafe {
            let status = Self::you_have_root(this_ptr, &mut unmark);

            match status {
                RootStatus::HasRoot => {
                    for r in unmark {
                        (*r.as_ptr()).unmark();
                    }

                    return;
                }

                RootStatus::HasNoRoot => {}
                RootStatus::RootPending => {
                    (*this_ptr.as_ptr()).flags.set_has_no_root();
                    Self::mark_dead(this_ptr, None);
                }

                RootStatus::None => {
                    warn!("Failed to find root status for a GcBox");
                    return;
                }
            }

            let this = this_ptr.as_ptr();

            //TODO: we externally need to destruct the GcBoxes that are marked as dead
            //externally because we don't know what other GcBoxes it might reference, that are also dead and so might already be nuked
        }
    }

    fn mark_dead(this_ptr: NonNull<Self>, look_later: Option<&mut Vec<NonNull<Self>>>) {
        let this = this_ptr.as_ptr();

        unsafe {
            let Some(read) = (*this).refs.spin_read() else {
                warn!("Failed to read references from a GcBox - maybe leaking memory");
                return;
            };

            let look_later_run = look_later.is_none();
            let later_vec = &mut Vec::new();
            let look_later = look_later.unwrap_or(later_vec);

            'refs: for r in &*read {
                if !(*r.as_ptr()).flags.is_root_pending() {
                    continue;
                }

                //check if we have more than 1 reference that is pending
                let mut pending = 0;
                let Some(r_read) = (*r.as_ptr()).refs.spin_read() else {
                    continue;
                };

                for rr in &*r_read {
                    if (*rr.as_ptr()).flags.is_root_pending() {
                        pending += 1;
                    }

                    if pending > 1 {
                        look_later.push(*r);
                        continue 'refs;
                    }
                }

                (*r.as_ptr()).flags.set_has_no_root();
                Self::mark_dead(*r, Some(look_later));
            }

            if look_later_run {
                //TODO: we might need to run look_later again (only if a reference that blocks one in look_later is also in look_later) => max 3 times (maybe also depend on the number of references?)

                'refs: for r in look_later {
                    let Some(r_read) = (*r.as_ptr()).refs.spin_read() else {
                        continue;
                    };

                    let mut pending = 0;
                    for rr in &*r_read {
                        if (*rr.as_ptr()).flags.is_root_pending() {
                            pending += 1;
                        }

                        if pending > 1 {
                            continue 'refs;
                        }
                    }
                }
            }
        }
    }

    /*
    /// (unmark, nuke)
    fn walk_from_dead(
        &mut self,
        unmark: &mut Vec<NonNull<Self>>,
        parent: *mut Self,
    ) -> (bool, bool) {
        if self.flags.is_marked() {
            return (false, false);
        }

        let Some(ref_by) = self.ref_by.spin_read() else {
            warn!("Failed to read references from a GcBox - leaking memory");
            return (false, false); // We need to unmark this GcBox
        };

        if ref_by.len() > 1 {
            //The parent is the only one that references this GcBox, which is about to be nuked
            return (false, true); // We don't need to unmark this GcBox, since we're nuking it anyway
        }

        for r in &*ref_by {
            if r.as_ptr() == parent {
                continue;
            }

            unsafe {
                let (um, root) = (*r.as_ptr()).check_root(&mut Some(unmark));
                if um {
                    unmark.push(*r);
                }
                if root {
                    self.flags.set_has_root();

                    return (true, true); // We need to unmark this GcBox
                }
            }
        }

        self.flags.set_has_no_root();

        (true, false)
    }

     */

    fn unmark(&mut self) {
        self.flags.reset();
    }

    fn nuke(this: *mut Self) {
        unsafe {
            let _ = Box::from_raw((*this).value.as_ptr());
        }

        //TODO: we need to also check if we have only 1 reference and if we need to drop references - i guess, this should do the destruction of the GcBox?
    }

    fn you_have_root(this_ptr: NonNull<Self>, unmark: &mut Vec<NonNull<Self>>) -> RootStatus {
        let this = this_ptr.as_ptr();
        unsafe {
            if let Some(ref_by) = (*this).ref_by.spin_read().map(|x| x.len()) {
                if (*this).strong.load(Ordering::Relaxed) > ref_by {
                    return RootStatus::HasRoot;
                }
            } else {
                warn!("Failed to read references from a GcBox - leaking memory");
                return RootStatus::HasRoot; // We say that we have a root, since we'd rather have a memory leak than a use-after-free
            }
            
            
            let flags = &mut (*this).flags;

            if flags.is_has_no_root() {
                return RootStatus::HasNoRoot;
            }

            if flags.is_has_root() {
                return RootStatus::HasRoot;
            }

            if flags.is_root_pending() {
                return RootStatus::RootPending;
            }

            let Some(refs) = (*this).refs.spin_read() else {
                return RootStatus::None;
            };

            unmark.push(this_ptr);
            flags.set_has_no_root();
            let mut status = RootStatus::HasNoRoot;

            for r in &*refs {
                let root = Self::you_have_root(*r, unmark);
                if root == RootStatus::HasRoot {
                    flags.set_has_root();
                    return RootStatus::HasRoot;
                }
                if root == RootStatus::RootPending {
                    flags.set_root_pending();
                    status = RootStatus::RootPending;
                }
            }

            return status;
        }
    }
}

impl<T: ?Sized> Drop for GcBox<T> {
    fn drop(&mut self) {
        self.flags.random();

        if let Some(ref_by) = self.ref_by.spin_read() {
            assert!(
                ref_by.is_empty(),
                "Cannot drop a GcBox that is still referenced"
            );
        } else {
            warn!("Failed to proof that all references to a GcBox have been dropped - this might be bad");
            //TODO: should we also panic here?
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
                    lock.retain(|x| (*x.as_ptr()).flags != self.flags);
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
            if (*self.inner.as_ptr())
                .strong
                .fetch_sub(1, Ordering::Relaxed)
                == 1
            // We are the last one (it returns the previous value, so we need to check if it was 1)
            {
                let ptr = &mut (*self.inner.as_ptr()).value;

                //we can drop the GcBox's value, but we might need to keep the GcBox, since there might be weak references
                let _ = Box::from_raw(ptr.as_ptr()); //TODO: maybe set the ptr to usize::MAX => we need https://github.com/rust-lang/rust/issues/81513
                println!("Dropped the value");

                if (*self.inner.as_ptr()).weak.load(Ordering::Relaxed) == 0 {
                    //we can drop the complete GcBox
                    let _ = Box::from_raw(self.inner.as_ptr());
                }

                return; // if strong == 0, it means, we also know that ref_by is empty, so we can skip the rest
            }

            if Some((*self.inner.as_ptr()).strong.load(Ordering::Relaxed))
                == (*self.inner.as_ptr()).ref_by.spin_read().map(|x| x.len())
            {
                //All strong refs are references by other GcBoxes
                GcBox::shake_tree(self.inner);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    #[test]
    fn it_works() {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();

        log::error!("Hello, world!");

        let x = Gc::new(5);
        println!("{:?}", *x);
        let y = x.clone();
        println!("{:?}", *x);
        let z = x.clone();
        println!("{:?}", *x);
        let w = x.clone();

        log::error!("Hello, world!");

        println!("{:?}", *x);

        drop(y);
        println!("{:?}", *x);
        drop(z);
        println!("{:?}", *x);
        drop(w);
        println!("{:?}", *x);
    }

    #[test]
    fn circular() {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();

        log::error!("Hello, world!");

        {
            struct Node {
                data: i32,
                other: Option<Gc<RefCell<Node>>>,
            }

            let x = Gc::new(RefCell::new(Node {
                data: 5,
                other: None,
            }));

            let y = Gc::new(RefCell::new(Node {
                data: 6,
                other: Some(x.clone()),
            }));

            y.add_ref(&x);
            x.add_ref_by(&y);

            x.borrow_mut().other = Some(y.clone());
            x.add_ref(&y);
            y.add_ref_by(&x);

            println!("{:?}", x.borrow().data);
            println!("{:?}", x.borrow().other.as_ref().unwrap().borrow().data);

            println!("{:?}", y.borrow().data);
            println!("{:?}", y.borrow().other.as_ref().unwrap().borrow().data);
        }
        println!("Hello, world!");
    }
}

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fmt::Debug;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::RwLock;

use log::{debug, error, info, trace, warn};

use spin_lock::SpinLock;

use crate::trace::{TraceID, TRACER};

pub(crate) mod spin_lock;

#[cfg(feature = "trace")]
mod trace;


pub trait Collectable {}

impl<T: ?Sized> Collectable for T {}

pub struct Gc<T: Collectable> {
    inner: NonNull<GcBox<T>>,
}


impl<T: Collectable> Clone for Gc<T> {
    fn clone(&self) -> Self {
        unsafe {
            (*self.inner.as_ptr())
                .strong
                .fetch_add(1, Ordering::Relaxed);
        }

        Self { inner: self.inner }
    }
}

impl<T: Collectable> Deref for Gc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(*self.inner.as_ptr()).value.as_ptr() }
    }
}

impl<T: Collectable> Gc<T> {
    pub fn add_ref(&self, other: &Self) {
        unsafe {
            let Some(mut lock) = (*self.inner.as_ptr()).refs.spin_write() else {
                warn!("Failed to add reference to a GcBox");
                return;
            };

            lock.push(other.inner);
        }

        #[cfg(feature = "trace")] {
            TRACER.add_ref(self.trace(), other.trace());
        }
    }

    pub fn add_ref_by(&self, other: &Self) {
        unsafe {
            let Some(mut lock) = (*self.inner.as_ptr()).ref_by.spin_write() else {
                warn!("Failed to add reference to a GcBox");
                return;
            };

            lock.push(other.inner);
        }
        #[cfg(feature = "trace")]
        {
            TRACER.add_ref_by(self.trace(), other.trace());
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


        #[cfg(feature = "trace")]
        {
            TRACER.remove_ref(self.trace(), other.trace());
        }
    }

    pub fn remove_ref_by(&self, other: &Self) {
        unsafe {
            let Some(mut lock) = (*self.inner.as_ptr()).ref_by.spin_write() else {
                warn!("Failed to remove reference from a GcBox");
                return;
            };

            lock.retain(|x| x != &other.inner);
        }


        #[cfg(feature = "trace")]
        {
            TRACER.remove_ref_by(self.trace(), other.trace());
        }
    }


    #[cfg(feature = "trace")]
    fn trace(&self) -> TraceID {
        unsafe { (*self.inner.as_ptr()).trace }
    }
}

impl<T: Collectable> Gc<T> {
    pub fn new(value: T) -> Self {
        let value = Box::new(value);
        let value = unsafe { NonNull::new_unchecked(Box::into_raw(value)) }; //Unsafe, since we know that Box::into_raw will not return null

        let gc_box = GcBox {
            value,
            ref_by: RwLock::new(Vec::new()),
            refs: RwLock::new(Vec::new()),
            weak: AtomicUsize::new(0),
            strong: AtomicUsize::new(1),
            flags: Flags::new(),
            #[cfg(feature = "trace")] trace: TRACER.add(),
        };

        let gc_box = Box::new(gc_box);
        let gc_box = unsafe { NonNull::new_unchecked(Box::into_raw(gc_box)) }; //Unsafe, since we know that Box::into_raw will not return null

        Self { inner: gc_box }
    }


    pub fn root(value: T) -> Self {
        let value = Box::new(value);
        let value = unsafe { NonNull::new_unchecked(Box::into_raw(value)) }; //Unsafe, since we know that Box::into_raw will not return null

        let gc_box = GcBox {
            value,
            ref_by: RwLock::new(Vec::new()),
            refs: RwLock::new(Vec::new()),
            weak: AtomicUsize::new(0),
            strong: AtomicUsize::new(1),
            flags: Flags::root(),
            #[cfg(feature = "trace")] trace: TRACER.add(),
        };

        let gc_box = Box::new(gc_box);
        let gc_box = unsafe { NonNull::new_unchecked(Box::into_raw(gc_box)) }; //Unsafe, since we know that Box::into_raw will not return null

        Self { inner: gc_box }
    }
}

type MaybeNull<T> = NonNull<T>;

//On low-ram devices we might want to use a smaller pointer size or just use a mark-and-sweep garbage collector
struct GcBox<T: Collectable> {
    value: MaybeNull<T>,                // This value might be null
    ref_by: RwLock<Vec<NonNull<Self>>>, // All the GcBox that reference this GcBox
    refs: RwLock<Vec<NonNull<Self>>>,   // All the GcBox that this GcBox reference
    weak: AtomicUsize, // Number of weak references by for example the Garbage Collector or WeakRef in JS
    strong: AtomicUsize, // Number of strong references
    flags: Flags, // Mark for garbage collection only accessible by the garbage collector thread
    #[cfg(feature = "trace")] trace: TraceID,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Flags(u8);

#[allow(dead_code)]
impl Flags {
    const MARKED: u8 = 0b0000_0001;

    /// This `GcBox` has a root
    const HAS_ROOT: u8 = 0b0000_0010;

    /// This `GcBox` has no root
    const HAS_NO_ROOT: u8 = 0b0000_0100;

    /// This `GcBox` is root pending because we still walk the tree to find out if it is a root (used to prevent infinite loops on circular references)
    const ROOT_PENDING: u8 = 0b0000_0110;

    /// This `GcBox` is a root
    const IS_ROOT: u8 = 0b0000_1000;

    /// This `GcBox` is externally dropped, this means that only the value will be dropped if it is not already dropped, but don't remove any references etc.
    const EXTERNALLY_DROPPED: u8 = 0b0001_0000;

    /// The value of this `GcBox` is dropped
    const VALUE_DROPPED: u8 = 0b0010_0000;

    const fn new() -> Self {
        Self(0)
    }

    const fn root() -> Self {
        Self(Self::IS_ROOT)
    }

    fn set_marked(&mut self) {
        self.0 |= Self::MARKED;
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

    fn set_root(&mut self) {
        self.0 |= Self::IS_ROOT;
    }

    fn unset_root(&mut self) {
        self.0 &= !Self::IS_ROOT;
    }

    fn set_externally_dropped(&mut self) {
        self.0 |= Self::EXTERNALLY_DROPPED;
    }

    fn set_value_dropped(&mut self) {
        self.0 |= Self::VALUE_DROPPED;
    }

    const fn is_marked(self) -> bool {
        self.0 & Self::MARKED != 0
    }

    const fn is_has_root(self) -> bool {
        self.0 & Self::HAS_ROOT != 0 && self.0 & Self::HAS_NO_ROOT == 0
    }

    const fn is_has_no_root(self) -> bool {
        self.0 & Self::HAS_NO_ROOT != 0 && self.0 & Self::HAS_ROOT == 0
    }

    const fn is_root_pending(self) -> bool {
        self.0 & Self::ROOT_PENDING != 0
    }

    const fn is_root(self) -> bool {
        self.0 & Self::IS_ROOT != 0
    }

    const fn is_externally_dropped(self) -> bool {
        self.0 & Self::EXTERNALLY_DROPPED != 0
    }

    const fn is_value_dropped(self) -> bool {
        self.0 & Self::VALUE_DROPPED != 0
    }

    fn reset(&mut self) {
        self.0 = 0;
    }

    /// Unsets any root flags and marked flags, but not `IS_ROOT` or `VALUE_DROPPED`
    fn unmark(&mut self) {
        self.0 &= !(Self::MARKED | Self::HAS_ROOT | Self::HAS_NO_ROOT | Self::ROOT_PENDING);
    }
}

#[derive(Debug, PartialEq, Eq)]
enum RootStatus {
    None,
    HasRoot,
    HasNoRoot,
    RootPending,
}

pub struct WeakGc<T: Collectable> {
    #[allow(dead_code)]
    inner: NonNull<GcBox<T>>,
}

impl<T: Collectable> GcBox<T> {
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

            let (drop, unmark): (Vec<_>, Vec<_>) = unmark.into_iter().partition(|x| {
                if (*x.as_ptr()).flags.is_has_no_root() {
                    (*x.as_ptr()).flags.set_externally_dropped();
                    true
                } else {
                    false
                }
            });

            for u in unmark {
                (*u.as_ptr()).unmark();
            }

            for d in &drop {
                Self::nuke(*d, &drop);
            }
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
        self.flags.unmark();
    }

    /// The caller is responsible for making sure that the `this_ptr` already has the `EXTERNALLY_DROPPED` flag set
    unsafe fn nuke(this_ptr: NonNull<Self>, dangerous: &[NonNull<Self>]) {
        unsafe {
            let this = this_ptr.as_ptr();
            if let Some(refs) = (*this).refs.spin_read() {
                for r in &*refs {
                    if dangerous.contains(r) {
                        continue;
                    }

                    let Some(mut lock) = (*r.as_ptr()).ref_by.spin_write() else {
                        warn!("Failed to remove reference from a GcBox - leaking memory");
                        continue;
                    };

                    lock.retain(|x| *x != this_ptr);
                }
            } else {
                warn!("Failed to remove all references from a GcBox - leaking memory");
            }

            // (*this).flags.set_externally_dropped(); // We don't need to set this flag, since we already set it in shake_tree
            let _ = Box::from_raw(this);
        }
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
            if flags.is_root() {
                return RootStatus::HasRoot;
            }

            if flags.is_has_no_root() {
                return RootStatus::HasNoRoot;
            }

            if flags.is_has_root() {
                return RootStatus::HasRoot;
            }

            if flags.is_root_pending() {
                return RootStatus::RootPending;
            }

            let Some(refs) = (*this).ref_by.spin_read() else {
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

            status
        }
    }
}

impl<T: Collectable> Drop for GcBox<T> {
    fn drop(&mut self) {
        if !self.flags.is_externally_dropped() {
            error!("EEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEE");
            // Drop all references that this GcBox has and check if all references to this GcBox have been dropped
            // info!("READ_BY: {:p}", &self.ref_by);
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

            let self_raw = self as *mut Self;
            if let Some(refs) = self.refs.spin_read() {
                for r in &*refs {
                    unsafe {
                        // debug!("READ: {:p}", &(*r.as_ptr()).ref_by);
                        let Some(mut lock) = (*r.as_ptr()).ref_by.spin_write() else {
                            warn!("Failed to remove reference from a GcBox - leaking memory");
                            continue;
                        };
                        lock.retain(|x| x.as_ptr() != self_raw);
                    }
                }
            } else {
                warn!("Failed to remove all references from a GcBox - leaking memory");
            }
        }

        if !self.flags.is_value_dropped() {
            let ptr = &mut self.value;
            unsafe {
                let _ = Box::from_raw(ptr.as_ptr());
            }
            //we don't need to set the value dropped flag, since we are about to drop the complete GcBox
        }
    }
}


impl<T: Collectable> Drop for Gc<T> {
    fn drop(&mut self) {
        unsafe {
            if (*self.inner.as_ptr()).flags.is_externally_dropped() {
                return;
            }

            if (*self.inner.as_ptr())
                .strong
                .fetch_sub(1, Ordering::Relaxed)
                == 1
            // We are the last one (it returns the previous value, so we need to check if it was 1)
            {
                let ptr = (*self.inner.as_ptr()).value.as_ptr();

                //we can drop the GcBox's value, but we might need to keep the GcBox, since there might be weak references
                let _ = Box::from_raw(ptr);
                (*self.inner.as_ptr()).flags.set_value_dropped();

                if (*self.inner.as_ptr()).weak.load(Ordering::Relaxed) == 0 {
                    //we can drop the complete GcBox
                    let _ = Box::from_raw(self.inner.as_ptr());
                }

                return; // if strong == 0, it means, we also know that ref_by is empty, so we can skip the rest
                //it also would be highly unsafe to continue, since we might have already dropped the GcBox
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
#[allow(clippy::items_after_statements, dead_code)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    macro_rules! setup {
        () => {
            env_logger::Builder::from_default_env()
                .filter_level(log::LevelFilter::Trace)
                .init();



            static mut NODES_LEFT: u32 = 0;

            struct Node {
                data: i32,
                other: Option<Gc<RefCell<Node>>>,
            }

            impl Node {
                fn new(data: i32) -> Self {
                    unsafe {
                        NODES_LEFT += 1;
                    }

                    Self {
                        data,
                        other: None,
                    }
                }

                fn with_other(data: i32, other: Gc<RefCell<Node>>) -> Self {
                    unsafe {
                        NODES_LEFT += 1;
                    }

                    Self {
                        data,
                        other: Some(other),
                    }
                }
            }

            impl Drop for Node {
                fn drop(&mut self) {
                    info!("Dropping Node with data: {}", self.data);
                    unsafe {
                        NODES_LEFT -= 1;
                    }
                }
            }
        };
        (root) => {
            Gc::root(RefCell::new(Node::new(9999)))
        };
    }





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
        setup!();
        {
            let x = Gc::new(RefCell::new(Node::new(5)));

            let y = Gc::new(RefCell::new(Node::with_other(6, x.clone())));

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

        assert_eq!(unsafe { NODES_LEFT }, 0);
    }


    #[test]
    fn with_root() {
        setup!();
        let root = setup!(root);
        {
            let x = Gc::new(RefCell::new(Node::new(5)));
            let y = Gc::new(RefCell::new(Node::with_other(6, x.clone())));

            y.add_ref(&x);
            x.add_ref_by(&y);

            x.borrow_mut().other = Some(y.clone());
            x.add_ref(&y);
            y.add_ref_by(&x);

            root.add_ref(&x);
            root.borrow_mut().other = Some(x);
        }

        assert_eq!(unsafe { NODES_LEFT }, 3); //root, x, y
        {
            let x = root.borrow_mut().other.take().unwrap();

            root.remove_ref(&x);
        }

        assert_eq!(unsafe { NODES_LEFT }, 1); //root (root will never be dropped)
    }

    #[test]
    fn deep_tree() {
        setup!();
        let root = setup!(root);
        {
            let mut x = root.clone();
            for i in 0..3 {
                let x_new = Gc::new(RefCell::new(Node::with_other(i, x.clone())));

                x.add_ref_by(&x_new);
                x_new.add_ref(&x);

                x = x_new;
            }


            x.add_ref_by(&root);
            root.add_ref(&x);

            let mut root = root.borrow_mut();
            root.other = Some(x);

            info!("left: {}", unsafe { NODES_LEFT });
        }

        dbg!(root.borrow().other.as_ref().is_some());

        let left = unsafe { NODES_LEFT };

        info!("{:?}", left);

        assert_eq!(unsafe { NODES_LEFT }, 4); //root, x, y
    }
}

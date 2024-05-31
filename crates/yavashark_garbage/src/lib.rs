#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fmt::Debug;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::atomic::{AtomicUsize, Ordering};

use log::warn;

use spin_lock::SpinLock;

#[cfg(feature = "trace")]
use crate::trace::{TraceID, TRACER};

pub(crate) mod spin_lock;

#[cfg(feature = "trace")]
mod trace;

pub trait Collectable: Sized {
    /// # Safety
    fn get_refs(&self) -> Vec<Gc<Self>>;


    /// (removed, added)
    fn get_refs_diff(&self, old: &[Gc<Self>]) -> (Vec<Gc<Self>>, Vec<Gc<Self>>);
}

pub struct Gc<T: Collectable> {
    inner: NonNull<GcBox<T>>,
}


impl<T: Collectable> PartialEq for Gc<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T: Collectable> Clone for Gc<T> {
    fn clone(&self) -> Self {
        unsafe {
            (*self.inner.as_ptr()).refs.inc_strong();
        }

        Self { inner: self.inner }
    }
}


pub struct GcGuard<'a, T: Collectable> {
    value_ptr: &'a T,
    gc: NonNull<GcBox<T>>,
}


/// Here the magic of gc references happens noe!
impl<T: Collectable> Drop for GcGuard<'_, T> {
    fn drop(&mut self) {
        //we now need to update the references => look what is still there and what is not
        unsafe {
            let Some(refs) = (*self.gc.as_ptr()).refs.read_refs() else {
                warn!("Failed to read references from a GcBox - this might be bad");
                return;
            };

            let refs = refs.iter().map(|x| Gc { inner: *x }).collect::<Vec<_>>();

            let (removed, added) = self.value_ptr.get_refs_diff(&refs);

            if removed.is_empty() && added.is_empty() {
                return;
            }

            let Some(mut write) = (*self.gc.as_ptr()).refs.write_refs() else {
                warn!("Failed to write references to a GcBox - this might be bad");
                return;
            };

            for r in &removed {
                write.retain(|x| *x != r.inner);

                (*r.inner.as_ptr()).refs.remove_ref_by(self.gc);
            }

            for a in &added {
                write.push(a.inner);

                (*a.inner.as_ptr()).refs.add_ref_by(self.gc);
            }
        }
    }
}

impl<'a, T: Collectable> Deref for GcGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &'a Self::Target {
        self.value_ptr
    }
}

impl<T: Collectable> Gc<T> {
    #[must_use]
    #[allow(
        clippy::missing_const_for_fn
    )] //Bug in clippy... we can't dereference a mut ptr in a const fn
    pub fn get(&self) -> GcGuard<T> {
        let value_ptr = unsafe { (*self.inner.as_ptr()).value.as_ref() };
        GcGuard {
            value_ptr,
            gc: self.inner,
        }
    }


    #[cfg(feature = "trace")]
    fn trace(&self) -> TraceID {
        unsafe { (*self.inner.as_ptr()).trace }
    }
}

impl<T: Collectable> Gc<T> {
    pub fn new(value: T) -> Self {
        let ref_to = value.get_refs();

        let value = Box::new(value);
        let value = unsafe { NonNull::new_unchecked(Box::into_raw(value)) }; //Unsafe, since we know that Box::into_raw will not return null


        let gc_box = GcBox {
            value,
            refs: Refs::new(),
            flags: Flags::new(),
            #[cfg(feature = "trace")]
            trace: TRACER.add(),
        };

        let gc_box = Box::new(gc_box);
        let gc_box = unsafe { NonNull::new_unchecked(Box::into_raw(gc_box)) }; //Unsafe, since we know that Box::into_raw will not return null

        unsafe {
            (*gc_box.as_ptr()).refs = Refs::with_refs_to(ref_to.into_iter().map(|x| x.inner).collect());
        }


        Self { inner: gc_box }
    }

    pub fn root(value: T) -> Self {
        let value = Box::new(value);
        let value = unsafe { NonNull::new_unchecked(Box::into_raw(value)) }; //Unsafe, since we know that Box::into_raw will not return null

        let gc_box = GcBox {
            value,
            refs: Refs::new(),
            flags: Flags::root(),
            #[cfg(feature = "trace")]
            trace: TRACER.add(),
        };

        let gc_box = Box::new(gc_box);
        let gc_box = unsafe { NonNull::new_unchecked(Box::into_raw(gc_box)) }; //Unsafe, since we know that Box::into_raw will not return null

        Self { inner: gc_box }
    }
}

type MaybeNull<T> = NonNull<T>;

struct Refs<T: Collectable> {
    ref_by: RwLock<Vec<NonNull<GcBox<T>>>>,
    ref_to: RwLock<Vec<NonNull<GcBox<T>>>>,
    weak: AtomicUsize, // Number of weak references by for example the Garbage Collector or WeakRef in JS
    strong: AtomicUsize, // Number of strong references
}

impl<T: Collectable> Refs<T> {
    fn new() -> Self {
        Self {
            ref_by: RwLock::new(Vec::new()),
            ref_to: RwLock::new(Vec::new()),
            weak: AtomicUsize::new(0),
            strong: AtomicUsize::new(1),
        }
    }

    fn with_refs_to(refs: Vec<NonNull<GcBox<T>>>) -> Self {
        Self {
            ref_by: RwLock::new(Vec::new()),
            ref_to: RwLock::new(refs),
            weak: AtomicUsize::new(0),
            strong: AtomicUsize::new(1),
        }
    }

    fn add_ref(&mut self, other: NonNull<GcBox<T>>) {
        if let Some(mut lock) = self.ref_to.spin_write() {
            lock.push(other);
        } else {
            warn!("Failed to add reference to a GcBox - this might be bad");
        }
    }

    fn remove_ref(&mut self, other: NonNull<GcBox<T>>) {
        if let Some(mut lock) = self.ref_to.spin_write() {
            lock.retain(|x| *x != other);
        } else {
            warn!("Failed to remove reference from a GcBox - this might be bad");
        }
    }

    fn add_ref_by(&mut self, other: NonNull<GcBox<T>>) {
        if let Some(mut lock) = self.ref_by.spin_write() {
            lock.push(other);
        } else {
            warn!("Failed to add reference to a GcBox - this might be bad");
        }
    }

    fn remove_ref_by(&mut self, other: NonNull<GcBox<T>>) {
        if let Some(mut lock) = self.ref_by.spin_write() {
            lock.retain(|x| *x != other);
        } else {
            warn!("Failed to remove reference from a GcBox - this might be bad");
        }
    }

    fn remove_ref_by_ptr(&mut self, other: *mut GcBox<T>) {
        if let Some(mut lock) = self.ref_by.spin_write() {
            lock.retain(|x| x.as_ptr() != other);
        } else {
            warn!("Failed to remove reference from a GcBox - this might be bad");
        }
    }

    fn inc_strong(&mut self) -> usize {
        self.strong.fetch_add(1, Ordering::Relaxed)
    }

    fn dec_strong(&mut self) -> usize {
        self.strong.fetch_sub(1, Ordering::Relaxed)
    }

    fn weak(&self) -> usize {
        self.weak.load(Ordering::Relaxed)
    }

    fn strong(&self) -> usize {
        self.strong.load(Ordering::Relaxed)
    }

    fn read_refs(&self) -> Option<RwLockReadGuard<Vec<NonNull<GcBox<T>>>>> {
        self.ref_to.spin_read()
    }

    fn read_ref_by(&self) -> Option<RwLockReadGuard<Vec<NonNull<GcBox<T>>>>> {
        self.ref_by.spin_read()
    }

    fn write_refs(&self) -> Option<RwLockWriteGuard<Vec<NonNull<GcBox<T>>>>> {
        self.ref_to.spin_write()
    }
}

//On low-ram devices we might want to use a smaller pointer size or just use a mark-and-sweep garbage collector
struct GcBox<T: Collectable> {
    value: MaybeNull<T>, // This value might be null
    refs: Refs<T>,
    flags: Flags, // Mark for garbage collection only accessible by the garbage collector thread
    #[cfg(feature = "trace")]
    trace: TraceID,
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
                Self::nuke_value(*d);
            }

            for d in &drop {
                Self::nuke(*d, &drop);
            }
        }
    }

    fn mark_dead(this_ptr: NonNull<Self>, look_later: Option<&mut Vec<NonNull<Self>>>) {
        let this = this_ptr.as_ptr();

        unsafe {
            let Some(read) = (*this).refs.read_refs() else {
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
                let Some(r_read) = (*r.as_ptr()).refs.read_refs() else {
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
                    let Some(r_read) = (*r.as_ptr()).refs.read_refs() else {
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

    fn unmark(&mut self) {
        self.flags.unmark();
    }

    unsafe fn nuke_value(this_ptr: NonNull<Self>) {
        unsafe {
            let value = (*this_ptr.as_ptr()).value;
            let _ = Box::from_raw(value.as_ptr());
            (*this_ptr.as_ptr()).flags.set_value_dropped();
        }
    }

    /// The caller is responsible for making sure that the `this_ptr` already has the `EXTERNALLY_DROPPED` flag set
    unsafe fn nuke(this_ptr: NonNull<Self>, dangerous: &[NonNull<Self>]) {
        unsafe {
            let this = this_ptr.as_ptr();
            if let Some(refs) = (*this).refs.read_refs() {
                for r in &*refs {
                    if dangerous.contains(r) {
                        continue;
                    }

                    (*r.as_ptr()).refs.remove_ref_by(this_ptr);
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
            if let Some(ref_by) = (*this).refs.read_ref_by().map(|x| x.len()) {
                if (*this).refs.strong() > ref_by {
                    return RootStatus::HasRoot;
                }
            } else {
                warn!("Failed to read references from a GcBox - leaking memory");
                return RootStatus::HasRoot; // We say that we have a root, since we'd rather have a memory leak than a use-after-free
            }

            let flags = &(*this).flags;
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

            let Some(refs) = (*this).refs.read_ref_by() else {
                return RootStatus::None;
            };

            unmark.push(this_ptr);
            (*this).flags.set_has_no_root();
            let mut status = RootStatus::HasNoRoot;

            for r in &*refs {
                let root = Self::you_have_root(*r, unmark);
                if root == RootStatus::HasRoot {
                    (*this).flags.set_has_root();
                    return RootStatus::HasRoot;
                }
                if root == RootStatus::RootPending {
                    (*this).flags.set_root_pending();
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
            // Drop all references that this GcBox has and check if all references to this GcBox have been dropped
            if let Some(ref_by) = self.refs.read_ref_by() {
                //TODO: try drop or some thing here
                #[cfg(debug_assertions)]
                if !ref_by.is_empty() {
                    for r in &*ref_by {
                        println!("{:p}", r.as_ptr());

                        let r_refs = unsafe { (*r.as_ptr()).refs.read_refs() };

                        if let Some(r_refs) = r_refs {
                            for rr in &*r_refs {
                                println!("  {:p}", rr.as_ptr());
                            }
                        } else {
                            println!("  Failed to read refs");
                        }
                    }
                }

                assert!(
                    ref_by.is_empty(),
                    "Cannot drop a GcBox that is still referenced"
                );
            } else {
                warn!("Failed to proof that all references to a GcBox have been dropped - this might be bad");
                //TODO: should we also panic here?
            }

            if self.refs.weak() != 0 {
                warn!("Dropping a GcBox that still has weak references - this might be bad");
            }

            let self_raw = self as *mut Self;
            if let Some(refs) = self.refs.read_refs() {
                for r in &*refs {
                    unsafe {
                        (*r.as_ptr()).refs.remove_ref_by_ptr(self_raw);
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

            if (*self.inner.as_ptr()).refs.dec_strong() == 1 {
                // We are the last one (it returns the previous value, so we need to check if it was 1)
                let ptr = (*self.inner.as_ptr()).value.as_ptr();

                //Drop all references
                if let Some(mut refs) = (*self.inner.as_ptr()).refs.write_refs() {
                    for r in &*refs {
                        (*r.as_ptr()).refs.remove_ref_by(self.inner);
                    }

                    refs.clear();
                } else {
                    warn!("Failed to remove all references from a GcBox - leaking memory");
                    //TODO: should we return here - probably, since we might panic if we continue
                }

                //we can drop the GcBox's value, but we might need to keep the GcBox, since there might be weak references
                let _ = Box::from_raw(ptr);
                (*self.inner.as_ptr()).flags.set_value_dropped();

                if (*self.inner.as_ptr()).refs.weak() == 0 {
                    //we can drop the complete GcBox
                    let _ = Box::from_raw(self.inner.as_ptr());
                }

                return; // if strong == 0, it means, we also know that ref_by is empty, so we can skip the rest
                //it also would be highly unsafe to continue, since we might have already dropped the GcBox
            }

            if Some((*self.inner.as_ptr()).refs.strong())
                == (*self.inner.as_ptr()).refs.read_ref_by().map(|x| x.len())
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
    use std::sync::Once;

    use log::info;

    use super::*;

    macro_rules! setup {
        () => {
            setup_logger();

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

                    Self { data, other: None }
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

            impl Collectable for RefCell<Node> {
                fn get_refs(&self) -> Vec<Gc<Self>> {
                    let this = self.borrow();
                    if let Some(other) = &this.other {
                        vec![other.clone()]
                    } else {
                        Vec::new()
                    }
                }

                fn get_refs_diff(&self, old: &[Gc<Self>]) -> (Vec<Gc<Self>>, Vec<Gc<Self>>) {
                    let this = self.borrow();

                    if let Some(other) = &this.other {
                        if old.contains(other) {
                            (Vec::new(), Vec::new())
                        } else {
                            (vec![other.clone()], Vec::new())
                        }
                    } else {
                        (Vec::new(), Vec::new())
                    }
                }
            }


        };
        (root) => {
            Gc::root(RefCell::new(Node::new(9999)))
        };
    }

    static LOGGER: Once = Once::new();

    fn setup_logger() {
        #[cfg(not(miri))]
        {
            LOGGER.call_once(|| {
                env_logger::Builder::from_default_env()
                    .filter_level(log::LevelFilter::Trace)
                    .init();
            });
        }
    }

    #[test]
    fn it_works() {
        setup_logger();


        impl Collectable for i32 {
            fn get_refs(&self) -> Vec<Gc<i32>> {
                Vec::new()
            }

            fn get_refs_diff(&self, _old: &[Gc<i32>]) -> (Vec<Gc<i32>>, Vec<Gc<i32>>) {
                (Vec::new(), Vec::new())
            }
        }

        let x = Gc::new(5);
        println!("{:?}", *x.get());
        let y = x.clone();
        println!("{:?}", *x.get());
        let z = x.clone();
        println!("{:?}", *x.get());
        let w = x.clone();

        println!("{:?}", *x.get());

        drop(y);
        println!("{:?}", *x.get());
        drop(z);
        println!("{:?}", *x.get());
        drop(w);
        println!("{:?}", *x.get());
    }

    #[test]
    fn circular() {
        setup!();

        {
            let x = Gc::new(RefCell::new(Node::new(5)));

            let y = Gc::new(RefCell::new(Node::with_other(6, x.clone())));

            x.get().borrow_mut().other = Some(y);
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


            x.get().borrow_mut().other = Some(y.clone());


            root.get().borrow_mut().other = Some(x);
        }

        assert_eq!(unsafe { NODES_LEFT }, 3); //root, x, y
        {
            let x = root.get().borrow_mut().other.take().unwrap();
        }

        assert_eq!(unsafe { NODES_LEFT }, 1); //root (root will never be dropped)
    }

    #[test]
    fn deep_tree() {
        setup!();
        {
            let root = setup!(root);
            {
                let mut x = root.clone();
                for i in 0..100 {
                    let x_new = Gc::new(RefCell::new(Node::with_other(i, x.clone())));

                    x = x_new;
                }


                let root = root.get();
                let mut root = root.borrow_mut();
                root.other = Some(x);

                info!("left: {}", unsafe { NODES_LEFT });
            }

            let left = unsafe { NODES_LEFT };

            info!("{:?}", left);

            assert_eq!(unsafe { NODES_LEFT }, 101); //root, x, y

            unsafe {
                (*root.inner.as_ptr()).flags.unset_root();
            }
        }

        assert_eq!(unsafe { NODES_LEFT }, 0);
    }
}

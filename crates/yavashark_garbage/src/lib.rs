#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![cfg_attr(miri, feature(strict_provenance, exposed_provenance))]

use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicU32, Ordering};

use log::warn;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use spin_lock::SpinLock;

use crate::tagged_ptr::TaggedPtr;
#[cfg(feature = "easy_debug")]
use crate::trace::{TraceID, TRACER};

pub(crate) mod spin_lock;

pub mod collectable;
pub(crate) mod tagged_ptr;
#[cfg(feature = "easy_debug")]
mod trace;
#[cfg(feature = "trace")]
mod trace_gui;

/// # Safety
/// The implementer must guarantee that all references are valid and all references are returned by `get_refs`
pub unsafe trait Collectable: Sized {
    fn get_refs(&self) -> Vec<GcRef<Self>>;

    /// Execute the destructor and free the value
    /// # Safety
    /// this actually needs to be non-null
    unsafe fn deallocate(this: NonNull<[(); 0]>) {
        let this: NonNull<Self> = this.cast();

        let _ = Box::from_raw(this.as_ptr());
    }

    #[cfg(feature = "easy_debug")]
    fn trace_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
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

impl<T: Collectable + Debug> Debug for Gc<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Gc")
            .field("data", unsafe { (*self.inner.as_ptr()).value.as_ref() })
            .field("strong", &unsafe { (*self.inner.as_ptr()).refs.strong() })
            .field("weak", &unsafe { (*self.inner.as_ptr()).refs.weak() })
            .finish()
    }
}

///Function to completely deallocate the value, including freeing the memory!
type DeallocFn = unsafe fn(NonNull<[(); 0]>);

pub struct UntypedGcRef {
    gc_box: NonNull<GcBox<()>>,
    dealloc_value: DeallocFn,
}

impl Clone for UntypedGcRef {
    fn clone(&self) -> Self {
        Self {
            gc_box: self.gc_box,
            dealloc_value: self.dealloc_value,
        }
    }
}

impl UntypedGcRef {
    fn new<T: Collectable>(ptr: NonNull<GcBox<T>>) -> Self {
        Self {
            gc_box: ptr.cast(),
            dealloc_value: T::deallocate,
        }
    }
}

pub struct GcRef<T: Collectable> {
    /// # Safety
    /// this pointer might not be a pointer to a `GcBox`, but also ca be a pointer to a `UntypedGcRef`
    ptr: TaggedPtr<GcBox<T>>,
}

impl<T: Collectable> Drop for GcRef<T> {
    fn drop(&mut self) {
        if self.ptr.tag() {
            let this: NonNull<UntypedGcRef> = self.ptr.ptr().cast();

            let _ = unsafe { Box::from_raw(this.as_ptr()) };
        }
    }
}

impl<T: Collectable> Debug for GcRef<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GcRef")
            .field("ptr_value", &self.ptr.ptr())
            .field("ptr_tag", &self.ptr.tag())
            .finish()
    }
}

impl<T: Collectable> Clone for GcRef<T> {
    fn clone(&self) -> Self {
        if self.ptr.tag() {
            let ptr: NonNull<UntypedGcRef> = self.ptr.ptr().cast();
            unsafe {
                let new = Box::new((*ptr.as_ptr()).clone());

                let untyped = NonNull::new_unchecked(Box::into_raw(new));

                let ptr = TaggedPtr::new(untyped, true).cast();

                Self { ptr }
            }
        } else {
            Self { ptr: self.ptr }
        }
    }
}

impl<T: Collectable> GcRef<T> {
    fn add_ref_by(&self, r: impl Into<Self>) {
        let r = r.into();

        let ptr = self.ptr.ptr();

        if self.ptr.tag() {
            let ptr: NonNull<UntypedGcRef> = ptr.cast();

            unsafe {
                (*(*ptr.as_ptr()).gc_box.as_ptr()).refs.add_ref_by(r.cast());
            }
        } else {
            unsafe {
                (*ptr.as_ptr()).refs.add_ref_by(r);
            }
        }
    }

    fn cast<U: Collectable>(&self) -> GcRef<U> {
        if self.ptr.tag() {
            let untyped: NonNull<UntypedGcRef> = self.ptr.ptr().cast();

            let clone = unsafe { (*untyped.as_ptr()).clone() };

            let untyped = Box::new(clone);

            // Unsafe because Box::into_raw guarantees the returned ptr is non-null
            let untyped = unsafe { NonNull::new_unchecked(Box::into_raw(untyped)) };

            let ptr = TaggedPtr::new(untyped, true).cast();

            GcRef { ptr }
        } else {
            let untyped = Box::new(UntypedGcRef::new(self.ptr.ptr()));

            // Unsafe because Box::into_raw guarantees the returned ptr is non-null
            let untyped = unsafe { NonNull::new_unchecked(Box::into_raw(untyped)) };

            let ptr = TaggedPtr::new(untyped, true).cast();

            GcRef { ptr }
        }
    }

    fn cast_with_dealloc<U: Collectable>(&self, dealloc: DeallocFn) -> GcRef<U> {
        if self.ptr.tag() {
            let untyped: NonNull<UntypedGcRef> = self.ptr.ptr().cast();

            let clone = unsafe { (*untyped.as_ptr()).clone() };

            let untyped = Box::new(clone);

            // Unsafe because Box::into_raw guarantees the returned ptr is non-null
            let untyped = unsafe { NonNull::new_unchecked(Box::into_raw(untyped)) };

            let ptr = TaggedPtr::new(untyped, true).cast();

            GcRef { ptr }
        } else {
            let untyped = Box::new(UntypedGcRef {
                gc_box: self.ptr.ptr().cast(),
                dealloc_value: dealloc,
            });

            // Unsafe because Box::into_raw guarantees the returned ptr is non-null
            let untyped = unsafe { NonNull::new_unchecked(Box::into_raw(untyped)) };

            let ptr = TaggedPtr::new(untyped, true).cast();

            GcRef { ptr }
        }
    }

    fn box_ptr(&self) -> NonNull<GcBox<()>> {
        if self.ptr.tag() {
            let this: NonNull<UntypedGcRef> = self.ptr.ptr().cast();

            unsafe { (*this.as_ptr()).gc_box }
        } else {
            self.ptr.ptr().cast()
        }
    }

    #[cfg(feature = "easy_debug")]
    fn trace_id(&self) -> TraceID {
        unsafe { (*self.box_ptr().as_ptr()).refs.trace }
    }
}

impl<T: Collectable> PartialEq for GcRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.box_ptr() == other.box_ptr()
    }
}

impl<T: Collectable> Eq for GcRef<T> {}

impl<T: Collectable> PartialEq<Gc<T>> for GcRef<T> {
    fn eq(&self, other: &Gc<T>) -> bool {
        self.ptr.as_ptr() == other.inner.as_ptr()
    }
}

impl<T: Collectable> PartialEq<GcRef<T>> for Gc<T> {
    fn eq(&self, other: &GcRef<T>) -> bool {
        self.inner.as_ptr() == other.ptr.as_ptr()
    }
}

impl<T: Collectable> From<NonNull<GcBox<T>>> for GcRef<T> {
    fn from(inner: NonNull<GcBox<T>>) -> Self {
        Self { ptr: inner.into() }
    }
}

pub struct GcGuard<'a, T: Collectable> {
    value_ptr: &'a T,
    gc: NonNull<GcBox<T>>,
}

/// Here the magic of gc references happens!
impl<T: Collectable> Drop for GcGuard<'_, T> {
    fn drop(&mut self) {
        //we now need to update the references => look what is still there and what is not
        unsafe {
            GcBox::update_refs(self.gc);
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
    #[allow(clippy::missing_const_for_fn)] //Bug in clippy... we can't dereference a mut ptr in a const fn
    pub fn get(&self) -> GcGuard<T> {
        let value_ptr = unsafe { (*self.inner.as_ptr()).value.as_ref() };
        GcGuard {
            value_ptr,
            gc: self.inner,
        }
    }
    
    
    pub fn ptr_id(&self) -> usize {
        self.inner.as_ptr() as usize
    }

    #[must_use]
    pub fn as_ptr(&self) -> *mut T {
        unsafe { (*self.inner.as_ptr()).value.as_ptr() }
    }

    ///Just a reference without incrementing the reference count.
    #[must_use]
    pub fn get_ref(&self) -> GcRef<T> {
        self.inner.into()
    }

    #[must_use]
    pub fn get_untyped_ref<O: Collectable>(&self) -> GcRef<O> {
        unsafe {
            let untyped = Box::new(UntypedGcRef::new(self.inner));

            //Unsafe because Box::into_raw is guaranteed to return a non-null pointer
            let untyped = NonNull::new_unchecked(Box::into_raw(untyped));

            // Safety: we set the tag bit, which means actually isn't a GcBox<O> but a UntypedGcRef
            let ptr = TaggedPtr::new(untyped, true).cast();

            GcRef { ptr }
        }
    }
}

impl<T: Collectable> From<NonNull<GcBox<T>>> for Gc<T> {
    fn from(inner: NonNull<GcBox<T>>) -> Self {
        unsafe { (*inner.as_ptr()).refs.inc_strong() };
        Self { inner }
    }
}

impl<T: Collectable> Gc<T> {
    pub fn new(value: T) -> Self {
        #[cfg(feature = "easy_debug")]
        let name = value.trace_name();

        let ref_to = value.get_refs();

        let value = Box::new(value);
        let value = unsafe { NonNull::new_unchecked(Box::into_raw(value)) }; //Unsafe, since we know that Box::into_raw will not return null

        let gc_box = GcBox {
            value,
            refs: Refs::new(),
            flags: Flags::new(),
            #[cfg(feature = "easy_debug")]
            ty_name: std::any::type_name::<T>(),
            #[cfg(feature = "easy_debug")]
            name,
        };

        let gc_box = Box::new(gc_box);
        let gc_box = unsafe { NonNull::new_unchecked(Box::into_raw(gc_box)) }; //Unsafe, since we know that Box::into_raw will not return null

        unsafe {
            for x in &ref_to {
                x.add_ref_by(gc_box);
            }

            (*gc_box.as_ptr()).refs.ref_to = RwLock::new(ref_to);

            #[cfg(feature = "easy_debug")]
            for r in &*(*gc_box.as_ptr()).refs.ref_to.read_recursive() {
                let id = (*gc_box.as_ptr()).refs.trace;

                TRACER.add_ref(id, r.trace_id());
            }
        }

        Self { inner: gc_box }
    }

    pub fn root(value: T) -> Self {
        #[cfg(feature = "easy_debug")]
        let name = value.trace_name();

        let value = Box::new(value);
        let value = unsafe { NonNull::new_unchecked(Box::into_raw(value)) }; //Unsafe, since we know that Box::into_raw will not return null

        let gc_box = GcBox {
            value,
            refs: Refs::new(),
            flags: Flags::root(),
            #[cfg(feature = "easy_debug")]
            ty_name: std::any::type_name::<T>(),
            #[cfg(feature = "easy_debug")]
            name,
        };

        let gc_box = Box::new(gc_box);
        let gc_box = unsafe { NonNull::new_unchecked(Box::into_raw(gc_box)) }; //Unsafe, since we know that Box::into_raw will not return null

        Self { inner: gc_box }
    }

    #[must_use]
    pub fn strong(&self) -> u32 {
        unsafe { (*self.inner.as_ptr()).refs.strong() }
    }

    #[must_use]
    pub fn weak(&self) -> u32 {
        unsafe { (*self.inner.as_ptr()).refs.weak() }
    }
}

type MaybeNull<T> = NonNull<T>;

struct Refs<T: Collectable> {
    ref_by: RwLock<Vec<GcRef<T>>>,
    ref_to: RwLock<Vec<GcRef<T>>>,
    weak: AtomicU32, // Number of weak references by for example the Garbage Collector or WeakRef in JS
    strong: AtomicU32, // Number of strong references
    #[cfg(feature = "easy_debug")]
    trace: TraceID,
}

impl<T: Collectable> Refs<T> {
    #[allow(clippy::missing_const_for_fn)]
    fn new() -> Self {
        Self {
            ref_by: RwLock::new(Vec::new()),
            ref_to: RwLock::new(Vec::new()),
            weak: AtomicU32::new(0),
            strong: AtomicU32::new(1),
            #[cfg(feature = "easy_debug")]
            trace: TRACER.add(),
        }
    }

    #[allow(clippy::needless_pass_by_ref_mut)]
    fn add_ref_by(&mut self, other: impl Into<GcRef<T>>) {
        if let Some(mut lock) = self.ref_by.spin_write() {
            lock.push(other.into());
        } else {
            warn!("Failed to add reference to a GcBox - this might be bad");
        }
    }

    #[allow(clippy::needless_pass_by_ref_mut)]
    fn remove_ref_by(&mut self, other: NonNull<GcBox<T>>) {
        if let Some(mut lock) = self.ref_by.spin_write() {
            lock.retain(|x| x.box_ptr() != other.cast());
        } else {
            warn!("Failed to remove reference from a GcBox - this might be bad");
        }
    }

    #[allow(clippy::needless_pass_by_ref_mut)]
    fn remove_ref_by_ptr(&mut self, other: *mut GcBox<T>) {
        if let Some(mut lock) = self.ref_by.spin_write() {
            lock.retain(|x| x.box_ptr().as_ptr() != other.cast());
        } else {
            warn!("Failed to remove reference from a GcBox - this might be bad");
        }
    }

    #[allow(clippy::needless_pass_by_ref_mut)]
    fn inc_strong(&mut self) -> u32 {
        self.strong.fetch_add(1, Ordering::Relaxed)
    }

    #[allow(clippy::needless_pass_by_ref_mut)]
    fn dec_strong(&mut self) -> u32 {
        self.strong.fetch_sub(1, Ordering::Relaxed)
    }

    fn weak(&self) -> u32 {
        self.weak.load(Ordering::Relaxed)
    }

    fn strong(&self) -> u32 {
        self.strong.load(Ordering::Relaxed)
    }

    fn read_refs(&self) -> Option<RwLockReadGuard<Vec<GcRef<T>>>> {
        self.ref_to.spin_read()
    }

    fn read_ref_by(&self) -> Option<RwLockReadGuard<Vec<GcRef<T>>>> {
        self.ref_by.spin_read()
    }

    fn write_refs(&self) -> Option<RwLockWriteGuard<Vec<GcRef<T>>>> {
        self.ref_to.spin_write()
    }
}

//On low-ram devices we might want to use a smaller pointer size or just use a mark-and-sweep garbage collector
struct GcBox<T: Collectable> {
    value: MaybeNull<T>, // This value might be null
    refs: Refs<T>,
    flags: Flags, // Mark for garbage collection only accessible by the garbage collector thread
    #[cfg(feature = "easy_debug")]
    #[allow(dead_code)]
    ty_name: &'static str,
    #[cfg(feature = "easy_debug")]
    #[allow(dead_code)]
    name: &'static str,
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
    #[allow(dead_code)]
    None,
    HasRoot,
    HasNoRoot,
    RootPending,
}

#[allow(unused)]
pub struct WeakGc<T: Collectable> {
    #[allow(dead_code)]
    inner: NonNull<GcBox<T>>,
}

impl<T: Collectable> GcBox<T> {
    fn shake_tree(this_ref: &GcRef<T>) {
        let mut unmark = Vec::new();
        unsafe {
            let status = Self::you_have_root(this_ref, &mut unmark);

            match status {
                RootStatus::HasRoot => {
                    for r in unmark {
                        (*r.box_ptr().as_ptr()).unmark();
                    }

                    return;
                }

                RootStatus::HasNoRoot => {}
                RootStatus::RootPending => {
                    (*this_ref.box_ptr().as_ptr()).flags.set_has_no_root();
                    Self::mark_dead(this_ref.box_ptr(), None);
                }

                RootStatus::None => {
                    warn!("Failed to find root status for a GcBox");
                    return;
                }
            }

            let (drop, unmark): (Vec<_>, Vec<_>) = unmark.into_iter().partition(|x| {
                if (*x.box_ptr().as_ptr()).flags.is_has_no_root() {
                    (*x.box_ptr().as_ptr()).flags.set_externally_dropped();
                    true
                } else {
                    false
                }
            });

            for u in unmark {
                (*u.box_ptr().as_ptr()).unmark();
            }

            for d in &drop {
                Self::nuke_refs(d.box_ptr());
            }

            for d in &drop {
                if (*d.box_ptr().as_ptr()).flags.is_value_dropped() {
                    continue;
                }

                Self::nuke_value(d.ptr);
            }

            for d in &drop {
                Self::nuke(d.box_ptr());
            }
        }
    }

    fn mark_dead(this_ptr: NonNull<GcBox<()>>, look_later: Option<&mut Vec<NonNull<GcBox<()>>>>) {
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
                if !(*r.box_ptr().as_ptr()).flags.is_root_pending() {
                    continue;
                }

                //check if we have more than 1 reference that is pending
                let mut pending = 0;
                let Some(r_read) = (*r.box_ptr().as_ptr()).refs.read_refs() else {
                    continue;
                };

                for rr in &*r_read {
                    if (*rr.box_ptr().as_ptr()).flags.is_root_pending() {
                        pending += 1;
                    }

                    if pending > 1 {
                        look_later.push(r.box_ptr());
                        continue 'refs;
                    }
                }

                (*r.box_ptr().as_ptr()).flags.set_has_no_root();
                Self::mark_dead(r.box_ptr(), Some(look_later));
            }

            if look_later_run {
                //TODO: we might need to run look_later again (only if a reference that blocks one in look_later is also in look_later) => max 3 times (maybe also depend on the number of references?)

                'refs: for r in look_later {
                    let Some(r_read) = (*r.as_ptr()).refs.read_refs() else {
                        continue;
                    };

                    let mut pending = 0;
                    for rr in &*r_read {
                        if (*rr.box_ptr().as_ptr()).flags.is_root_pending() {
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

    unsafe fn nuke_value(this_ptr: TaggedPtr<Self>) {
        if this_ptr.tag() {
            //The ptr doesn't point to a self ref, but instead to a GcTreeRef
            let this: NonNull<UntypedGcRef> = this_ptr.ptr().cast();
            let this = this.as_ptr();

            unsafe {
                #[cfg(feature = "easy_debug")]
                {
                    TRACER.remove((*(*this).gc_box.as_ptr()).refs.trace);
                }
                let value = (*(*this).gc_box.as_ptr()).value.cast();

                ((*this).dealloc_value)(value);
            }

            (*(*this).gc_box.as_ptr()).flags.set_value_dropped();
            return;
        }

        unsafe {
            let value = (*this_ptr.as_ptr()).value;

            let _ = Box::from_raw(value.as_ptr());
            (*this_ptr.as_ptr()).flags.set_value_dropped();
            #[cfg(feature = "easy_debug")]
            {
                TRACER.remove((*this_ptr.as_ptr()).refs.trace);
            }
        }
    }

    /// The caller is responsible for making sure that the `this_ptr` already has the `EXTERNALLY_DROPPED` flag set
    unsafe fn nuke(this_ptr: NonNull<GcBox<()>>) {
        unsafe {
            let this = this_ptr.as_ptr();
            // (*this).flags.set_externally_dropped(); // We don't need to set this flag, since we already set it in shake_tree
            let _ = Box::from_raw(this);
        }
    }

    unsafe fn nuke_refs(this_ptr: NonNull<GcBox<()>>) {
        unsafe {
            let this = this_ptr.as_ptr();
            if let Some(refs) = (*this).refs.read_refs() {
                for r in &*refs {
                    (*r.box_ptr().as_ptr()).refs.remove_ref_by(this_ptr);
                }
            } else {
                warn!("Failed to remove all references from a GcBox - leaking memory");
            }
        }
    }

    fn you_have_root(this_ptr: &GcRef<T>, unmark: &mut Vec<GcRef<T>>) -> RootStatus {
        let this = this_ptr.box_ptr().as_ptr();
        unsafe {
            let Some(refs) = (*this).refs.read_ref_by() else {
                warn!("Failed to read references from a GcBox - leaking memory");
                return RootStatus::HasRoot; // We say that we have a root, since we'd rather have a memory leak than a use-after-free
            };

            if (*this).refs.strong() > refs.len() as u32 {
                return RootStatus::HasRoot;
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

            unmark.push(this_ptr.clone());
            (*this).flags.set_has_no_root();
            let mut status = RootStatus::HasNoRoot;

            if this_ptr.ptr.tag() {
                for r in &*refs {
                    //TODO: maybe we could improve this to avoid unnecessary clones here, since we
                    //we know that this_ptr hasn't T as the real type
                    let ptr: NonNull<UntypedGcRef> = this_ptr.ptr.ptr().cast();
                    let dealloc = (*ptr.as_ptr()).dealloc_value;

                    let root = Self::you_have_root(&r.clone().cast_with_dealloc(dealloc), unmark);
                    if root == RootStatus::HasRoot {
                        (*this).flags.set_has_root();
                        return RootStatus::HasRoot;
                    }
                    if root == RootStatus::RootPending {
                        (*this).flags.set_root_pending();
                        status = RootStatus::RootPending;
                    }
                }
            } else {
                let refs = &*refs;

                // Safety
                // We know that the this_ptr has T as the type since we don't have the tag set, so this is safe
                let refs: &Vec<GcRef<T>> = &*std::ptr::from_ref(refs).cast();

                for r in refs {
                    let root = Self::you_have_root(r, unmark);
                    if root == RootStatus::HasRoot {
                        (*this).flags.set_has_root();
                        return RootStatus::HasRoot;
                    }
                    if root == RootStatus::RootPending {
                        (*this).flags.set_root_pending();
                        status = RootStatus::RootPending;
                    }
                }
            }

            status
        }
    }

    unsafe fn update_refs(this_ptr: NonNull<Self>) {
        let value = (*this_ptr.as_ptr()).value.as_ref();

        let all_refs = value.get_refs();

        // TODO: from here on we could switch threads to a other thread, so we don't block the main thread anymore
        let Some(refs_lock) = (*this_ptr.as_ptr()).refs.read_refs() else {
            warn!("Failed to read references from a GcBox - this might be bad");
            return;
        };

        let old_refs = refs_lock.clone();
        drop(refs_lock);

        let removed = old_refs
            .iter()
            .filter(|r| !all_refs.contains(r))
            .collect::<Vec<_>>();
        let added = all_refs
            .into_iter()
            .filter(|r| !old_refs.contains(r))
            .collect::<Vec<_>>();

        if removed.is_empty() && added.is_empty() {
            return;
        }

        let Some(mut write) = (*this_ptr.as_ptr()).refs.write_refs() else {
            warn!("Failed to write references to a GcBox - this might be bad");
            return;
        };

        for r in removed {
            let ptr = r.box_ptr();

            write.retain(|x| x.box_ptr() != ptr);

            let ptr = ptr.as_ptr();
            (*ptr).refs.remove_ref_by(this_ptr.cast());

            Self::collect(r);

            #[cfg(feature = "easy_debug")]
            {
                let this_id = (*this_ptr.as_ptr()).refs.trace;
                TRACER.remove_ref(this_id, r.trace_id());
            }
        }

        for a in &added {
            write.push(a.clone());
            #[cfg(feature = "easy_debug")]
            {
                let this_id = (*this_ptr.as_ptr()).refs.trace;
                TRACER.add_ref(this_id, a.trace_id());
            }
        }

        drop(write);

        for a in &added {
            if a.ptr.tag() {
                (*a.box_ptr().as_ptr())
                    .refs
                    .add_ref_by(GcRef::from(this_ptr).cast());
            } else {
                (*a.ptr.ptr().as_ptr())
                    .refs
                    .add_ref_by(GcRef::from(this_ptr));
            }
        }
    }

    unsafe fn collect(this: &GcRef<T>) {
        let this_ptr = this.box_ptr().as_ptr();

        let strong = (*this_ptr).refs.strong();
        let Some(refs) = (*this_ptr).refs.read_ref_by().map(|x| x.len() as u32) else {
            warn!("Failed to check refs - leaking memory");
            return;
        };

        if strong == refs {
            //All strong refs are references by other GcBoxes
            Self::shake_tree(this);
        } else {
            #[cfg(debug_assertions)]
            if strong < refs {
                // In theory this should only occur if we remove a ref and instantly drop it, while the gc refs aren't updated yet
                warn!("Less strong refs than ref_bys - wrong use of gc or memory leak");
            }
        }
    }
}

impl<T: Collectable> Drop for GcBox<T> {
    fn drop(&mut self) {
        if !self.flags.is_externally_dropped() {
            // Drop all references that this GcBox has and check if all references to this GcBox have been dropped

            #[cfg(debug_assertions)]
            if let Some(ref_by) = self.refs.read_ref_by() {
                //TODO: try drop or some thing here
                #[cfg(debug_assertions)]
                if !ref_by.is_empty() {
                    for r in &*ref_by {
                        println!("{:p}", r.box_ptr().as_ptr());

                        let r_refs = unsafe { (*r.box_ptr().as_ptr()).refs.read_refs() };

                        if let Some(r_refs) = r_refs {
                            for rr in &*r_refs {
                                println!("  {:p}", rr.box_ptr().as_ptr());
                            }
                        } else {
                            println!("  Failed to read refs");
                        }
                    }
                }

                debug_assert!(
                    ref_by.is_empty(),
                    "Cannot drop a GcBox that is still referenced - wrong use of gc or memory leak"
                );
            } else {
                warn!("Failed to proof that all references to a GcBox have been dropped - this might be bad");
                //TODO: should we also panic here?
            }

            if self.refs.weak() != 0 || self.refs.strong() != 0 {
                warn!("Dropping a GcBox that still has references - this might be bad");
            }

            let self_raw = self as *mut Self;
            if let Some(refs) = self.refs.read_refs() {
                for r in &*refs {
                    unsafe {
                        (*r.box_ptr().as_ptr())
                            .refs
                            .remove_ref_by_ptr(self_raw.cast());
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
                #[cfg(feature = "easy_debug")]
                {
                    TRACER.remove(self.refs.trace);
                }
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

            let old_strong = (*self.inner.as_ptr()).refs.dec_strong();

            if old_strong == 1 {
                // We are the last one (it returns the previous value, so we need to check if it was 1)
                let ptr = (*self.inner.as_ptr()).value.as_ptr();

                //Drop all references
                if let Some(mut refs) = (*self.inner.as_ptr()).refs.write_refs() {
                    for r in &*refs {
                        (*r.box_ptr().as_ptr())
                            .refs
                            .remove_ref_by(self.inner.cast());
                    }

                    refs.clear();
                } else {
                    warn!("Failed to remove all references from a GcBox - leaking memory");
                    //TODO: should we return here - probably, since we might panic if we continue
                }

                // if let Some(refs) = (*self.inner.as_ptr()).refs.read_ref_by() {
                //     let can_drop = true;
                //
                //     for r in &*refs {
                //         (*r.box_ptr().as_ptr()).refs.
                //     }
                // }

                //we can drop the GcBox's value, but we might need to keep the GcBox, since there might be weak references

                #[cfg(debug_assertions)]
                if (*self.inner.as_ptr()).flags.is_value_dropped() {
                    warn!("Dropping value that was already dropped!");
                }

                let _ = Box::from_raw(ptr);
                (*self.inner.as_ptr()).flags.set_value_dropped();

                let Some(refs) = (*self.inner.as_ptr()).refs.read_ref_by() else {
                    warn!("Failed to read references - leaking memory");
                    //we don't need to execute GcBox::collect because it will also read `refs.ref_by` and we just failed to do so,
                    //so it also won't succeed.
                    return;
                };

                if (*self.inner.as_ptr()).refs.weak() == 0 && refs.len() == 0 {
                    drop(refs);
                    //we can drop the complete GcBox
                    let _ = Box::from_raw(self.inner.as_ptr());
                }

                return; // if strong == 0, it means, we also know that ref_by is empty, so we can skip the rest
                        //it also would be highly unsafe to continue, since we might have already dropped the GcBox
            }

            GcBox::collect(&self.inner.into());
        }
    }
}

#[cfg(test)]
#[allow(clippy::items_after_statements, dead_code)]
mod tests {
    use std::cell::RefCell;
    use std::sync::Once;

    use log::info;

    use crate::collectable::CellCollectable;

    use super::*;

    macro_rules! setup {
        () => {
            setup_logger();

            static mut NODES_LEFT: u32 = 0;

            struct Node {
                data: i32,
                other: Vec<Gc<RefCell<Node>>>,
            }

            unsafe impl CellCollectable<RefCell<Node>> for Node {
                fn get_refs(&self) -> Vec<GcRef<RefCell<Self>>> {
                    self.other.iter().map(|x| x.get_ref()).collect()
                }
            }

            impl Node {
                const fn create(data: i32, other: Vec<Gc<RefCell<Node>>>) -> Self {
                    Self { data, other }
                }
            }

            setup!(funcs);
        };

        (untyped) => {
            setup_logger();

            static mut NODES_LEFT: u32 = 0;

            #[derive(Debug)]
            struct Node {
                data: i32,
                other: Vec<Gc<RefCell<Node>>>,
                other_type: Vec<Gc<Other>>,
            }

            #[derive(Debug)]
            struct Other {
                data: i32,
            }

            unsafe impl Collectable for Other {
                fn get_refs(&self) -> Vec<GcRef<Self>> {
                    Vec::new()
                }
            }

            impl Other {
                fn new(data: i32) -> Gc<Self> {
                    unsafe {
                        NODES_LEFT += 1;
                    }

                    Gc::new(Self { data })
                }
            }

            impl Drop for Other {
                fn drop(&mut self) {
                    unsafe {
                        NODES_LEFT -= 1;
                    }
                }
            }

            impl Node {
                const fn create(data: i32, other: Vec<Gc<RefCell<Node>>>) -> Self {
                    Self {
                        data,
                        other,
                        other_type: Vec::new(),
                    }
                }

                const fn create_all(
                    data: i32,
                    other: Vec<Gc<RefCell<Node>>>,
                    other_type: Vec<Gc<Other>>,
                ) -> Self {
                    Self {
                        data,
                        other,
                        other_type,
                    }
                }

                fn with_other_type(
                    data: i32,
                    other: Vec<Gc<RefCell<Node>>>,
                    other_type: Vec<Gc<Other>>,
                ) -> Self {
                    unsafe {
                        NODES_LEFT += 1;
                    }
                    Self::create_all(data, other, other_type)
                }

                fn add_vec_type(
                    data: i32,
                    others: Vec<&Gc<RefCell<Node>>>,
                    other_type: Vec<&Gc<Other>>,
                ) -> Gc<RefCell<Node>> {
                    unsafe {
                        NODES_LEFT += 1;
                    }

                    let other = others.iter().map(|e| (*e).clone()).collect();
                    let other_type = other_type.iter().map(|e| (*e).clone()).collect();

                    Gc::new(RefCell::new(Self::create_all(data, other, other_type)))
                }
            }

            unsafe impl CellCollectable<RefCell<Node>> for Node {
                fn get_refs(&self) -> Vec<GcRef<RefCell<Self>>> {
                    let mut refs: Vec<_> = self.other.iter().map(|x| x.get_ref()).collect();

                    for r in &self.other_type {
                        refs.push(r.get_untyped_ref())
                    }

                    refs
                }
            }

            setup!(funcs);
        };

        (funcs) => {
            impl Node {
                fn new(data: i32) -> Self {
                    unsafe {
                        NODES_LEFT += 1;
                    }

                    Self::create(data, Vec::new())
                }

                fn add(data: i32) -> Gc<RefCell<Node>> {
                    Gc::new(RefCell::new(Self::new(data)))
                }

                fn with_other(data: i32, other: Gc<RefCell<Node>>) -> Self {
                    unsafe {
                        NODES_LEFT += 1;
                    }

                    Self::create(data, vec![other])
                }

                fn add_with_other(data: i32, other: &Gc<RefCell<Node>>) -> Gc<RefCell<Node>> {
                    Gc::new(RefCell::new(Self::with_other(data, other.clone())))
                }

                fn add_vec(data: i32, others: Vec<&Gc<RefCell<Node>>>) -> Gc<RefCell<Node>> {
                    unsafe {
                        NODES_LEFT += 1;
                    }

                    let other = others.iter().map(|e| (*e).clone()).collect();

                    Gc::new(RefCell::new(Self::create(data, other)))
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

            x.get().borrow_mut().other.push(y.clone());

            let _x = x;
            let _y = y;
        }

        assert_eq!(unsafe { NODES_LEFT }, 0);
    }

    #[test]
    fn with_root_circular() {
        setup!();

        let root = setup!(root);
        {
            let x = Gc::new(RefCell::new(Node::new(5)));
            let y = Gc::new(RefCell::new(Node::with_other(6, x.clone())));

            x.get().borrow_mut().other.push(y);

            root.get().borrow_mut().other.push(x);
        }

        assert_eq!(unsafe { NODES_LEFT }, 3); //root, x, y
        {
            root.borrow_mut().unwrap().other.clear();
        }

        assert_eq!(unsafe { NODES_LEFT }, 1); //root (root will never be dropped)
    }

    #[test]
    fn with_root() {
        setup!();

        let root = setup!(root);
        {
            let x = Gc::new(RefCell::new(Node::new(5)));
            let y = Gc::new(RefCell::new(Node::with_other(6, x)));

            root.get().borrow_mut().other.push(y);
        }

        assert_eq!(unsafe { NODES_LEFT }, 3); //root, x, y
        {
            root.borrow_mut().unwrap().other.clear();
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
                root.other.push(x);

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

    #[test]
    #[allow(clippy::similar_names)]
    fn complex() {
        setup!();

        {
            let proto = Node::add(0);

            let func_proto = Node::add_with_other(1, &proto);

            let m1 = Node::add_with_other(8, &func_proto);
            let m2 = Node::add_with_other(8, &func_proto);
            let m3 = Node::add_with_other(8, &func_proto);
            let m4 = Node::add_with_other(8, &func_proto);

            let pr = proto.get();
            let mut pr = pr.borrow_mut();
            pr.other.push(m1);
            pr.other.push(m2);
            pr.other.push(m3);
            pr.other.push(m4);

            {
                let obj_1 = Node::add_with_other(10, &proto);
                let obj_2 = Node::add_with_other(10, &proto);
                let obj_3 = Node::add_with_other(10, &proto);
                let obj_4 = Node::add_with_other(10, &proto);
                let obj_5 = Node::add_with_other(10, &proto);
                let obj_6 = Node::add_with_other(10, &proto);
                let obj_7 = Node::add_with_other(10, &proto);
                let obj_8 = Node::add_with_other(10, &proto);
                let obj_9 = Node::add_with_other(10, &proto);
                let obj_10 = Node::add_with_other(10, &proto);
                let obj_11 = Node::add_with_other(10, &proto);
                let obj_12 = Node::add_with_other(10, &proto);

                let obj_ref_4_5 = Node::add_vec(18, vec![&obj_4, &obj_5, &func_proto]);

                let _a = Node::add_vec(
                    99,
                    vec![
                        &obj_1,
                        &obj_2,
                        &obj_3,
                        &obj_4,
                        &obj_5,
                        &obj_6,
                        &obj_7,
                        &obj_8,
                        &obj_9,
                        &obj_10,
                        &obj_11,
                        &obj_12,
                        &obj_ref_4_5,
                    ],
                );
            }
        }

        assert_eq!(unsafe { NODES_LEFT }, 0);
    }

    #[test]
    fn untyped_circular() {
        setup!(untyped);

        {
            let y = Other::new(1337);
            let y2 = Other::new(1337);
            let y3 = Other::new(1337);
            let y4 = Other::new(1337);

            let x = Node::add_vec_type(42, vec![], vec![&y, &y2, &y3, &y4]);

            dbg!(x);
        }

        assert_eq!(unsafe { NODES_LEFT }, 0);
    }
}

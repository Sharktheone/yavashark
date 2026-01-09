use super::IteratorHelperImpl;
use crate::realm::Intrinsic;
use crate::value::{MutObj, ObjectImpl};
use crate::{MutObject, ObjectHandle, Realm, Res};
use std::any::TypeId;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::fmt::Debug;
use std::ops::Deref;
use std::ptr::NonNull;

#[derive(Debug)]
pub struct IteratorHelperObject<T: IteratorHelperImpl> {
    pub inner: RefCell<MutObject>,
    pub native: T,
}

impl<T: IteratorHelperImpl + Intrinsic> IteratorHelperObject<T> {
    pub fn new(native: T, realm: &mut Realm) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutObject::with_proto(T::get_intrinsic(realm)?)),
            native,
        })
    }
}

impl<T: IteratorHelperImpl + Intrinsic> Intrinsic for IteratorHelperObject<T> {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        T::initialize(realm)
    }

    fn get_intrinsic(realm: &mut Realm) -> Res<ObjectHandle> {
        T::get_intrinsic(realm)
    }

    fn get_global(realm: &mut Realm) -> Res<ObjectHandle> {
        T::get_global(realm)
    }
}

impl<T: IteratorHelperImpl> ObjectImpl for IteratorHelperObject<T> {
    type Inner = MutObject;

    fn get_wrapped_object(&self) -> impl std::ops::DerefMut<Target = impl MutObj> {
        self.inner.borrow_mut()
    }

    fn get_inner(&self) -> impl std::ops::Deref<Target = Self::Inner> {
        self.inner.borrow()
    }

    fn get_inner_mut(&self) -> impl std::ops::DerefMut<Target = Self::Inner> {
        self.inner.borrow_mut()
    }

    /// # Safety
    /// - Caller and implementer must ensure that the pointer is a valid pointer to the type which the type id represents
    /// - Caller and implementer must ensure that the pointer is valid for the same lifetime of self
    unsafe fn inner_downcast(&self, ty: TypeId) -> Option<NonNull<()>> {
        if ty == TypeId::of::<Self>() {
            Some(NonNull::from(self).cast())
        } else if ty == TypeId::of::<T>() {
            Some(NonNull::from(&self.native).cast())
        } else {
            self.get_wrapped_object().inner_downcast(ty)
        }
    }

    /// # Safety
    /// The caller must ensure that the returned pointer is only used for the lifetime of self.
    unsafe fn inner_downcast_fat_ptr(&self, ty: TypeId) -> Option<NonNull<[()]>> {
        if ty == TypeId::of::<dyn IteratorHelperImpl>() {
            // Create a trait object reference
            let trait_ref: &dyn IteratorHelperImpl = &self.native;
            // Transmute the fat pointer to NonNull<[()]> for storage
            // This preserves both the data pointer and vtable pointer
            Some(std::mem::transmute_copy::<&dyn IteratorHelperImpl, NonNull<[()]>>(&trait_ref))
        } else {
            None
        }
    }
}

impl<T: IteratorHelperImpl> Deref for IteratorHelperObject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.native
    }
}

impl<T: IteratorHelperImpl> Borrow<T> for IteratorHelperObject<T> {
    fn borrow(&self) -> &T {
        &self.native
    }
}

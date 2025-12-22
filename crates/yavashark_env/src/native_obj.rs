#![allow(unused)]

use crate::conversion::TryIntoValue;
use crate::inline_props::{InlineObject, PropertiesHook};
use crate::partial_init::Initializer;
use crate::realm::Intrinsic;
use crate::value::{IntoValue, MutObj, Obj, ObjectImpl};
use crate::{MutObject, ObjectHandle, Realm, Res, Value};
use std::any::TypeId;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::fmt::Debug;
use std::ops::Deref;
use std::ptr::NonNull;

#[derive(Debug)]
pub struct NativeObject<T: ?Sized> {
    pub inner: RefCell<MutObject>,
    pub native: T,
}

impl<T: Intrinsic> NativeObject<T> {
    pub fn new(native: T, realm: &mut Realm) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutObject::with_proto(T::get_intrinsic(realm)?)),
            native,
        })
    }
}

impl<T: ?Sized + Intrinsic> Intrinsic for NativeObject<T> {
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

impl<T: ?Sized + Debug + 'static> ObjectImpl for NativeObject<T> {
    type Inner = MutObject;

    fn get_wrapped_object(&self) -> impl std::ops::DerefMut<Target = impl crate::value::MutObj> {
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
}

impl<T: ?Sized + 'static> Deref for NativeObject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.native
    }
}

impl<T: ?Sized + 'static> Borrow<T> for NativeObject<T> {
    fn borrow(&self) -> &T {
        &self.native
    }
}

#![allow(unused)]

use crate::conversion::TryIntoValue;
use crate::inline_props::{InlineObject, PropertiesHook};
use crate::realm::Intrinsic;
use crate::value::{IntoValue, ObjectImpl};
use crate::{MutObject, ObjectHandle, Realm, Res, Value};
use std::cell::RefCell;
use std::fmt::Debug;
use crate::partial_init::Initializer;

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
}
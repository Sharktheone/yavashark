#![allow(unused)]

use std::cell::RefCell;
use crate::{MutObject, ObjectHandle, Realm, Res, Value};

pub struct NativeObject<N: ?Sized> {
    pub inner: RefCell<MutObject>,
    pub native_inner: N,
}

pub trait SetupNativeObj {
    fn get_prototype(realm: &Realm) -> &ObjectHandle;
    fn get_prototype_mut(realm: &mut Realm) -> &mut ObjectHandle;

    fn setup_callback(&self, realm: &mut Realm) {}
}

pub trait NativeObj: SetupNativeObj {
    fn set_property(&self, name: Value, value: Value) -> Res<bool> {
        Ok(false)
    }
}

pub trait DynNativeObj: 'static {
    fn foo(&self) -> String {
        "Default foo implementation".to_string()
    }
}

pub struct Bar;

impl DynNativeObj for Bar {
    fn foo(&self) -> String {
        "Bar's foo implementation".to_string()
    }
}

impl<N: DynNativeObj> NativeObject<N> {
    pub const fn new(inner: MutObject, native_inner: N) -> Self {
        Self {
            inner: RefCell::new(inner),
            native_inner,
        }
    }
}

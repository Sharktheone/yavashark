#![allow(unused)]

use crate::{MutObject, ObjectHandle, Realm, Res, Value};

pub struct NativeObject<N: DynNativeObj + ?Sized> {
    pub inner: MutObject,
    pub native_inner: N,
}

pub trait NativeObj {
    fn prototype(realm: &Realm) -> ObjectHandle;
    fn setup_realm(realm: &mut Realm);

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
            inner,
            native_inner,
        }
    }

    pub fn as_dyn(self) -> Box<DynNativeObject> {
        Box::new(self)
    }
}

pub type DynNativeObject = NativeObject<dyn DynNativeObj>;

// pub struct NativeObject2 {
//     inner: MutObject,
//     native_inner: dyn DynNativeObj,
// }

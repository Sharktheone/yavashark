#![allow(unused)]

use crate::{MutObject, ObjectHandle, Realm, Res, Value};

pub struct NativeObject<N: NativeObj> {
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

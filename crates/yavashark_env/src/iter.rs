use std::cell::RefCell;
use std::fmt::Debug;
use yavashark_macro::object;
use crate::{MutObject, ObjectHandle, Realm, Res, Value};
use crate::conversion::TryIntoValue;

pub trait IntoJsIter {
    fn into_js_iter(self, realm: &mut Realm) -> Res<ObjectHandle>;
    fn into_js_iter_proto(self, proto: ObjectHandle, realm: &mut Realm) -> Res<ObjectHandle>;
}

pub trait JSIterator: 'static {
    fn next(&mut self, realm: &mut Realm) -> Res<Option<Value>>;
}

impl<T: Iterator<Item = I> + 'static, I: TryIntoValue> JSIterator for T {
    fn next(&mut self, realm: &mut Realm) -> Res<Option<Value>> {
        match Iterator::next(self) {
            Some(item) => {
                let value = item.try_into_value(realm)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
}


#[object]
pub struct NativeIterator<I: JSIterator + ?Sized = dyn JSIterator> {
    iter: RefCell<I>,
}


impl<I: JSIterator + 'static> NativeIterator<I> {
    pub fn new(iter: I, proto: ObjectHandle) -> Self {
        Self {
            inner: RefCell::new(MutableNativeIterator {
                object: MutObject::with_proto(proto),
            }),

            iter: RefCell::new(iter),
        }
    }

    pub fn as_dyn(&self) -> &NativeIterator {
        self
    }

    pub fn to_dyn_boxed(self) -> Box<NativeIterator> {
        Box::new(self)
    }

    pub fn to_handle(self) -> ObjectHandle {
        ObjectHandle::from_boxed_obj(self.to_dyn_boxed())
    }
}

impl Debug for NativeIterator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeIterator")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<I: JSIterator> IntoJsIter for I {
    fn into_js_iter(self, realm: &mut Realm) -> Res<ObjectHandle> {
        // let iter_proto = realm.intrinsics.clone_public().iterator.get(realm)?.clone();
        let iter_proto = realm.intrinsics.clone_public().obj.clone();
        self.into_js_iter_proto(iter_proto, realm)
    }

    fn into_js_iter_proto(self, proto: ObjectHandle, _realm: &mut Realm) -> Res<ObjectHandle> {
        let native_iter = NativeIterator::new(self, proto);
        Ok(native_iter.to_handle())
    }
}
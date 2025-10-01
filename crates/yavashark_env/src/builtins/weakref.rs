use crate::value::Obj;
use crate::{MutObject, ObjectHandle, Realm, Value, WeakObjectHandle};
use std::cell::RefCell;
use yavashark_macro::{object, props};

#[object]
#[derive(Debug)]
pub struct WeakRef {
    handle: WeakObjectHandle,
}

impl WeakRef {
    pub fn new(handle: WeakObjectHandle, realm: &Realm) -> Self {
        Self {
            inner: RefCell::new(MutableWeakRef {
                object: MutObject::with_proto(realm.intrinsics.weak_ref.clone()),
            }),
            handle,
        }
    }
}

#[props]
impl WeakRef {
    #[constructor]
    pub fn construct(handle: &ObjectHandle, realm: &Realm) -> ObjectHandle {
        Self::new(handle.downgrade(), realm).into_object()
    }

    pub fn deref(&self) -> Value {
        self.handle
            .upgrade()
            .map(Into::into)
            .unwrap_or(Value::Undefined)
    }
}

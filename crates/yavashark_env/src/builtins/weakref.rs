use crate::value::Obj;
use crate::{MutObject, ObjectHandle, Realm, Res, Value, WeakObjectHandle};
use std::cell::RefCell;
use yavashark_macro::{object, props};

#[object]
#[derive(Debug)]
pub struct WeakRef {
    handle: WeakObjectHandle,
}

impl WeakRef {
    pub fn new(handle: WeakObjectHandle, realm: &mut Realm) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutableWeakRef {
                object: MutObject::with_proto(realm.intrinsics.clone_public().weak_ref.get(realm)?.clone()),
            }),
            handle,
        })
    }
}

#[props(intrinsic_name = weak_ref)]
impl WeakRef {
    #[constructor]
    pub fn construct(handle: &ObjectHandle, realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(Self::new(handle.downgrade(), realm)?.into_object())
    }

    pub fn deref(&self) -> Value {
        self.handle
            .upgrade()
            .map(Into::into)
            .unwrap_or(Value::Undefined)
    }
}

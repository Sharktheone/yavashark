use crate::value::Obj;
use crate::{MutObject, ObjectHandle, Realm, Res, Value, WeakObjectHandle};
use std::cell::RefCell;
use yavashark_macro::{object, props};
use crate::or_symbol::OrSymbol;

#[object]
#[derive(Debug)]
pub struct WeakRef {
    handle: OrSymbol<WeakObjectHandle>,
}

impl WeakRef {
    pub fn new(handle: OrSymbol<WeakObjectHandle>, realm: &mut Realm) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutableWeakRef {
                object: MutObject::with_proto(
                    realm.intrinsics.clone_public().weak_ref.get(realm)?.clone(),
                ),
            }),
            handle,
        })
    }
}

#[props(intrinsic_name = weak_ref)]
impl WeakRef {
    #[constructor]
    pub fn construct(handle: OrSymbol<ObjectHandle>, realm: &mut Realm) -> Res<ObjectHandle> {
        let handle = handle.map(|h| h.downgrade());

        Ok(Self::new(handle, realm)?.into_object())
    }

    pub fn deref(&self) -> Value {
        self.handle
            .as_ref()
            .try_map(|h| h.upgrade().ok_or(()))
            .map(Into::into)
            .unwrap_or(Value::Undefined)
    }
}

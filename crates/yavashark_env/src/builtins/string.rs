use crate::{MutObject, ObjectHandle, Realm, ValueResult};
use std::cell::RefCell;
use yavashark_macro::{object, properties};
use yavashark_value::Obj;

#[object]
#[derive(Debug)]
pub struct StringConstructor {}

impl StringConstructor {
    #[allow(clippy::new_ret_no_self, dead_code)]
    pub fn new(proto: ObjectHandle, _func: ObjectHandle) -> crate::Result<ObjectHandle> {
        let this = Self {
            inner: RefCell::new(MutableStringConstructor {
                object: MutObject::with_proto(proto.into()),
            }),
        };

        Ok(this.into_object())
    }
}

#[properties]
impl StringConstructor {
    #[new]
    pub fn create(realm: &Realm) -> ValueResult {
        Ok(Self::new(realm.intrinsics.obj.clone(), realm.intrinsics.func.clone())?.into())
    }
}

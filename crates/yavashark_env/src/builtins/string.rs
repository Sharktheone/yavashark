use yavashark_macro::{object, properties};
use yavashark_value::Obj;
use crate::{ObjectHandle, Realm, Object, ValueResult};

#[object]
#[derive(Debug)]
pub struct StringConstructor {}




impl StringConstructor {

    #[allow(clippy::new_ret_no_self)]
    pub fn new(proto: ObjectHandle, func: ObjectHandle) -> crate::Result<ObjectHandle> {
        let mut this = Self {
            object: Object::raw_with_proto(proto.into()),
        };


        Ok(this.into_object())
    }
}


#[properties]
impl StringConstructor {
    #[new]
    #[must_use]
    pub fn create(realm: &mut Realm) -> ValueResult {
        Ok(Self::new(realm.intrinsics.obj.clone(), realm.intrinsics.func.clone())?.into())
    }
    
}
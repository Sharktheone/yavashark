use yavashark_macro::{object, properties};
use yavashark_value::Obj;
use crate::{ObjectHandle, Realm, Value, Object};

#[object]
#[derive(Debug)]
pub struct StringConstructor {}




impl StringConstructor {

    #[allow(clippy::new_ret_no_self)]
    pub fn new(proto: ObjectHandle, func: ObjectHandle) -> crate::Result<ObjectHandle> {
        let mut this = Self {
            object: Object::raw_with_proto(proto.into()),
        };

        this.initialize(func.into())?;

        Ok(this.into_object())
    }
}


#[properties]
impl StringConstructor {
    #[new]
    #[must_use]
    pub fn create(realm: &mut Realm) -> Value {
        let this = Self::new(proto.copy());

        ObjectHandle::new(this).into()
    }
    
}
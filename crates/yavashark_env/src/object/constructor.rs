use std::cell::RefCell;
use yavashark_macro::{object, properties_new};
use yavashark_value::Obj;
use crate::{MutObject, Object, ObjectHandle, Result, Value, ValueResult, Variable};

#[object]
#[derive(Debug)]
pub struct ObjectConstructor {}


impl ObjectConstructor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(proto: Value, func: Value) -> Result<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableObjectConstructor {
                object: MutObject::with_proto(proto),
            }),
        };

        this.initialize(func)?;

        Ok(this.into_object())
    }
}

#[properties_new(raw)]
impl ObjectConstructor {
    fn create(proto: ObjectHandle) -> ObjectHandle {
        Object::with_proto(proto.into())
    }

    #[prop("defineProperty")]
    fn define_property(obj: ObjectHandle, key: Value, descriptor: ObjectHandle) -> ValueResult {


        let value = descriptor.get_property(&"value".into()).map(|v| v.value).unwrap_or(Value::Undefined);

        let writable = descriptor.get_property(&"writable".into()).map(|v| v.value.is_truthy()).unwrap_or(false);
        let enumerable = descriptor.get_property(&"enumerable".into()).map(|v| v.value.is_truthy()).unwrap_or(false);
        let configurable = descriptor.get_property(&"configurable".into()).map(|v| v.value.is_truthy()).unwrap_or(false);
        
        
        let var = Variable::new_with_attributes(value, writable, enumerable, configurable);
        
        obj.define_variable(key, var)?;
        
        Ok(obj.into())
    }
}

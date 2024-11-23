use yavashark_macro::{object, properties};
use crate::{Object, ObjectHandle, Value, ValueResult, Error};

#[object]
#[derive(Debug)]
pub struct Math {

}


impl Math {
    pub fn new(proto: ObjectHandle) -> ObjectHandle {
        ObjectHandle::new(Self {
            object: Object::raw_with_proto(proto.into())
        })
    }
}

#[properties]
impl Math {
    #[prop("pow")]
    fn pow(args: Vec<Value>, _: Value) -> ValueResult {
        if args.len() < 2 {
            return Ok(Value::Number(f64::NAN))
        }
        
        let base = args[0].as_number();
        let exponent = args[1].as_number();
        
        Ok(base.powf(exponent).into())
    }

}
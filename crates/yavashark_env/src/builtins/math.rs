use yavashark_macro::{object, properties};
use yavashark_value::Obj;
use crate::{Object, ObjectHandle, Value, ValueResult, Error, Result, Res};

#[object]
#[derive(Debug)]
pub struct Math {

}


impl Math {
    pub fn new(proto: ObjectHandle, func: ObjectHandle) -> Result<ObjectHandle> {
        
        let mut this = Self {
            object: Object::raw_with_proto(proto.clone().into()),
        };

        this.initialize(func.into())?;
        
        Ok(this.into_object())
    }
}

#[properties]
impl Math {
    #[prop("pow")]
    fn pow(&self, args: Vec<Value>, this: Value) -> ValueResult {
        if args.len() < 2 {
            return Ok(Value::Number(f64::NAN))
        }
        
        let k = this.as_object()?.get()?;
        
        
        
        
        let base = args[0].as_number();
        let exponent = args[1].as_number();
        
        Ok(base.powf(exponent).into())
        
    }   
    pub(crate) fn initialize(&mut self, func_proto: Value) -> Res {
        use yavashark_value::{AsAny, Obj};
        let function = crate::NativeFunction::with_proto(stringify!(pow), |args, this, realm| {
            match this.copy() {
                crate::Value::Object(ref x) => {
                    let x = x.get()?;
                    let deez = (***x).as_any().downcast_ref::<Self>().ok_or(Error::ty_error(format!("Function {:?} was not called with a valid this value: {:?}", stringify!(pow), this)))?;
                    deez.pow(args, this)
                }
                _ => Err(Error::ty_error(format!("Function {:?} was not called with a valid this value: {:?}", stringify!(pow), this))),
            }
        }, func_proto.copy()).into();
        self.define_variable(stringify!(pow).into(), crate::Variable::new_with_attributes(function, true, false, false));
        
        Ok(())
    }

}
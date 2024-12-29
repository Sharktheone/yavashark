use crate::{Error, Object, ObjectHandle, Res, Result, Value, ValueResult};
use yavashark_macro::{object, properties, properties_new};
use yavashark_value::Obj;

#[object]
#[derive(Debug)]
pub struct Math {}

impl Math {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(proto: ObjectHandle, func: ObjectHandle) -> Result<ObjectHandle> {
        let mut this = Self {
            object: Object::raw_with_proto(proto.into()),
        };

        this.initialize(func.into())?;

        Ok(this.into_object())
    }
}



#[properties_new(raw)]
impl Math {
    const PI: f64 = std::f64::consts::PI;
    const E: f64 = std::f64::consts::E;
    
    fn pow(left: f64, right: f64) -> f64 {
        left.powf(right)
    }
    
}

// #[properties]
// impl Math {
//     #[prop("pow")]
//     #[allow(clippy::unused_self, clippy::needless_pass_by_value)]
//     fn pow(&self, args: Vec<Value>, _: Value) -> ValueResult {
//         if args.len() < 2 {
//             return Ok(Value::Number(f64::NAN));
//         }
// 
//         let base = args[0].as_number();
//         let exponent = args[1].as_number();
// 
//         Ok(base.powf(exponent).into())
//     }
//     pub(crate) fn initialize(&mut self, func_proto: Value) -> Res {
//         use yavashark_value::Obj;
//         let function = crate::NativeFunction::with_proto(
//             stringify!(pow),
//             |args, this, _| match this.copy() {
//                 Value::Object(ref x) => {
//                     let x = x.get()?;
//                     let deez = (***x)
//                         .as_any()
//                         .downcast_ref::<Self>()
//                         .ok_or(Error::ty_error(format!(
//                             "Function {:?} was not called with a valid this value: {:?}",
//                             stringify!(pow),
//                             this
//                         )))?;
//                     deez.pow(args, this)
//                 }
//                 _ => Err(Error::ty_error(format!(
//                     "Function {:?} was not called with a valid this value: {:?}",
//                     stringify!(pow),
//                     this
//                 ))),
//             },
//             func_proto,
//         )
//         .into();
//         self.define_variable(
//             stringify!(pow).into(),
//             crate::Variable::new_with_attributes(function, true, false, false),
//         );
// 
//         Ok(())
//     }
// }

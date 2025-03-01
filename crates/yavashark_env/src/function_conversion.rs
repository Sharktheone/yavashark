use crate::{Realm, RuntimeResult, Value};

pub type Callback = Box<dyn Fn(Vec<Value>, Value, &mut Realm) -> RuntimeResult>;

#[allow(unused)]
pub struct NativeFunctionWrapper {
    args: usize,
    func: Callback,
}


impl<F: Fn(i32) + 'static> From<F> for NativeFunctionWrapper {
    fn from(func: F) -> Self {
        Self {
            args: 0,
            func: Box::new(move |_, _, _| {
                func(0);
                Ok(Value::Undefined)
            }),
        }
    }
}

// impl<F: Fn(Value) + 'static> From<F> for NativeFunctionWrapper {
//     fn from(func: F) -> Self {
//         Self {
//             args: 1,
//             func: Box::new(move |args, _, _| {
//                 func(args[0].clone());
//                 Ok(Value::Undefined)
//             }),
//         }
//     }
// }
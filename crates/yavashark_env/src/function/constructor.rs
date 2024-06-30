use std::fmt;
use std::fmt::{Debug, Formatter};
use yavashark_macro::object;
use yavashark_value::{Constructor, Error, Func};
use crate::{Context, Value, ValueResult};
use crate::function::NativeFn;

#[object(function, constructor)]
pub struct NativeConstructor {
    /// The name of the constructor
    pub name: String,
    /// The function that is called when the constructor is called
    pub f: NativeFn,
    /// The function that returns the constructor value
    pub f_value: Box<dyn Fn(&mut Context, &Value) -> Value>,
    #[gc]
    /// The prototype of the constructor
    pub proto: Value,
    /// Can this constructor be called without `new`?
    pub special: bool,
}


impl Debug for NativeConstructor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "NativeConstructor({})", self.name)
    }
}



impl Constructor<Context> for NativeConstructor {
    fn get_constructor(&self) -> Value {
        Value::Object(self.clone().into())
    }

    fn special_constructor(&self) -> bool {
        self.special
    }

    fn get_constructor_value(&self, ctx: &mut Context) -> Option<Value> {
        self.f_value(ctx, &self.proto)
    }

    fn get_constructor_proto(&self, ctx: &mut Context) -> Option<Value> {
        Some(self.proto.clone())
    }
}



impl Func<Context> for NativeConstructor {
    fn call(&mut self, ctx: &mut Context, args: Vec<Value>, this: Value) -> ValueResult {
        if self.special {
            self.f(ctx, args, this)
        } else {
            Err(Error::ty_error(format!("Constructor {} requires 'new'", self.name)))
        }
    }
}
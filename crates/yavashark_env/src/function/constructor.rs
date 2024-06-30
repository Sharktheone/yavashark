use std::fmt;
use std::fmt::{Debug, Formatter};
use yavashark_macro::object;
use yavashark_value::{Constructor, Error, Func};
use crate::{Context, Value, ValueResult};

#[object(function, constructor)]
pub struct NativeConstructor {
    /// The name of the constructor
    pub name: String,
    /// The function that is called when the constructor is called
    pub f: Box<dyn Fn(&Value) -> Value>,
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
        (self.f)(&self.proto)
    }

    fn special_constructor(&self) -> bool {
        self.special
    }

    fn value(&self, ctx: &mut Context) -> Value {
        (self.f_value)(ctx, &self.proto)
    }

    fn proto(&self, _ctx: &mut Context) -> Value {
        self.proto.clone()
    }
}


impl Func<Context> for NativeConstructor {
    fn call(&mut self, ctx: &mut Context, args: Vec<Value>, _this: Value) -> ValueResult {
        if self.special {
            let value = (self.f_value)(ctx, &self.proto);
            
            (self.f)(&self.proto).call(ctx, args, value.copy())?;
            
            Ok(value)
        } else {
            Err(Error::ty_error(format!("Constructor {} requires 'new'", self.name)))
        }
    }
}
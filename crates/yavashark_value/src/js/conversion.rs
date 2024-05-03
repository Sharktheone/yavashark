use std::cell::RefCell;
use std::rc::Rc;

use crate::{Ctx, Func, Function, Object, Value};

impl<C: Ctx> From<&str> for Value<C> {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl<C: Ctx> From<String> for Value<C> {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl<C: Ctx> From<&String> for Value<C> {
    fn from(s: &String) -> Self {
        Value::String(s.clone())
    }
}

impl<C: Ctx> From<()> for Value<C> {
    fn from(_: ()) -> Self {
        Value::Undefined
    }
}

impl<C: Ctx> From<f64> for Value<C> {
    fn from(n: f64) -> Self {
        Value::Number(n)
    }
}

impl<C: Ctx> From<bool> for Value<C> {
    fn from(b: bool) -> Self {
        Value::Boolean(b)
    }
}

impl<C: Ctx> From<u8> for Value<C> {
    fn from(n: u8) -> Self {
        Value::Number(n as f64)
    }
}

impl<C: Ctx> From<u16> for Value<C> {
    fn from(n: u16) -> Self {
        Value::Number(n as f64)
    }
}

impl<C: Ctx> From<u32> for Value<C> {
    fn from(n: u32) -> Self {
        Value::Number(n as f64)
    }
}

impl<C: Ctx> From<u64> for Value<C> {
    fn from(n: u64) -> Self {
        Value::Number(n as f64)
    }
}

impl<C: Ctx> From<i8> for Value<C> {
    fn from(n: i8) -> Self {
        Value::Number(n as f64)
    }
}

impl<C: Ctx> From<i16> for Value<C> {
    fn from(n: i16) -> Self {
        Value::Number(n as f64)
    }
}

impl<C: Ctx> From<i32> for Value<C> {
    fn from(n: i32) -> Self {
        Value::Number(n as f64)
    }
}

impl<C: Ctx> From<i64> for Value<C> {
    fn from(n: i64) -> Self {
        Value::Number(n as f64)
    }
}

impl<C: Ctx> From<usize> for Value<C> {
    fn from(n: usize) -> Self {
        Value::Number(n as f64)
    }
}

impl<C: Ctx> From<isize> for Value<C> {
    fn from(n: isize) -> Self {
        Value::Number(n as f64)
    }
}

impl<C: Ctx> From<f32> for Value<C> {
    fn from(n: f32) -> Self {
        Value::Number(n as f64)
    }
}

impl<O: Into<Object<C>>, C: Ctx> From<O> for Value<C> {
    fn from(o: O) -> Self {
        Value::Object(o.into())
    }
}

impl<C: Ctx> From<Function<C>> for Value<C> {
    fn from(f: Function<C>) -> Self {
        Value::Function(f)
    }
}

impl<C: Ctx> From<Rc<RefCell<Box<dyn Func<C>>>>> for Value<C> {
    fn from(f: Rc<RefCell<Box<dyn Func<C>>>>) -> Self {
        Value::Function(Function::from(f))
    }
}

impl<C: Ctx> From<Box<dyn Func<C>>> for Value<C> {
    fn from(f: Box<dyn Func<C>>) -> Self {
        Value::Function(Function::from(f))
    }
}

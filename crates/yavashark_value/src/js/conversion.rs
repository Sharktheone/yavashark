use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use crate::error::{Error, StackTrace};
use crate::object::Object;
use crate::Value;
use crate::Func;

impl<T: Func> From<&str> for Value<T> {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl<T: Func> From<String> for Value<T> {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl<T: Func> From<&String> for Value<T> {
    fn from(s: &String) -> Self {
        Value::String(s.clone())
    }
}


impl<T: Func> From<()> for Value<T> {
    fn from(_: ()) -> Self {
        Value::Undefined
    }
}

impl<T: Func> From<f64> for Value<T> {
    fn from(n: f64) -> Self {
        Value::Number(n)
    }
}

impl<T: Func> From<bool> for Value<T> {
    fn from(b: bool) -> Self {
        Value::Boolean(b)
    }
}

impl<T: Func> From<Rc<RefCell<Object<T>>>> for Value<T> {
    fn from(o: Rc<RefCell<Object<T>>>) -> Self {
        Value::Object(o)
    }
}

impl<T: Func> From<Object<T>> for Value<T> {
    fn from(o: Object<T>) -> Self {
        Value::Object(Rc::new(RefCell::new(o)))
    }
}

impl<T: Func> From<u8> for Value<T> {
    fn from(n: u8) -> Self {
        Value::Number(n as f64)
    }
}

impl<T: Func> From<u16> for Value<T> {
    fn from(n: u16) -> Self {
        Value::Number(n as f64)
    }
}

impl<T: Func> From<u32> for Value<T> {
    fn from(n: u32) -> Self {
        Value::Number(n as f64)
    }
}

impl<T: Func> From<u64> for Value<T> {
    fn from(n: u64) -> Self {
        Value::Number(n as f64)
    }
}

impl<T: Func> From<i8> for Value<T> {
    fn from(n: i8) -> Self {
        Value::Number(n as f64)
    }
}

impl<T: Func> From<i16> for Value<T> {
    fn from(n: i16) -> Self {
        Value::Number(n as f64)
    }
}

impl<T: Func> From<i32> for Value<T> {
    fn from(n: i32) -> Self {
        Value::Number(n as f64)
    }
}

impl<T: Func> From<i64> for Value<T> {
    fn from(n: i64) -> Self {
        Value::Number(n as f64)
    }
}

impl<T: Func> From<usize> for Value<T> {
    fn from(n: usize) -> Self {
        Value::Number(n as f64)
    }
}

impl<T: Func> From<isize> for Value<T> {
    fn from(n: isize) -> Self {
        Value::Number(n as f64)
    }
}

impl<T: Func> From<f32> for Value<T> {
    fn from(n: f32) -> Self {
        Value::Number(n as f64)
    }
}


impl<T: Func> From<Error<T>> for Value<T> {
    fn from(e: Error<T>) -> Self {
        Value::Object(Rc::new(RefCell::new(e.into())))
    }
}

impl<T: Func> From<Error<T>> for Object<T> {
    fn from(e: Error<T>) -> Self {
        let mut obj = Object::new();

        obj.define_property("message".to_string(), e.message().into());
        obj.define_property("stack".to_string(), e.stack().into());
        obj.define_property("name".to_string(), e.name().into());
        obj.define_property("fileName".to_string(), e.file_name().into());
        obj.define_property("lineNumber".to_string(), e.line_number().into());
        obj.define_property("columnNumber".to_string(), e.column_number().into());

        obj
    }
}


impl<T: Func> From<&StackTrace>  for Value<T> {
    fn from(s: &StackTrace) -> Self {
        Value::String(format!("{:#?}", s)) //TODO: Implement a better way to convert stack traces
    }
}


impl<T: Func> From<T> for Object<T> {
    fn from(t: T) -> Self {
        let mut obj = Object::new();
        obj.call = Some(t);
        obj
    }
}

impl<T: Func> From<T> for Value<T> {
    fn from(o: T) -> Self {
        Value::Object(Rc::new(RefCell::new(o.into())))
    }
}


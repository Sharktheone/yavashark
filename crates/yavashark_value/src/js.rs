use crate::object::Object;
use std::cell::RefCell;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

pub mod object;
mod ops;

#[derive(Debug, PartialEq)]
pub enum Value<F: Debug> {
    Null,
    Undefined,
    Number(f64),
    String(String),
    Boolean(bool),
    //TODO: This can create cyclic references: we need a GC like thing for that
    Object(Rc<RefCell<Object<F>>>),
}

impl<F: Debug> Value<F> {
    pub fn copy(&self) -> Self {
        match self {
            Value::Null => Value::Null,
            Value::Undefined => Value::Undefined,
            Value::Number(n) => Value::Number(*n),
            Value::String(s) => Value::String(s.clone()),
            Value::Boolean(b) => Value::Boolean(*b),
            Value::Object(o) => Value::Object(Rc::clone(o)),
        }
    }
}

impl<F: Debug> Value<F> {
    pub fn type_of(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Undefined => "undefined",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Boolean(_) => "boolean",
            Value::Object(_) => "object",
        }
    }
}

impl<F: Debug> Display for Value<F> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Undefined => write!(f, "undefined"),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Object(o) => write!(f, "[object Object]"),
        }
    }
}

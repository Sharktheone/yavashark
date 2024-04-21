use crate::object::Object;

pub mod object;

pub enum Value {
    Null,
    Undefined,
    Number(f64),
    String(String),
    Boolean(bool),
    Object(Object),
}
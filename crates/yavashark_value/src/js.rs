use std::ops::{Add, Shl, Rem};
use crate::object::Object;

pub mod object;
mod ops;


#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Undefined,
    Number(f64),
    String(String),
    Boolean(bool),
    Object(Object),
}
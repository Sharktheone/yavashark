use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

pub use context::*;
pub use function::*;
pub use object::*;
pub use symbol::*;

use crate::Error;

mod context;
mod conversion;
mod function;
mod object;
mod ops;
mod symbol;

#[derive(Debug, PartialEq)]
pub enum Value<C: Ctx> {
    Null,
    Undefined,
    Number(f64),
    String(String),
    Boolean(bool),
    //TODO: This can create cyclic references: we need a GC like thing for that
    Object(Object<C>),
    Function(Function<C>),
    Symbol(String),
}

impl<C: Ctx> Eq for Value<C> {}

#[derive(Debug, Hash)]
pub enum Type {
    Null,
    Undefined,
    Number,
    String,
    Boolean,
    Object,
    Function,
    Symbol,
}

impl<C: Ctx> Hash for Value<C> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        //Hash the type and the value to minimize collisions
        match self {
            Value::Null => Type::Null.hash(state),
            Value::Undefined => Type::Undefined.hash(state),
            Value::Number(n) => (Type::Number, n.to_bits()).hash(state),
            Value::String(s) => (Type::String, s).hash(state),
            Value::Boolean(b) => (Type::Boolean, b).hash(state),
            Value::Object(o) => (Type::Object, o).hash(state),
            Value::Function(f) => (Type::Function, f).hash(state),
            Value::Symbol(s) => (Type::Symbol, s).hash(state),
        }
    }
}

impl<C: Ctx> Value<C> {
    pub fn copy(&self) -> Self {
        match self {
            Value::Null => Value::Null,
            Value::Undefined => Value::Undefined,
            Value::Number(n) => Value::Number(*n),
            Value::String(s) => Value::String(s.clone()),
            Value::Boolean(b) => Value::Boolean(*b),
            Value::Object(o) => Value::Object(Object::clone(o)),
            Value::Function(f) => Value::Function(Function::clone(f)),
            Value::Symbol(s) => Value::Symbol(s.clone()),
        }
    }

    pub fn is_nan(&self) -> bool {
        match self {
            Value::Number(n) => n.is_nan(),
            _ => false,
        }
    }

    pub fn is_falsey(&self) -> bool {
        match self {
            Value::Null | Value::Undefined => true,
            Value::Number(n) => n == &0.0,
            Value::String(s) => s.is_empty(),
            Value::Boolean(b) => !b,
            Value::Object(_) => false,
            Value::Function(_) => false,
            Value::Symbol(_) => false,
        }
    }

    pub fn type_of(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Undefined => "undefined",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Boolean(_) => "boolean",
            Value::Object(_) => "object",
            Value::Function(_) => "function",
            Value::Symbol(_) => "symbol",
        }
    }
}

impl<C: Ctx> Display for Value<C> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Undefined => write!(f, "undefined"),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Object(o) => write!(f, "{}", o),
            Value::Function(func) => write!(f, "{}", func),
            Value::Symbol(s) => write!(f, "Symbol({})", s),
        }
    }
}


impl<C: Ctx> Value<C> {
    // pub fn iter(&self) -> Result<Iter<C>, Error<C>> {
    //    todo!() 
    // }
    
    pub fn get_property(&self, name: &Value<C>) -> Result<Value<C>, Error<C>> {
        match self {
            Value::Object(o) => o.get_property(name),
            Value::Function(f) => f.get_property(name),
            _ => Err(Error::ty("Value is not an object")),
        }
    }
    
    pub fn update_or_define_property(&self, name: Value<C>, value: Value<C>) -> Result<(), Error<C>> {
        match self {
            Value::Object(o) => o.update_or_define_property(name, value),
            Value::Function(f) => f.update_or_define_property(name, value),
            _ => Err(Error::ty("Value is not an object")),
        }
    }
    
    pub fn define_property(&self, name: Value<C>, value: Value<C>) -> Result<(), Error<C>> {
        match self {
            Value::Object(o) => o.define_property(name, value),
            Value::Function(f) => f.define_property(name, value),
            _ => Err(Error::ty("Value is not an object")),
        }
    }
    
    pub fn contains_key(&self, name: &Value<C>) -> Result<bool, Error<C>> {
        match self {
            Value::Object(o) => o.contains_key(name),
            Value::Function(f) => f.contains_key(name),
            _ => Err(Error::ty("Value is not an object")),
        }
    }
    

}

// struct Iter'a, C: Ctx> {
//     value: &'a Value<C>,
//     index: usize,
// }
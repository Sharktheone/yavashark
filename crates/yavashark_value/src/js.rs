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
pub mod variable;

#[derive(Debug, PartialEq, Hash, Clone)]
pub enum ConstString {
    String(&'static str),
    Owned(String),
}

impl Display for ConstString {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ConstString::String(s) => write!(f, "{}", s)?,
            ConstString::Owned(s) => write!(f, "{}", s)?,
        }

        Ok(())
    }
}

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
    Symbol(ConstString),
}

impl<C: Ctx> Clone for Value<C> {
    fn clone(&self) -> Self {
        self.copy()
    }
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

    pub const fn symbol(name: &'static str) -> Self {
        Value::Symbol(ConstString::String(name))
    }

    pub fn string(s: &str) -> Self {
        Value::String(s.to_string())
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

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null | Value::Undefined => false,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Boolean(b) => *b,
            Value::Object(_) => true,
            Value::Function(_) => true,
            Value::Symbol(_) => true,
        }
    }

    pub fn is_nullish(&self) -> bool {
        matches!(self, Value::Null | Value::Undefined)
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
    pub fn iter<'a>(&self, ctx: &'a mut C) -> Result<CtxIter<'a, C>, Error<C>> {
        let iter = self
            .get_property(&Symbol::ITERATOR)
            .map_err(|_| Error::ty("Result of the Symbol.iterator method is not an object"))?;
        let iter = iter.call(ctx, Vec::new(), self.copy())?;

        match iter {
            Value::Function(f) => Ok(CtxIter { next: f, ctx }),
            _ => Err(Error::ty("Value is not a function")),
        }
    }

    pub fn iter_no_ctx(&self, ctx: &mut C) -> Result<Iter<C>, Error<C>> {
        let iter = self
            .get_property(&Symbol::ITERATOR)
            .map_err(|_| Error::ty("Result of the Symbol.iterator method is not an object"))?;
        let iter = iter.call(ctx, Vec::new(), self.copy())?;

        match iter {
            Value::Function(f) => Ok(Iter { next: f }),
            _ => Err(Error::ty("Value is not a function")),
        }
    }

    pub fn get_property(&self, name: &Value<C>) -> Result<Value<C>, Error<C>> {
        match self {
            Value::Object(o) => o.get_property(name),
            Value::Function(f) => f.get_property(name),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn update_or_define_property(
        &self,
        name: Value<C>,
        value: Value<C>,
    ) -> Result<(), Error<C>> {
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

    pub fn call(
        &self,
        ctx: &mut C,
        args: Vec<Value<C>>,
        this: Value<C>,
    ) -> Result<Value<C>, Error<C>> {
        match self {
            Value::Function(f) => f.call(ctx, args, this),
            _ => Err(Error::ty("Value is not a function")),
        }
    }

    #[allow(clippy::type_complexity)]
    ///(name, value)
    pub fn properties(&self) -> Result<Vec<(Value<C>, Value<C>)>, Error<C>> {
        match self {
            Value::Object(o) => o.properties(),
            Value::Function(f) => f.properties(),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn keys(&self) -> Result<Vec<Value<C>>, Error<C>> {
        match self {
            Value::Object(o) => o.keys(),
            Value::Function(f) => f.keys(),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn values(&self) -> Result<Vec<Value<C>>, Error<C>> {
        match self {
            Value::Object(o) => o.values(),
            Value::Function(f) => f.values(),
            _ => Err(Error::ty("Value is not an object")),
        }
    }
}

pub struct Iter<C: Ctx> {
    next: Function<C>,
}

pub struct CtxIter<'a, C: Ctx> {
    next: Function<C>,
    ctx: &'a mut C,
}

impl<C: Ctx> Iterator for CtxIter<'_, C> {
    type Item = Result<Value<C>, Error<C>>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next.call(self.ctx, Vec::new(), Value::Undefined);
        let next = match next {
            Ok(next) => next,
            Err(e) => return Some(Err(e)),
        };

        let done = next.get_property(&Value::string("done"));

        let done = match done {
            Ok(done) => done.is_truthy(),
            Err(e) => return Some(Err(e)),
        };

        if done {
            return None;
        }

        Some(next.get_property(&Value::string("value")))
    }
}

impl<C: Ctx> Iter<C> {
    pub fn next(&self, ctx: &mut C) -> Result<Option<Value<C>>, Error<C>> {
        let next = self.next.call(ctx, Vec::new(), Value::Undefined)?;
        let done = next.get_property(&Value::string("done"))?;
        if done.is_truthy() {
            return Ok(None);
        }
        next.get_property(&Value::string("value")).map(Some)
    }
}

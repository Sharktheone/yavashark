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

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ConstString {
    String(&'static str),
    Owned(String),
}

impl Display for ConstString {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(s) => write!(f, "{s}")?,
            Self::Owned(s) => write!(f, "{s}")?,
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
            Self::Null => Type::Null.hash(state),
            Self::Undefined => Type::Undefined.hash(state),
            Self::Number(n) => (Type::Number, n.to_bits()).hash(state),
            Self::String(s) => (Type::String, s).hash(state),
            Self::Boolean(b) => (Type::Boolean, b).hash(state),
            Self::Object(o) => (Type::Object, o).hash(state),
            Self::Function(f) => (Type::Function, f).hash(state),
            Self::Symbol(s) => (Type::Symbol, s).hash(state),
        }
    }
}

impl<C: Ctx> Value<C> {
    #[must_use] pub fn copy(&self) -> Self {
        match self {
            Self::Null => Self::Null,
            Self::Undefined => Self::Undefined,
            Self::Number(n) => Self::Number(*n),
            Self::String(s) => Self::String(s.clone()),
            Self::Boolean(b) => Self::Boolean(*b),
            Self::Object(o) => Self::Object(Object::clone(o)),
            Self::Function(f) => Self::Function(Function::clone(f)),
            Self::Symbol(s) => Self::Symbol(s.clone()),
        }
    }

    #[must_use] pub const fn symbol(name: &'static str) -> Self {
        Self::Symbol(ConstString::String(name))
    }

    #[must_use] pub fn string(s: &str) -> Self {
        Self::String(s.to_string())
    }

    #[must_use] pub fn is_nan(&self) -> bool {
        match self {
            Self::Number(n) => n.is_nan(),
            _ => false,
        }
    }

    #[must_use] pub fn is_falsey(&self) -> bool {
        match self {
            Self::Null | Self::Undefined => true,
            Self::Number(n) => *n == 0.0,
            Self::String(s) => s.is_empty(),
            Self::Boolean(b) => !b,
            Self::Object(_)
            | Self::Function(_)
            | Self::Symbol(_) => false,
        }
    }

    #[must_use] pub fn is_truthy(&self) -> bool {
        match self {
            Self::Null | Self::Undefined => false,
            Self::Number(n) => *n != 0.0,
            Self::String(s) => !s.is_empty(),
            Self::Boolean(b) => *b,
            Self::Object(_)
            | Self::Function(_)
            | Self::Symbol(_) => true,
        }
    }

    #[must_use] pub const fn is_nullish(&self) -> bool {
        matches!(self, Self::Null | Self::Undefined)
    }

    #[must_use] pub const fn type_of(&self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Undefined => "undefined",
            Self::Number(_) => "number",
            Self::String(_) => "string",
            Self::Boolean(_) => "boolean",
            Self::Object(_) => "object",
            Self::Function(_) => "function",
            Self::Symbol(_) => "symbol",
        }
    }
}

impl<C: Ctx> Display for Value<C> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Undefined => write!(f, "undefined"),
            Self::Number(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "{s}"),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::Object(o) => write!(f, "{o}"),
            Self::Function(func) => write!(f, "{func}"),
            Self::Symbol(s) => write!(f, "Symbol({s})"),
        }
    }
}

impl<C: Ctx> Value<C> {
    #[allow(clippy::iter_not_returning_iterator)]
    pub fn iter<'a>(&self, ctx: &'a mut C) -> Result<CtxIter<'a, C>, Error<C>> {
        let iter = self
            .get_property(&Symbol::ITERATOR)
            .map_err(|_| Error::ty("Result of the Symbol.iterator method is not an object"))?;
        let iter = iter.call(ctx, Vec::new(), self.copy())?;

        match iter {
            Self::Function(f) => Ok(CtxIter { next: f, ctx }),
            _ => Err(Error::ty("Value is not a function")),
        }
    }

    pub fn iter_no_ctx(&self, ctx: &mut C) -> Result<Iter<C>, Error<C>> {
        let iter = self
            .get_property(&Symbol::ITERATOR)
            .map_err(|_| Error::ty("Result of the Symbol.iterator method is not an object"))?;
        let iter = iter.call(ctx, Vec::new(), self.copy())?;

        match iter {
            Self::Function(f) => Ok(Iter { next: f }),
            _ => Err(Error::ty("Value is not a function")),
        }
    }

    pub fn get_property(&self, name: &Self) -> Result<Self, Error<C>> {
        match self {
            Self::Object(o) => o.get_property(name),
            Self::Function(f) => f.get_property(name),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn update_or_define_property(
        &self,
        name: Self,
        value: Self,
    ) -> Result<(), Error<C>> {
        match self {
            Self::Object(o) => o.update_or_define_property(name, value),
            Self::Function(f) => f.update_or_define_property(name, value),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn define_property(&self, name: Self, value: Self) -> Result<(), Error<C>> {
        match self {
            Self::Object(o) => o.define_property(name, value),
            Self::Function(f) => f.define_property(name, value),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn contains_key(&self, name: &Self) -> Result<bool, Error<C>> {
        match self {
            Self::Object(o) => o.contains_key(name),
            Self::Function(f) => f.contains_key(name),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn call(
        &self,
        ctx: &mut C,
        args: Vec<Self>,
        this: Self,
    ) -> Result<Self, Error<C>> {
        match self {
            Self::Function(f) => f.call(ctx, args, this),
            _ => Err(Error::ty("Value is not a function")),
        }
    }

    #[allow(clippy::type_complexity)]
    ///(name, value)
    pub fn properties(&self) -> Result<Vec<(Self, Self)>, Error<C>> {
        match self {
            Self::Object(o) => o.properties(),
            Self::Function(f) => f.properties(),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn keys(&self) -> Result<Vec<Self>, Error<C>> {
        match self {
            Self::Object(o) => o.keys(),
            Self::Function(f) => f.keys(),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn values(&self) -> Result<Vec<Self>, Error<C>> {
        match self {
            Self::Object(o) => o.values(),
            Self::Function(f) => f.values(),
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

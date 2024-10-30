use std::cell::RefCell;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

pub use constructor::*;
pub use context::*;
pub use function::*;
pub use name::*;
pub use object::*;
pub use symbol::*;
pub use variable::*;
use yavashark_garbage::{Collectable, GcRef};

use crate::Error;

mod constructor;
mod context;
mod conversion;
mod function;
mod name;
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
pub enum Value<C: Realm> {
    Null,
    Undefined,
    Number(f64),
    String(String),
    Boolean(bool),
    Object(Object<C>),
    Symbol(ConstString),
}

impl<C: Realm> Clone for Value<C> {
    fn clone(&self) -> Self {
        self.copy()
    }
}

impl<C: Realm> Eq for Value<C> {}

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

impl<C: Realm> Hash for Value<C> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        //Hash the type and the value to minimize collisions
        match self {
            Self::Null => Type::Null.hash(state),
            Self::Undefined => Type::Undefined.hash(state),
            Self::Number(n) => (Type::Number, n.to_bits()).hash(state),
            Self::String(s) => (Type::String, s).hash(state),
            Self::Boolean(b) => (Type::Boolean, b).hash(state),
            Self::Object(o) => (Type::Object, o).hash(state),
            Self::Symbol(s) => (Type::Symbol, s).hash(state),
        }
    }
}

impl<C: Realm> Value<C> {
    #[must_use]
    pub fn copy(&self) -> Self {
        match self {
            Self::Null => Self::Null,
            Self::Undefined => Self::Undefined,
            Self::Number(n) => Self::Number(*n),
            Self::String(s) => Self::String(s.clone()),
            Self::Boolean(b) => Self::Boolean(*b),
            Self::Object(o) => Self::Object(Object::clone(o)),
            Self::Symbol(s) => Self::Symbol(s.clone()),
        }
    }

    #[must_use]
    pub const fn symbol(name: &'static str) -> Self {
        Self::Symbol(ConstString::String(name))
    }

    #[must_use]
    pub fn string(s: &str) -> Self {
        Self::String(s.to_string())
    }

    #[must_use]
    pub fn is_nan(&self) -> bool {
        match self {
            Self::Number(n) => n.is_nan(),
            _ => false,
        }
    }

    #[must_use]
    pub fn is_falsey(&self) -> bool {
        match self {
            Self::Null | Self::Undefined => true,
            Self::Number(n) => *n == 0.0,
            Self::String(s) => s.is_empty(),
            Self::Boolean(b) => !b,
            Self::Object(_) | Self::Symbol(_) => false,
        }
    }

    #[must_use]
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Null | Self::Undefined => false,
            Self::Number(n) => *n != 0.0,
            Self::String(s) => !s.is_empty(),
            Self::Boolean(b) => *b,
            Self::Object(_) | Self::Symbol(_) => true,
        }
    }

    #[must_use]
    pub const fn is_nullish(&self) -> bool {
        matches!(self, Self::Null | Self::Undefined)
    }

    #[must_use]
    pub const fn type_of(&self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Undefined => "undefined",
            Self::Number(_) => "number",
            Self::String(_) => "string",
            Self::Boolean(_) => "boolean",
            Self::Object(_) => "object",
            Self::Symbol(_) => "symbol",
        }
    }

    #[must_use]
    pub fn gc_ref(&self) -> Option<GcRef<RefCell<BoxedObj<C>>>> {
        match self {
            Self::Object(o) => Some(o.gc_get_ref()),
            _ => None,
        }
    }

    pub fn prototype(&self, realm: &mut C) -> Result<Self, Error<C>> {
        let obj = self.as_object()?;

        let obj = obj.get()?;

        let proto = obj.prototype();

        drop(obj);

        proto.resolve(self.copy(), realm)
    }

    pub fn as_object(&self) -> Result<&Object<C>, Error<C>> {
        let Self::Object(obj) = &self else {
            return Err(Error::ty("expected object"));
        };

        Ok(obj)
    }
}

#[cfg(any(test, debug_assertions, feature = "display_object"))]
impl<C: Realm> Display for Value<C> {
    /// This function shouldn't be used in production code, only for debugging
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Undefined => write!(f, "undefined"),
            Self::Number(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "{s}"),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::Object(o) => write!(f, "{o}"),
            Self::Symbol(s) => write!(f, "Symbol({s})"),
        }
    }
}

impl<C: Realm> CustomGcRefUntyped for Value<C> {
    fn gc_untyped_ref<U: Collectable>(&self) -> Option<GcRef<U>> {
        match self {
            Self::Object(o) => Some(o.gc_get_untyped_ref()),
            _ => None,
        }
    }
}

impl<C: Realm> Value<C> {
    #[allow(clippy::iter_not_returning_iterator)]
    pub fn iter<'a>(&self, realm: &'a mut C) -> Result<CtxIter<'a, C>, Error<C>> {
        let iter = self
            .get_property(&Symbol::ITERATOR, realm)
            .map_err(|_| Error::ty("Result of the Symbol.iterator method is not an object"))?;
        let iter = iter.call(realm, Vec::new(), self.copy())?;

        Ok(CtxIter {
            next_obj: iter,
            realm,
        })
    }

    pub fn iter_no_realm(&self, realm: &mut C) -> Result<Iter<C>, Error<C>> {
        let iter = self
            .get_property(&Symbol::ITERATOR, realm)
            .map_err(|_| Error::ty("Result of the Symbol.iterator method is not an object"))?;
        let iter = iter.call(realm, Vec::new(), self.copy())?;

        Ok(Iter { next_obj: iter })
    }

    pub fn get_property(&self, name: &Self, realm: &mut C) -> Result<Self, Error<C>> {
        match self {
            Self::Object(o) => o
                .resolve_property(name, realm)?
                .ok_or(Error::reference_error(format!(
                    "{name} does not exist on object"
                ))),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn get_property_no_get_set(&self, name: &Self) -> Result<ObjectProperty<C>, Error<C>> {
        match self {
            Self::Object(o) => o
                .resolve_property_no_get_set(name)?
                .ok_or(Error::reference_error(format!(
                    "{name} does not exist on object"
                ))),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn define_property(&self, name: Self, value: Self) -> Result<(), Error<C>> {
        match self {
            Self::Object(o) => o.define_property(name, value),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn contains_key(&self, name: &Self) -> Result<bool, Error<C>> {
        match self {
            Self::Object(o) => o.contains_key(name),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn call(&self, realm: &mut C, args: Vec<Self>, this: Self) -> Result<Self, Error<C>> {
        match self {
            Self::Object(o) => o.call(realm, args, this),
            _ => Err(Error::ty("Value is not a function")),
        }
    }

    pub fn call_method(
        &self,
        name: &Self,
        realm: &mut C,
        args: Vec<Self>,
    ) -> Result<Self, Error<C>> {
        let method = self.get_property(name, realm)?;

        method.call(realm, args, self.copy())
    }

    #[allow(clippy::type_complexity)]
    ///(name, value)
    pub fn properties(&self) -> Result<Vec<(Self, Self)>, Error<C>> {
        match self {
            Self::Object(o) => o.properties(),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn keys(&self) -> Result<Vec<Self>, Error<C>> {
        match self {
            Self::Object(o) => o.keys(),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn values(&self) -> Result<Vec<Self>, Error<C>> {
        match self {
            Self::Object(o) => o.values(),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn exchange(&self, other: Box<dyn Obj<C>>) -> Result<(), Error<C>> {
        if let Self::Object(o) = self {
            o.exchange(other)?;
        }

        Ok(())
    }

    pub fn to_string(&self, realm: &mut C) -> Result<String, Error<C>> {
        Ok(match self {
            Self::Object(o) => o.to_string(realm)?,
            Self::Null => "null".to_string(),
            Self::Undefined => "undefined".to_string(),
            Self::Number(n) => n.to_string(),
            Self::String(s) => s.clone(),
            Self::Boolean(b) => b.to_string(),
            Self::Symbol(s) => s.to_string(),
        })
    }
}

#[derive(Debug)]
pub struct Iter<C: Realm> {
    next_obj: Value<C>,
}

pub struct CtxIter<'a, C: Realm> {
    next_obj: Value<C>,
    realm: &'a mut C,
}

impl<C: Realm> Iterator for CtxIter<'_, C> {
    type Item = Result<Value<C>, Error<C>>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self
            .next_obj
            .call_method(&"next".into(), self.realm, Vec::new());
        let next = match next {
            Ok(next) => next,
            Err(e) => return Some(Err(e)),
        };

        let done = next.get_property(&Value::string("done"), self.realm);

        let done = match done {
            Ok(done) => done.is_truthy(),
            Err(e) => return Some(Err(e)),
        };

        if done {
            return None;
        }

        Some(next.get_property(&Value::string("value"), self.realm))
    }
}

impl<C: Realm> Iter<C> {
    pub fn next(&self, realm: &mut C) -> Result<Option<Value<C>>, Error<C>> {
        let next = self
            .next_obj
            .call_method(&"next".into(), realm, Vec::new())?;
        let done = next.get_property(&Value::string("done"), realm)?;

        if done.is_truthy() {
            return Ok(None);
        }
        next.get_property(&Value::string("value"), realm).map(Some)
    }
}

pub trait CustomGcRef: Collectable + CustomGcRefUntyped {
    fn gc_ref(&self) -> Option<GcRef<Self>>;
}

pub trait CustomGcRefUntyped {
    fn gc_untyped_ref<U: Collectable>(&self) -> Option<GcRef<U>>;
}

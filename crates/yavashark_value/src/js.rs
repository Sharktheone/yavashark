use crate::Error;
pub use constructor::*;
pub use context::*;
pub use conversion::*;
pub use function::*;
pub use name::*;
use num_bigint::BigInt;
use num_traits::Zero;
pub use object::*;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::rc::Rc;
pub use symbol::*;
pub use variable::*;
use yavashark_garbage::{Collectable, GcRef, OwningGcGuard};
use yavashark_string::{ToYSString, YSString};

mod constructor;
mod context;
mod conversion;
mod function;
mod name;
mod object;
mod object_impl;
pub mod ops;
mod symbol;
pub mod variable;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ConstString {
    String(&'static str),
    Owned(String),
}

impl AsRef<str> for ConstString {
    fn as_ref(&self) -> &str {
        match self {
            Self::String(s) => s,
            Self::Owned(s) => s,
        }
    }
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
    String(YSString),
    Boolean(bool),
    Object(Object<C>),
    Symbol(Symbol),
    BigInt(Rc<BigInt>),
}

impl<C: Realm> Clone for Value<C> {
    fn clone(&self) -> Self {
        self.copy()
    }
}

impl<C: Realm> Eq for Value<C> {}

impl<C: Realm> AsRef<Self> for Value<C> {
    fn as_ref(&self) -> &Self {
        self
    }
}

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
    BigInt,
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Null => write!(f, "Null"),
            Self::Undefined => write!(f, "Undefined"),
            Self::Number => write!(f, "Number"),
            Self::String => write!(f, "String"),
            Self::Boolean => write!(f, "Boolean"),
            Self::Object => write!(f, "Object"),
            Self::Function => write!(f, "Function"),
            Self::Symbol => write!(f, "Symbol"),
            Self::BigInt => write!(f, "Bigint"),
        }
    }
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
            Self::BigInt(b) => (Type::BigInt, b).hash(state),
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
            Self::BigInt(b) => Self::BigInt(b.clone()),
        }
    }

    #[must_use]
    pub fn ty(&self) -> Type {
        match self {
            Self::Null => Type::Null,
            Self::Undefined => Type::Undefined,
            Self::Number(_) => Type::Number,
            Self::String(_) => Type::String,
            Self::Boolean(_) => Type::Boolean,
            Self::Object(o) => {
                if o.is_function() {
                    Type::Function
                } else {
                    Type::Object
                }
            }
            Self::Symbol(_) => Type::Symbol,
            Self::BigInt(_) => Type::BigInt,
        }
    }

    #[must_use]
    pub fn string(s: &'static str) -> Self {
        s.into()
    }

    #[must_use]
    pub const fn is_nan(&self) -> bool {
        match self {
            Self::Number(n) => n.is_nan(),
            _ => false,
        }
    }

    #[must_use]
    pub fn is_falsey(&self) -> bool {
        match self {
            Self::Null | Self::Undefined => true,
            Self::Number(n) => *n == 0.0 || n.is_nan(),
            Self::String(s) => s.is_empty(),
            Self::Boolean(b) => !b,
            Self::Object(o) => {
                o.primitive()
                    .is_some_and(|o| if o.is_object() { false } else { o.is_falsey() })
            }
            Self::Symbol(_) => false,
            Self::BigInt(b) => b.is_zero(),
        }
    }

    #[must_use]
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Null | Self::Undefined => false,
            Self::Number(n) => !(*n == 0.0 || n.is_nan()),
            Self::String(s) => !s.is_empty(),
            Self::Boolean(b) => *b,
            Self::Object(o) => {
                o.primitive()
                    .is_none_or(|o| if o.is_object() { true } else { o.is_truthy() })
            }
            Self::Symbol(_) => true,
            Self::BigInt(b) => !b.is_zero(),
        }
    }

    #[must_use]
    pub const fn is_nullish(&self) -> bool {
        matches!(self, Self::Null | Self::Undefined)
    }

    #[must_use]
    pub fn type_of(&self) -> &'static str {
        match self {
            Self::Null => "object",
            Self::Undefined => "undefined",
            Self::Number(_) => "number",
            Self::String(_) => "string",
            Self::Boolean(_) => "boolean",
            Self::Object(o) => {
                if o.is_function() | o.is_constructor() {
                    "function"
                } else {
                    "object"
                }
            }
            Self::Symbol(_) => "symbol",
            Self::BigInt(_) => "bigint",
        }
    }

    #[must_use]
    pub fn gc_ref(&self) -> Option<GcRef<BoxedObj<C>>> {
        match self {
            Self::Object(o) => Some(o.get_ref()),
            _ => None,
        }
    }

    pub fn prototype(&self, realm: &mut C) -> Result<Self, Error<C>> {
        let obj = self.as_object()?;

        let proto = obj.prototype()?;

        proto.resolve(self.copy(), realm)
    }

    pub fn as_object(&self) -> Result<&Object<C>, Error<C>> {
        let Self::Object(obj) = &self else {
            return Err(Error::ty("expected object"));
        };

        Ok(obj)
    }

    #[allow(clippy::needless_lifetimes)]
    pub fn downcast<'a, T: 'static>(
        &'a self,
    ) -> Result<Option<OwningGcGuard<'a, BoxedObj<C>, T>>, Error<C>> {
        let obj = self.as_object()?;

        Ok(obj.downcast())
    }

    pub fn to_object(self) -> Result<Object<C>, Error<C>> {
        match self {
            Self::Object(o) => Ok(o),
            _ => Err(Error::ty("expected object")),
        }
    }

    #[must_use]
    pub const fn as_number(&self) -> f64 {
        let Self::Number(n) = &self else {
            return f64::NAN;
        };

        *n
    }

    #[must_use]
    pub const fn is_undefined(&self) -> bool {
        matches!(self, Self::Undefined)
    }

    #[must_use]
    pub const fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    #[must_use]
    pub const fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    #[must_use]
    pub const fn is_boolean(&self) -> bool {
        matches!(self, Self::Boolean(_))
    }

    #[must_use]
    pub const fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    #[must_use]
    pub const fn is_symbol(&self) -> bool {
        matches!(self, Self::Symbol(_))
    }

    #[must_use]
    pub const fn is_bigint(&self) -> bool {
        matches!(self, Self::BigInt(_))
    }

    #[must_use]
    pub fn is_function(&self) -> bool {
        matches!(self, Self::Object(o) if o.is_function())
    }

    pub fn assert_no_object(self) -> Result<Self, Error<C>> {
        match self {
            Self::Object(_) => Err(Error::ty("expected primitive, got object")),
            _ => Ok(self),
        }
    }

    pub fn to_primitive(&self, hint: Hint, realm: &mut C) -> Result<Self, Error<C>> {
        match self {
            Self::Object(o) => o.to_primitive(hint, realm),
            _ => Ok(self.copy()),
        }
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
            Self::Symbol(s) => write!(f, "{s}"),
            Self::BigInt(b) => write!(f, "{b}"),
        }
    }
}

impl<C: Realm> ToYSString for Value<C> {
    fn to_ys_string(&self) -> YSString {
        match self {
            Self::Null => YSString::new_static("null"),
            Self::Undefined => YSString::new_static("undefined"),
            Self::Number(n) => YSString::from(n.to_string()),
            Self::String(s) => s.clone(),
            Self::Boolean(b) => b.to_ys_string(),
            Self::Object(b) => b.to_ys_string(),
            Self::Symbol(s) => s.to_ys_string(),
            Self::BigInt(n) => YSString::from(n.to_string()),
        }
    }
}

impl<C: Realm> CustomGcRefUntyped for Value<C> {
    fn gc_untyped_ref<U: Collectable>(&self) -> Option<GcRef<U>> {
        match self {
            Self::Object(o) => Some(o.get_untyped_ref()),
            _ => None,
        }
    }
}

impl<C: Realm> Value<C> {
    #[allow(clippy::iter_not_returning_iterator)]
    pub fn iter<'a>(&self, realm: &'a mut C) -> Result<CtxIter<'a, C>, Error<C>> {
        let iter = self.get_property(&Symbol::ITERATOR.into(), realm)?;
        let iter = iter.call(realm, Vec::new(), self.copy())?;

        Ok(CtxIter {
            next_obj: iter,
            realm,
        })
    }

    pub fn iter_no_realm(&self, realm: &mut C) -> Result<Iter<C>, Error<C>> {
        let iter = self.get_property(&Symbol::ITERATOR.into(), realm)?;
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

    pub fn get_property_opt(&self, name: &Self, realm: &mut C) -> Result<Option<Self>, Error<C>> {
        match self {
            Self::Object(o) => o.resolve_property(name, realm),
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

    pub fn has_key(&self, name: &Self) -> Result<bool, Error<C>> {
        match self {
            Self::Object(o) => o.has_key(name),
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

    pub fn to_string(&self, realm: &mut C) -> Result<YSString, Error<C>> {
        Ok(match self {
            Self::Object(o) => {
                if let Some(prim) = o.primitive() {
                    return prim.to_string(realm);
                }

                o.to_string(realm)?
            }
            Self::Null => "null".into(),
            Self::Undefined => "undefined".into(),
            Self::Number(n) => fmt_num(*n),
            Self::String(s) => s.clone(),
            Self::Boolean(b) => b.to_ys_string(),
            Self::Symbol(s) => s.to_ys_string(),
            Self::BigInt(b) => b.to_string().into(),
        })
    }

    pub fn to_string_no_realm(&self) -> Result<YSString, Error<C>> {
        Ok(match self {
            Self::Object(o) => {
                if let Some(prim) = o.primitive() {
                    return prim.to_string_no_realm();
                }

                o.to_string_internal()?
            }
            Self::Null => YSString::new_static("null"),
            Self::Undefined => YSString::new_static("undefined"),
            Self::Number(n) => fmt_num(*n),
            Self::String(s) => s.clone(),
            Self::Boolean(b) => b.to_ys_string(),
            Self::Symbol(s) => s.to_ys_string(),
            Self::BigInt(b) => b.to_string().into(),
        })
    }

    pub fn into_string(self, realm: &mut C) -> Result<YSString, Error<C>> {
        Ok(match self {
            Self::Object(o) => {
                if let Some(prim) = o.primitive() {
                    return prim.into_string(realm);
                }

                o.to_string(realm)?
            }
            Self::Null => YSString::new_static("null"),
            Self::Undefined => YSString::new_static("undefined"),
            Self::Number(n) => fmt_num(n),
            Self::String(s) => s,
            Self::Boolean(b) => b.to_ys_string(),
            Self::Symbol(s) => s.to_ys_string(),
            Self::BigInt(b) => b.to_string().into(),
        })
    }
}

#[must_use]
pub fn fmt_num(n: f64) -> YSString {
    if n.is_nan() {
        YSString::new_static("NaN")
    } else if n == 0.0 {
        YSString::new_static("0")
    } else if n.is_infinite() {
        if n.is_sign_positive() {
            YSString::new_static("Infinity")
        } else {
            YSString::new_static("-Infinity")
        }
    } else {
        let abs_n = n.abs();
        if (1e-6..1e21).contains(&abs_n) {
            YSString::from_string(n.to_string())
        } else {
            YSString::from_string(format!("{n:e}"))
        }
    }
}

impl<C: Realm> From<Symbol> for Value<C> {
    fn from(s: Symbol) -> Self {
        Self::Symbol(s)
    }
}

impl<C: Realm> From<&Symbol> for Value<C> {
    fn from(s: &Symbol) -> Self {
        Self::Symbol(s.clone())
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

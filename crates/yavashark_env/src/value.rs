use crate::error::Error;
pub use constructor::*;
pub use conversion::*;
pub use function::*;
pub use name::*;
use num_bigint::BigInt;
use num_traits::Zero;
pub use object::*;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use indexmap::Equivalent;
pub use symbol::*;
pub use variable::*;
use yavashark_garbage::{Collectable, GcRef, OwningGcGuard};
use yavashark_string::{ToYSString, YSString};
use crate::Realm;

mod constructor;
mod conversion;
mod function;
mod name;
mod obj;
mod object;
mod object_impl;
mod object_v2;
pub mod ops;
pub mod property_key;
mod symbol;
pub mod variable;


#[derive(Debug, PartialEq)]
pub enum Value {
    Null,
    Undefined,
    Number(f64),
    String(YSString),
    Boolean(bool),
    Object(Object),
    Symbol(Symbol),
    BigInt(Rc<BigInt>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum WeakValue {
    Null,
    Undefined,
    Number(f64),
    String(YSString),
    Boolean(bool),
    Object(WeakObject),
    Symbol(Symbol),
    BigInt(Rc<BigInt>),
}

impl Clone for Value {
    fn clone(&self) -> Self {
        self.copy()
    }
}

impl Eq for Value {}
impl Eq for WeakValue {}

impl Equivalent<Value> for WeakValue {
    fn equivalent(&self, other: &Value) -> bool {
        match (self, other) {
            (Self::Null, Value::Null) => true,
            (Self::Undefined, Value::Undefined) => true,
            (Self::Number(a), Value::Number(b)) => a.to_bits() == b.to_bits(),
            (Self::String(a), Value::String(b)) => a == b,
            (Self::Boolean(a), Value::Boolean(b)) => a == b,
            (Self::Object(a), Value::Object(b)) => a.equivalent(b),
            (Self::Symbol(a), Value::Symbol(b)) => a == b,
            (Self::BigInt(a), Value::BigInt(b)) => a == b,
            _ => false,
        }
    }
}

impl Equivalent<WeakValue> for Value {
    fn equivalent(&self, other: &WeakValue) -> bool {
        other.equivalent(self)
    }
}

impl Hash for WeakValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
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

impl AsRef<Self> for Value {
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

impl Hash for Value {
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

impl Value {
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
            Self::BigInt(b) => Self::BigInt(Rc::clone(b)),
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
            Self::Object(_) | Self::Symbol(_) => false,
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
            Self::Object(_) | Self::Symbol(_) => true,
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
    pub fn gc_ref(&self) -> Option<GcRef<BoxedObj>> {
        match self {
            Self::Object(o) => Some(o.get_ref()),
            _ => None,
        }
    }

    pub fn prototype(&self, realm: &mut Realm) -> Result<Self, Error> {
        let obj = self.as_object()?;

        let proto = obj.prototype()?;

        proto.resolve(self.copy(), realm)
    }

    pub const fn as_object(&self) -> Result<&Object, Error> {
        let Self::Object(obj) = &self else {
            return Err(Error::ty("expected object"));
        };

        Ok(obj)
    }

    #[allow(clippy::needless_lifetimes)]
    pub fn downcast<T: 'static>(
        &self,
    ) -> Result<Option<OwningGcGuard<'static, BoxedObj, T>>, Error> {
        let obj = self.as_object()?;

        Ok(obj.downcast())
    }

    pub fn to_object(self) -> Result<Object, Error> {
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

    pub const fn is_null(&self) -> bool {
        matches!(self, Self::Null)
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

    #[must_use]
    pub fn is_constructor(&self) -> bool {
        matches!(self, Self::Object(o) if o.is_constructor())
    }

    pub fn assert_no_object(self) -> Result<Self, Error> {
        match self {
            Self::Object(_) => Err(Error::ty("expected primitive, got object")),
            _ => Ok(self),
        }
    }

    pub fn to_primitive(&self, hint: Hint, realm: &mut Realm) -> Result<Self, Error> {
        match self {
            Self::Object(o) => o.to_primitive(hint, realm),
            _ => Ok(self.copy()),
        }
    }

    pub fn downgrade(&self) -> WeakValue {
        match self {
            Self::Null => WeakValue::Null,
            Self::Undefined => WeakValue::Undefined,
            Self::Number(n) => WeakValue::Number(*n),
            Self::String(s) => WeakValue::String(s.clone()),
            Self::Boolean(b) => WeakValue::Boolean(*b),
            Self::Object(o) => WeakValue::Object(o.downgrade()),
            Self::Symbol(s) => WeakValue::Symbol(s.clone()),
            Self::BigInt(b) => WeakValue::BigInt(Rc::clone(b)),
        }
    }
}

impl WeakValue {
    pub fn upgrade(&self) -> Option<Value> {
        Some(match self {
            Self::Null => Value::Null,
            Self::Undefined => Value::Undefined,
            Self::Number(n) => Value::Number(*n),
            Self::String(s) => Value::String(s.clone()),
            Self::Boolean(b) => Value::Boolean(*b),
            Self::Object(o) => Value::Object(o.upgrade()?),
            Self::Symbol(s) => Value::Symbol(s.clone()),
            Self::BigInt(b) => Value::BigInt(Rc::clone(b)),
        })
    }
}

impl Display for Value {
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
            Self::BigInt(b) => write!(f, "{b}"),
        }
    }
}

impl ToYSString for Value {
    fn to_ys_string(&self) -> YSString {
        match self {
            Self::Null => YSString::new_static("null"),
            Self::Undefined => YSString::new_static("undefined"),
            Self::Number(n) => YSString::from(n.to_string()),
            Self::String(s) => s.clone(),
            Self::Boolean(b) => b.to_ys_string(),
            Self::Object(b) => b.to_ys_string(),
            Self::Symbol(s) => format!("Symbol({})", s.as_ref()).into(),
            Self::BigInt(n) => YSString::from(n.to_string()),
        }
    }
}

impl CustomGcRefUntyped for Value {
    fn gc_untyped_ref<U: Collectable>(&self) -> Option<GcRef<U>> {
        match self {
            Self::Object(o) => Some(o.get_untyped_ref()),
            _ => None,
        }
    }
}

impl Object {
    #[allow(clippy::iter_not_returning_iterator)]
    pub fn iter<'a>(&self, realm: &'a mut Realm) -> Result<CtxIter<'a>, Error> {
        let iter = self.get_iter(realm)?;

        Ok(CtxIter {
            next_obj: iter,
            realm,
        })
    }

    pub fn iter_no_realm(&self, realm: &mut Realm) -> Result<Iter, Error> {
        let iter = self.get_iter(realm)?;

        Ok(Iter { next_obj: iter })
    }

    pub fn get_iter(&self, realm: &mut Realm) -> Result<Value, Error> {
        let iter = self
            .resolve_property(&Symbol::ITERATOR.into(), realm)?
            .ok_or(Error::reference("Object is not iterable"))?;

        iter.call(realm, Vec::new(), self.clone().into())
    }

    pub fn get_async_iter(&self, realm: &mut Realm) -> Result<Value, Error> {
        let iter = self
            .resolve_property(&Symbol::ASYNC_ITERATOR.into(), realm)?
            .ok_or(Error::reference("Object is not async iterable"))?;

        iter.call(realm, Vec::new(), self.clone().into())
    }
}

impl Value {
    #[allow(clippy::iter_not_returning_iterator)]
    pub fn iter<'a>(&self, realm: &'a mut Realm) -> Result<CtxIter<'a>, Error> {
        self.as_object()?.iter(realm)
    }

    pub fn iter_no_realm(&self, realm: &mut Realm) -> Result<Iter, Error> {
        self.as_object()?.iter_no_realm(realm)
    }

    pub fn get_iter(&self, realm: &mut Realm) -> Result<Self, Error> {
        self.as_object()?.get_iter(realm)
    }

    pub fn get_async_iter(&self, realm: &mut Realm) -> Result<Self, Error> {
        self.as_object()?.get_async_iter(realm)
    }

    pub fn get_property(&self, name: &Self, realm: &mut Realm) -> Result<Self, Error> {
        match self {
            Self::Object(o) => o
                .resolve_property(name, realm)?
                .ok_or(Error::reference_error(format!(
                    "{name} does not exist on object"
                ))),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn get_property_opt(&self, name: &Self, realm: &mut Realm) -> Result<Option<Self>, Error> {
        match self {
            Self::Object(o) => o.resolve_property(name, realm),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn get_property_no_get_set(&self, name: &Self) -> Result<ObjectProperty, Error> {
        match self {
            Self::Object(o) => o
                .resolve_property_no_get_set(name)?
                .ok_or(Error::reference_error(format!(
                    "{name} does not exist on object"
                ))),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn define_property(&self, name: Self, value: Self) -> Result<(), Error> {
        match self {
            Self::Object(o) => o.define_property(name, value),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn contains_key(&self, name: &Self) -> Result<bool, Error> {
        match self {
            Self::Object(o) => o.contains_key(name),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn has_key(&self, name: &Self) -> Result<bool, Error> {
        match self {
            Self::Object(o) => o.has_key(name),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn call(&self, realm: &mut Realm, args: Vec<Self>, this: Self) -> Result<Self, Error> {
        match self {
            Self::Object(o) => o.call(realm, args, this),
            _ => Err(Error::ty("Value is not a function")),
        }
    }

    pub fn call_method(
        &self,
        name: &Self,
        realm: &mut Realm,
        args: Vec<Self>,
    ) -> Result<Self, Error> {
        let method = self.get_property(name, realm)?;

        method.call(realm, args, self.copy())
    }

    #[allow(clippy::type_complexity)]
    ///(name, value)
    pub fn properties(&self) -> Result<Vec<(Self, Self)>, Error> {
        match self {
            Self::Object(o) => o.properties(),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn keys(&self) -> Result<Vec<Self>, Error> {
        match self {
            Self::Object(o) => o.keys(),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn values(&self) -> Result<Vec<Self>, Error> {
        match self {
            Self::Object(o) => o.values(),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn to_string(&self, realm: &mut Realm) -> Result<YSString, Error> {
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
            Self::Symbol(s) => format!("Symbol({})", s.as_ref()).into(),
            Self::BigInt(b) => b.to_string().into(),
        })
    }

    pub fn to_string_no_realm(&self) -> Result<YSString, Error> {
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
            Self::Symbol(_) => return Err(Error::ty("Cannot convert Symbol to string")),
            Self::BigInt(b) => b.to_string().into(),
        })
    }

    pub fn into_string(self, realm: &mut Realm) -> Result<YSString, Error> {
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
            Self::Symbol(s) => format!("Symbol({})", s.as_ref()).into(),
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
            let mut num = format!("{n:e}");

            if let Some(e_idx) = num.find('e') {
                if num.as_bytes().get(e_idx + 1).copied() != Some(b'-') && num.len() > e_idx + 1 {
                    num.insert(e_idx + 1, '+');
                }
            }

            YSString::from_string(num)
        }
    }
}

impl From<Symbol> for Value {
    fn from(s: Symbol) -> Self {
        Self::Symbol(s)
    }
}

impl From<&Symbol> for Value {
    fn from(s: &Symbol) -> Self {
        Self::Symbol(s.clone())
    }
}

#[derive(Debug)]
pub struct Iter {
    next_obj: Value,
}

pub struct CtxIter<'a> {
    next_obj: Value,
    realm: &'a mut Realm,
}

impl Iterator for CtxIter<'_> {
    type Item = Result<Value, Error>;

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

impl Iter {
    pub fn next(&self, realm: &mut Realm) -> Result<Option<Value>, Error> {
        self.next_obj.iter_next(realm)
    }

    pub fn close(self, realm: &mut Realm) -> Result<(), Error> {
        self.next_obj.iter_close(realm)
    }
}

impl Value {
    pub fn iter_next(&self, realm: &mut Realm) -> Result<Option<Self>, Error> {
        self.as_object()?.iter_next(realm)
    }

    pub fn async_iter_next(&self, realm: &mut Realm) -> Result<Self, Error> {
        self.as_object()?.async_iter_next(realm)
    }

    pub fn iter_res(&self, realm: &mut Realm) -> Result<Option<Self>, Error> {
        self.as_object()?.iter_res(realm)
    }

    pub fn iter_done(&self, realm: &mut Realm) -> Result<bool, Error> {
        self.as_object()?.iter_done(realm)
    }

    pub fn iter_next_no_out(&self, realm: &mut Realm) -> Result<(), Error> {
        self.as_object()?.iter_next_no_out(realm)
    }

    pub fn iter_next_is_finished(&self, realm: &mut Realm) -> Result<bool, Error> {
        self.as_object()?.iter_next_is_finished(realm)
    }

    pub fn iter_close(&self, realm: &mut Realm) -> Result<(), Error> {
        let obj = self.as_object()?;
        let return_method = obj.resolve_property(&"return".into(), realm)?;

        if let Some(return_method) = return_method {
            return_method.call(realm, Vec::new(), self.clone())?;
        }

        Ok(())
    }
}

impl Object {
    pub fn iter_next(&self, realm: &mut Realm) -> Result<Option<Value>, Error> {
        let next = self.call_method(&"next".into(), realm, Vec::new())?;
        let done = next.get_property_opt(&Value::string("done"), realm)?;

        if done.is_some_and(|x| x.is_truthy()) {
            return Ok(None);
        }

        next.get_property_opt(&Value::string("value"), realm)
            .map(|opt| Some(opt.unwrap_or(Value::Undefined)))
    }

    pub fn async_iter_next(&self, realm: &mut Realm) -> Result<Value, Error> {
        let promise = self.call_method(&"next".into(), realm, Vec::new())?;

        Ok(promise)
    }

    pub fn iter_res(&self, realm: &mut Realm) -> Result<Option<Value>, Error> {
        let done = self.resolve_property(&Value::string("done"), realm)?;

        if done.is_some_and(|x| x.is_truthy()) {
            return Ok(None);
        }
        self.resolve_property(&Value::string("value"), realm)
            .map(|x| Some(x.unwrap_or(Value::Undefined)))
    }

    pub fn iter_done(&self, realm: &mut Realm) -> Result<bool, Error> {
        let done = self.resolve_property(&Value::string("done"), realm)?;

        Ok(done.is_some_and(|x| x.is_truthy()))
    }

    pub fn iter_next_no_out(&self, realm: &mut Realm) -> Result<(), Error> {
        let _ = self.call_method(&"next".into(), realm, Vec::new())?;

        Ok(())
    }

    pub fn iter_next_is_finished(&self, realm: &mut Realm) -> Result<bool, Error> {
        let next = self.call_method(&"next".into(), realm, Vec::new())?;
        let done = next.get_property_opt(&Value::string("done"), realm)?;

        Ok(done.is_some_and(|done| done.is_truthy()))
    }
}

pub trait CustomGcRef: Collectable + CustomGcRefUntyped {
    fn gc_ref(&self) -> Option<GcRef<Self>>;
}

pub trait CustomGcRefUntyped {
    fn gc_untyped_ref<U: Collectable>(&self) -> Option<GcRef<U>>;
}

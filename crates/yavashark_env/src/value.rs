use crate::error::Error;
use crate::{GCd, InternalPropertyKey, ObjectHandle, PropertyKey, Realm, Res};
pub use constructor::*;
pub use conversion::*;
pub use function::*;
use indexmap::Equivalent;
pub use name::*;
use num_bigint::BigInt;
use num_traits::Zero;
pub use object::*;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
pub use symbol::*;
pub use variable::*;
use yavashark_garbage::{Collectable, GcRef, OwningGcGuard};
use yavashark_string::{ToYSString, YSString};
use crate::value::property_key::IntoPropertyKey;

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

#[derive(Clone, Debug, PartialEq)]
pub enum PrimitiveValue {
    Null,
    Undefined,
    Number(f64),
    String(YSString),
    Boolean(bool),
    Symbol(Symbol),
    BigInt(Rc<BigInt>),
}

impl Display for PrimitiveValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Undefined => write!(f, "undefined"),
            Self::Number(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "{s}"),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::Symbol(s) => write!(f, "Symbol({s})"),
            Self::BigInt(b) => write!(f, "{b}"),
        }
    }
}

impl PrimitiveValue {
    pub fn into_string(self) -> YSString {
        match self {
            Self::Null => YSString::new_static("null"),
            Self::Undefined => YSString::new_static("undefined"),
            Self::Number(n) => fmt_num(n),
            Self::String(s) => s,
            Self::Boolean(b) => b.to_ys_string(),
            Self::Symbol(s) => format!("Symbol({})", s.as_ref()).into(),
            Self::BigInt(b) => b.to_string().into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObjectOrNull {
    Object(Object),
    Null,
}

impl ObjectOrNull {
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub fn to_object(self) -> Res<Object> {
        match self {
            Self::Object(o) => Ok(o),
            Self::Null => Err(Error::ty("expected object, got null")),
        }
    }
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

impl Equivalent<PrimitiveValue> for Value {
    fn equivalent(&self, other: &PrimitiveValue) -> bool {
        match (self, other) {
            (Value::Null, PrimitiveValue::Null) => true,
            (Value::Undefined, PrimitiveValue::Undefined) => true,
            (Value::Number(a), PrimitiveValue::Number(b)) => a.to_bits() == b.to_bits(),
            (Value::String(a), PrimitiveValue::String(b)) => a == b,
            (Value::Boolean(a), PrimitiveValue::Boolean(b)) => a == b,
            (Value::Symbol(a), PrimitiveValue::Symbol(b)) => a == b,
            (Value::BigInt(a), PrimitiveValue::BigInt(b)) => a == b,
            _ => false,
        }
    }
}

impl Equivalent<Value> for PrimitiveValue {
    fn equivalent(&self, other: &Value) -> bool {
        other.equivalent(self)
    }
}

impl Equivalent<PrimitiveValue> for WeakValue {
    fn equivalent(&self, other: &PrimitiveValue) -> bool {
        match (self, other) {
            (Self::Null, PrimitiveValue::Null) => true,
            (Self::Undefined, PrimitiveValue::Undefined) => true,
            (Self::Number(a), PrimitiveValue::Number(b)) => a.to_bits() == b.to_bits(),
            (Self::String(a), PrimitiveValue::String(b)) => a == b,
            (Self::Boolean(a), PrimitiveValue::Boolean(b)) => a == b,
            (Self::Symbol(a), PrimitiveValue::Symbol(b)) => a == b,
            (Self::BigInt(a), PrimitiveValue::BigInt(b)) => a == b,
            _ => false,
        }
    }
}

impl Equivalent<WeakValue> for PrimitiveValue {
    fn equivalent(&self, other: &WeakValue) -> bool {
        other.equivalent(self)
    }
}

impl Equivalent<ObjectOrNull> for Value {
    fn equivalent(&self, other: &ObjectOrNull) -> bool {
        match (self, other) {
            (Value::Null, ObjectOrNull::Null) => true,
            (Value::Object(a), ObjectOrNull::Object(b)) => a.equivalent(b),
            _ => false,
        }
    }
}

impl Equivalent<Value> for ObjectOrNull {
    fn equivalent(&self, other: &Value) -> bool {
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

impl From<ObjectHandle> for ObjectOrNull {
    fn from(o: ObjectHandle) -> Self {
        ObjectOrNull::Object(o)
    }
}

impl From<Option<ObjectHandle>> for ObjectOrNull {
    fn from(o: Option<ObjectHandle>) -> Self {
        match o {
            Some(o) => ObjectOrNull::Object(o),
            None => ObjectOrNull::Null,
        }
    }
}

impl TryFrom<Value> for ObjectOrNull {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Object(o) => Ok(ObjectOrNull::Object(o)),
            Value::Null => Ok(ObjectOrNull::Null),
            _ => Err(Error::ty("expected object or null")),
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
                if o.is_callable() {
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
                if o.is_callable() | o.is_constructable() {
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

    pub fn prototype(&self, realm: &mut Realm) -> Res<ObjectOrNull> {
        let obj = self.as_object()?;

        obj.prototype(realm)
    }

    pub const fn as_object(&self) -> Res<&Object> {
        let Self::Object(obj) = &self else {
            return Err(Error::ty("expected object"));
        };

        Ok(obj)
    }

    #[allow(clippy::needless_lifetimes)]
    pub fn downcast<T: 'static>(
        &self,
    ) -> Res<Option<GCd<T>>> {
        let obj = self.as_object()?;

        Ok(obj.downcast())
    }

    pub fn to_object(self) -> Res<Object> {
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
    pub fn is_callable(&self) -> bool {
        matches!(self, Self::Object(o) if o.is_callable())
    }

    #[must_use]
    pub fn is_constructable(&self) -> bool {
        matches!(self, Self::Object(o) if o.is_constructable())
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
            .resolve_property(Symbol::ITERATOR, realm)?
            .ok_or(Error::reference("Object is not iterable"))?;

        iter.call(realm, Vec::new(), self.clone().into())
    }

    pub fn get_async_iter(&self, realm: &mut Realm) -> Result<Value, Error> {
        let iter = self
            .resolve_property(Symbol::ASYNC_ITERATOR, realm)?
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

    pub fn get_property(&self, name: impl IntoPropertyKey, realm: &mut Realm) -> Result<Self, Error> {
        let name = name.into_property_key(realm)?;
        match self {
            Self::Object(o) => o
                .resolve_property(name.clone(), realm)?
                .ok_or(Error::reference_error(format!(
                    "{name} does not exist on object"
                ))),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn get_property_opt(&self, name: impl IntoPropertyKey, realm: &mut Realm) -> Result<Option<Self>, Error> {
        match self {
            Self::Object(o) => o.resolve_property(name, realm),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    // pub fn get_property_no_get_set(&self, name: &Self) -> Result<ObjectProperty, Error> {
    //     match self {
    //         Self::Object(o) => o
    //             .resolve_property_no_get_set(name)?
    //             .ok_or(Error::reference_error(format!(
    //                 "{name} does not exist on object"
    //             ))),
    //         _ => Err(Error::ty("Value is not an object")),
    //     }
    // }

    pub fn define_property(&self, name: impl IntoPropertyKey, value: Self, realm: &mut Realm) -> Result<(), Error> {

        match self {
            Self::Object(o) => {
                let name = name.into_internal_property_key(realm)?;
                o.define_property(name, value, realm)?;

                Ok(())
            },
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn contains_key(&self, name: impl IntoPropertyKey, realm: &mut Realm) -> Result<bool, Error> {
        match self {
            Self::Object(o) => {
                let name = name.into_internal_property_key(realm)?;
                o.contains_own_key(name, realm)
            },
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn has_key(&self, name: impl IntoPropertyKey, realm: &mut Realm) -> Result<bool, Error> {
        match self {
            Self::Object(o) => {
                let name = name.into_internal_property_key(realm)?;
                o.contains_key(name, realm)
            },
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn call(&self, realm: &mut Realm, args: Vec<Self>, this: Self) -> Result<Self, Error> {
        match self {
            Self::Object(o) => o.call(args, this, realm),
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
    pub fn properties(&self, realm: &mut Realm) -> Result<Vec<(PropertyKey, Self)>, Error> {
        match self {
            Self::Object(o) => o.properties(realm),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn keys(&self, realm: &mut Realm) -> Result<Vec<PropertyKey>, Error> {
        match self {
            Self::Object(o) => o.keys(realm),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn values(&self, realm: &mut Realm) -> Result<Vec<Self>, Error> {
        match self {
            Self::Object(o) => o.values(realm),
            _ => Err(Error::ty("Value is not an object")),
        }
    }

    pub fn to_string(&self, realm: &mut Realm) -> Result<YSString, Error> {
        Ok(match self {
            Self::Object(o) => {
                if let Some(prim) = o.primitive(realm)? {
                    return Ok(prim.to_string().into());
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

    pub fn into_string(self, realm: &mut Realm) -> Result<YSString, Error> {
        Ok(match self {
            Self::Object(o) => {
                if let Some(prim) = o.primitive(realm)? {
                    return Ok(prim.into_string());
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

impl From<PrimitiveValue> for Value {
    fn from(p: PrimitiveValue) -> Self {
        match p {
            PrimitiveValue::Null => Self::Null,
            PrimitiveValue::Undefined => Self::Undefined,
            PrimitiveValue::Number(n) => Self::Number(n),
            PrimitiveValue::String(s) => Self::String(s),
            PrimitiveValue::Boolean(b) => Self::Boolean(b),
            PrimitiveValue::Symbol(s) => Self::Symbol(s),
            PrimitiveValue::BigInt(b) => Self::BigInt(b),
        }
    }
}

impl From<ObjectOrNull> for Value {
    fn from(o: ObjectOrNull) -> Self {
        match o {
            ObjectOrNull::Object(o) => Self::Object(o),
            ObjectOrNull::Null => Self::Null,
        }
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
        let return_method = obj.resolve_property("return", realm)?;

        if let Some(return_method) = return_method {
            return_method.call(realm, Vec::new(), self.clone())?;
        }

        Ok(())
    }
}

impl Object {
    pub fn iter_next(&self, realm: &mut Realm) -> Result<Option<Value>, Error> {
        let next = self.call_method("next", realm, Vec::new())?;
        let done = next.get_property_opt(&Value::string("done"), realm)?;

        if done.is_some_and(|x| x.is_truthy()) {
            return Ok(None);
        }

        next.get_property_opt(&Value::string("value"), realm)
            .map(|opt| Some(opt.unwrap_or(Value::Undefined)))
    }

    pub fn async_iter_next(&self, realm: &mut Realm) -> Result<Value, Error> {
        let promise = self.call_method("next", realm, Vec::new())?;

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
        let _ = self.call_method("next", realm, Vec::new())?;

        Ok(())
    }

    pub fn iter_next_is_finished(&self, realm: &mut Realm) -> Result<bool, Error> {
        let next = self.call_method("next", realm, Vec::new())?;
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

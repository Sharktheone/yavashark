use crate::{BoxedObj, Error, Object, Realm, Symbol, Value};
use half::f16;
use num_bigint::BigInt;
use std::any::type_name;
use std::rc::Rc;
use yavashark_garbage::OwningGcGuard;
use yavashark_string::YSString;

impl<C: Realm> From<&'static str> for Value<C> {
    fn from(s: &'static str) -> Self {
        Self::String(YSString::new_static(s))
    }
}

impl<C: Realm> From<String> for Value<C> {
    fn from(s: String) -> Self {
        Self::String(YSString::from_string(s))
    }
}

impl<C: Realm> From<&String> for Value<C> {
    fn from(s: &String) -> Self {
        Self::String(YSString::from_ref(&s))
    }
}

impl<C: Realm> From<YSString>  for Value<C> {
    fn from(s: YSString) -> Self {
        Self::String(s)
    }
}

impl<C: Realm> From<&YSString> for Value<C> {
    fn from(s: &YSString) -> Self {
        Self::String(s.clone())
    }
}

impl<C: Realm> From<()> for Value<C> {
    fn from((): ()) -> Self {
        Self::Undefined
    }
}

impl<C: Realm> From<f64> for Value<C> {
    fn from(n: f64) -> Self {
        Self::Number(n)
    }
}

impl<C: Realm> From<bool> for Value<C> {
    fn from(b: bool) -> Self {
        Self::Boolean(b)
    }
}

impl<C: Realm> From<u8> for Value<C> {
    fn from(n: u8) -> Self {
        Self::Number(f64::from(n))
    }
}

impl<C: Realm> From<u16> for Value<C> {
    fn from(n: u16) -> Self {
        Self::Number(f64::from(n))
    }
}

impl<C: Realm> From<u32> for Value<C> {
    fn from(n: u32) -> Self {
        Self::Number(f64::from(n))
    }
}

impl<C: Realm> From<u64> for Value<C> {
    fn from(n: u64) -> Self {
        Self::Number(n as f64)
    }
}

impl<C: Realm> From<i8> for Value<C> {
    fn from(n: i8) -> Self {
        Self::Number(f64::from(n))
    }
}

impl<C: Realm> From<i16> for Value<C> {
    fn from(n: i16) -> Self {
        Self::Number(f64::from(n))
    }
}

impl<C: Realm> From<i32> for Value<C> {
    fn from(n: i32) -> Self {
        Self::Number(f64::from(n))
    }
}

impl<C: Realm> From<i64> for Value<C> {
    fn from(n: i64) -> Self {
        Self::Number(n as f64)
    }
}

impl<C: Realm> From<usize> for Value<C> {
    fn from(n: usize) -> Self {
        Self::Number(n as f64)
    }
}

impl<C: Realm> From<isize> for Value<C> {
    fn from(n: isize) -> Self {
        Self::Number(n as f64)
    }
}

impl<C: Realm> From<f16> for Value<C> {
    fn from(n: f16) -> Self {
        Self::Number(f64::from(n))
    }
}

impl<C: Realm> From<f32> for Value<C> {
    fn from(n: f32) -> Self {
        Self::Number(f64::from(n))
    }
}

impl<C: Realm> From<BigInt> for Value<C> {
    fn from(n: BigInt) -> Self {
        Self::BigInt(Rc::new(n))
    }
}

impl<C: Realm> From<Rc<BigInt>> for Value<C> {
    fn from(n: Rc<BigInt>) -> Self {
        Self::BigInt(n)
    }
}

impl<C: Realm> From<Value<C>> for Result<Value<C>, Error<C>> {
    fn from(value: Value<C>) -> Self {
        Ok(value)
    }
}

impl<O: Into<Object<C>>, C: Realm> From<O> for Value<C> {
    fn from(o: O) -> Self {
        Self::Object(o.into())
    }
}

pub trait FromValue<R: Realm>: Sized {
    fn from_value(value: Value<R>) -> Result<Self, Error<R>>;
}

pub trait IntoValue<C: Realm> {
    fn into_value(self) -> Value<C>;
}

pub trait IntoValueRef<C: Realm> {
    type ValueRef: AsRef<Value<C>>;

    fn into_value_ref(self) -> Self::ValueRef;
}

impl<V: IntoValue<C>, C: Realm> IntoValueRef<C> for V {
    type ValueRef = Value<C>;

    fn into_value_ref(self) -> Self::ValueRef {
        self.into_value()
    }
}

impl<C: Realm> IntoValue<C> for Value<C> {
    fn into_value(self) -> Self {
        self
    }
}

impl<C: Realm> FromValue<C> for Value<C> {
    fn from_value(value: Self) -> Result<Self, Error<C>> {
        Ok(value)
    }
}

impl<C: Realm> FromValue<C> for YSString {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        match value {
            Value::String(s) => Ok(s),
            _ => Err(Error::ty_error(format!(
                "Expected a string, found {value:?}"
            ))),
        }
    }
}


// impl<C: Realm> FromValue<C> for String {
//     fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
//         match value {
//             Value::String(s) => Ok(s.to_string()),
//             _ => Err(Error::ty_error(format!(
//                 "Expected a string, found {value:?}"
//             ))),
//         }
//     }
// }

impl<C: Realm> FromValue<C> for bool {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        match value {
            Value::Boolean(b) => Ok(b),
            _ => Err(Error::ty_error(format!(
                "Expected a boolean, found {value:?}"
            ))),
        }
    }
}

impl<C: Realm> FromValue<C> for BigInt {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        match value {
            Value::BigInt(n) => Ok((*n).clone()),
            _ => Err(Error::ty_error(format!(
                "Expected a BigInt, found {value:?}"
            ))),
        }
    }
}

impl<C: Realm> FromValue<C> for Object<C> {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        match value {
            Value::Object(o) => Ok(o),
            _ => Err(Error::ty_error(format!(
                "Expected an object, found {value:?}"
            ))),
        }
    }
}

impl<C: Realm> FromValue<C> for () {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        match value {
            Value::Undefined => Ok(()),
            _ => Err(Error::ty_error(format!(
                "Expected undefined, found {value:?}"
            ))),
        }
    }
}

impl<C: Realm> IntoValue<C> for String {
    fn into_value(self) -> Value<C> {
        Value::String(self.into())
    }
}


impl<C: Realm> IntoValue<C> for YSString {
    fn into_value(self) -> Value<C> {
        Value::String(self)
    }
}

impl<C: Realm> IntoValue<C> for &'static str {
    fn into_value(self) -> Value<C> {
        Value::String(YSString::new_static(self))
    }
}

impl<C: Realm> IntoValue<C> for bool {
    fn into_value(self) -> Value<C> {
        Value::Boolean(self)
    }
}

impl<C: Realm> IntoValue<C> for Object<C> {
    fn into_value(self) -> Value<C> {
        Value::Object(self)
    }
}

impl<C: Realm> IntoValue<C> for BigInt {
    fn into_value(self) -> Value<C> {
        Value::BigInt(Rc::new(self))
    }
}

impl<C: Realm> IntoValue<C> for Rc<BigInt> {
    fn into_value(self) -> Value<C> {
        Value::BigInt(self)
    }
}

impl<C: Realm> IntoValue<C> for Symbol {
    fn into_value(self) -> Value<C> {
        Value::Symbol(self)
    }
}


impl<C: Realm> IntoValue<C> for &Symbol {
    fn into_value(self) -> Value<C> {
        Value::Symbol(self.clone())
    }
}

impl<C: Realm> IntoValue<C> for () {
    fn into_value(self) -> Value<C> {
        Value::Undefined
    }
}

macro_rules! impl_from_value {
    ($($t:ty),*) => {
        $(
            impl<C: Realm> FromValue<C> for $t {
                fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
                    match value {
                        Value::Number(n) => Ok(n as $t),
                        _ => Err(Error::ty_error(format!("Expected a number, found {:?}", value))),
                    }
                }
            }

            impl<C: Realm> IntoValue<C> for $t {
                fn into_value(self) -> Value<C> {
                    #[allow(clippy::cast_lossless)]
                    Value::Number(self as f64)
                }
            }
        )*
    };
    () => {};
}

impl_from_value!(u8, u16, u32, u64, i8, i16, i32, i64, i128, usize, isize, f32, f64);

impl<C: Realm, V: 'static> FromValue<C> for OwningGcGuard<'_, BoxedObj<C>, V> {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        let obj = match value {
            Value::Object(obj) => Ok(obj.get_owning()),
            _ => Err(Error::ty_error(format!(
                "Expected an object, found {value:?}"
            ))),
        }?;

        obj.maybe_map(BoxedObj::downcast).map_err(|obj| {
            Error::ty_error(format!(
                "Expected an object of type {:?}, found {:?}",
                obj.name(),
                type_name::<V>(),
            ))
        })
    }
}

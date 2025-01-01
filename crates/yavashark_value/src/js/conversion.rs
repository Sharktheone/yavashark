use crate::{
    AsAny, BoxedObj, Error, Obj, Object, ObjectProperty, Realm, Value, Variable,
};
use std::any::type_name;
use std::fmt::Debug;
use yavashark_garbage::collectable::{OwningGcMutRefCellGuard, OwningGcRefCellGuard};

impl<C: Realm> From<&str> for Value<C> {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl<C: Realm> From<String> for Value<C> {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl<C: Realm> From<&String> for Value<C> {
    fn from(s: &String) -> Self {
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

impl<C: Realm> From<f32> for Value<C> {
    fn from(n: f32) -> Self {
        Self::Number(f64::from(n))
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

pub trait FromValue<C: Realm>: Sized {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>>;
}


pub trait IntoValue<C: Realm> {
    fn into_value(self) -> Value<C>;
}




impl<C: Realm> IntoValue<C> for Value<C> {
    fn into_value(self) -> Value<C> {
        self
    }
}

impl<C: Realm> FromValue<C> for Value<C> {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        Ok(value)
    }
}


impl<C: Realm> FromValue<C> for String {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        match value {
            Value::String(s) => Ok(s),
            _ => Err(Error::ty_error(format!(
                "Expected a string, found {value:?}"
            ))),
        }
    }
}

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
        Value::String(self)
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
                    Value::Number(self as f64)
                }
            }
        )*
    };
    () => {};
}

impl_from_value!(u8, u16, u32, u64, i8, i16, i32, i64, usize, isize, f32, f64);

impl<R: Realm, O: Obj<R>> FromValue<R> for OwningGcRefCellGuard<'_, BoxedObj<R>, O> {
    fn from_value(value: Value<R>) -> Result<Self, Error<R>> {
        let Value::Object(obj) = value else {
            return Err(Error::ty_error(format!(
                "Expected a number, found {value:?}"
            )));
        };

        obj.get_owned()?
            .maybe_map(|this| {
                let any = this.as_any();

                any.downcast_ref()
            })
            .map_err(|other| {
                Error::ty_error(format!(
                    "Expected {}, found {}",
                    type_name::<O>(),
                    other.class_name()
                ))
            })
    }
}

impl<R: Realm, O: Obj<R>> FromValue<R> for OwningGcMutRefCellGuard<'_, BoxedObj<R>, O> {
    fn from_value(value: Value<R>) -> Result<Self, Error<R>> {
        let Value::Object(obj) = value else {
            return Err(Error::ty_error(format!(
                "Expected a number, found {value:?}"
            )));
        };

        obj.get_owned_mut()?
            .maybe_map(|this| {
                let any = this.as_any_mut();

                any.downcast_mut()
            })
            .map_err(|other| {
                Error::ty_error(format!(
                    "Expected {}, found {}",
                    type_name::<O>(),
                    other.class_name()
                ))
            })
    }
}
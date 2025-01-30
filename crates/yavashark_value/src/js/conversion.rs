use crate::{BoxedObj, Error, Obj, Object, Realm, Symbol, Value};
use std::any::type_name;
use num_bigint::BigInt;
use yavashark_garbage::OwningGcGuard;

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

impl<C: Realm> From<BigInt> for Value<C> {
    fn from(n: BigInt) -> Self {
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

pub trait FromValue<C: Realm>: Sized {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>>;
}

pub trait IntoValue<C: Realm> {
    fn into_value(self) -> Value<C>;
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

impl<C: Realm> FromValue<C> for BigInt {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        match value {
            Value::BigInt(n) => Ok(n),
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

impl<C: Realm> IntoValue<C> for Symbol {
    fn into_value(self) -> Value<C> {
        Value::Symbol(self)
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

impl_from_value!(u8, u16, u32, u64, i8, i16, i32, i64, usize, isize, f32, f64);

impl<C: Realm, V: Obj<C>> FromValue<C> for OwningGcGuard<'_, BoxedObj<C>, V> {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        let obj = match value {
            Value::Object(obj) => Ok(obj.get_owning()),
            _ => Err(Error::ty_error(format!(
                "Expected an object, found {value:?}"
            ))),
        }?;

        obj.maybe_map(|o| (**o).as_any().downcast_ref::<V>())
            .map_err(|obj| {
                Error::ty_error(format!(
                    "Expected an object of type {:?}, found {:?}",
                    obj.name(),
                    type_name::<V>(),
                ))
            })
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    
    
    #[derive(Debug, PartialEq, Eq, Clone)]
    struct R;
    
    impl Realm for R {}
    
    
    trait FromValue2<C: Realm>: Sized {
        type Output: IntoValue2;
        
        fn from_value(value: Value<C>) -> Result<Self::Output, Error<C>>;
    }
    
    
    trait IntoValue2 {}
    
    impl IntoValue2 for String {}
    impl IntoValue2 for i32 {}
    impl IntoValue2 for bool {}
    impl IntoValue2 for f64 {}
    
    
    impl<T: FromValue<R> + IntoValue2> FromValue2<R> for T {
        type Output = T;
        
        fn from_value(value: Value<R>) -> Result<Self::Output, Error<R>> {
            T::from_value(value)
        }
    }
    
    fn extract_value<T: FromValue2<R>>(vals: &mut [Value<R>], idx: usize) -> Result<Option<T::Output>, Error<R>> {
        let Some(val) = vals.get_mut(idx) else {
            return Ok(None);
        };
        
        let val = std::mem::replace(val, Value::Undefined);
        
        Ok(Some(T::from_value(val)?))
    }
    
    trait OptionalConvert<T>: Sized {
        fn convert(this: Option<T>) -> Result<Self, Error<R>>;
    }
    
    impl<T: Sized + IntoValue2> OptionalConvert<T> for Option<T> {
        fn convert(this: Option<T>) -> Result<Self, Error<R>> {
            Ok(this)
        }
    }
    
    impl<T: Sized + IntoValue2> OptionalConvert<T> for T {
        fn convert(this: Option<T>) -> Result<Self, Error<R>> {
            match this {
                Some(val) => Ok(val),
                None => Err(Error::ty_error("Expected a value".to_owned())),
            }
        }
    }
    
    #[test]
    fn test_from_str() {
        
        let mut values: Vec<Value<R>> = vec![
            Value::from("hello"),
            Value::from(8),
            Value::from(true),
        ];
        
        let a = extract_value::<String>(&mut values, 0).unwrap();
        let b = extract_value::<i32>(&mut values, 1).unwrap();
        let c = extract_value::<bool>(&mut values, 2).unwrap();
        let d = extract_value::<f64>(&mut values, 3).unwrap();
        
        
        
        test(OptionalConvert::convert(a).unwrap(), OptionalConvert::convert(b).unwrap(), OptionalConvert::convert(c).unwrap(), OptionalConvert::convert(d).unwrap());
        
    }
    
    
    fn test(a: String, b: i32, c: bool, d: Option<f64>) {
        println!("{} {} {} {:?}", a, b, c, d);
        
    }
}

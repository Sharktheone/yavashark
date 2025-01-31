use crate::{BoxedObj, Error, Obj, Object, Realm, Symbol, Value};
use std::any::type_name;
use num_bigint::BigInt;
use yavashark_garbage::OwningGcGuard;
use std::any::Any;

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
    use std::cell::{Cell, UnsafeCell};
    use std::fmt::Debug;
    use std::mem;
    use crate::{ObjectProperty, Variable};
    use super::*;


    #[derive(Debug, PartialEq, Eq, Clone)]
    struct R;

    impl Realm for R {}


    trait FromValue2<C: Realm>: Sized {
        type Output;
        fn from_value(value: Value<C>) -> Result<Self::Output, Error<C>>;
    }


    impl FromValue2<R> for String {
        type Output = String;

        fn from_value(value: Value<R>) -> Result<Self::Output, Error<R>> {
            match value {
                Value::String(s) => Ok(s),
                _ => Err(Error::ty_error(format!(
                    "Expected a string, found {value:?}"
                ))),
            }
        }
    }

    impl FromValue2<R> for i32 {
        type Output = i32;

        fn from_value(value: Value<R>) -> Result<Self::Output, Error<R>> {
            match value {
                Value::Number(n) => Ok(n as i32),
                _ => Err(Error::ty_error(format!(
                    "Expected a number, found {value:?}"
                ))),
            }
        }
    }

    impl FromValue2<R> for f64 {
        type Output = f64;

        fn from_value(value: Value<R>) -> Result<Self::Output, Error<R>> {
            match value {
                Value::Number(n) => Ok(n),
                _ => Err(Error::ty_error(format!(
                    "Expected a number, found {value:?}"
                ))),
            }
        }
    }

    impl FromValue2<R> for bool {
        type Output = bool;

        fn from_value(value: Value<R>) -> Result<Self::Output, Error<R>> {
            match value {
                Value::Boolean(b) => Ok(b),
                _ => Err(Error::ty_error(format!(
                    "Expected a boolean, found {value:?}"
                ))),
            }
        }
    }

    impl FromValue2<R> for &str {
        type Output = String;

        fn from_value(value: Value<R>) -> Result<Self::Output, Error<R>> {
            match value {
                Value::String(s) => Ok(s),
                _ => Err(Error::ty_error(format!(
                    "Expected a string, found {value:?}"
                ))),
            }
        }
    }

    fn extract_value<T: FromValue2<R>>(vals: &mut [Value<R>], idx: usize) -> Result<Option<T::Output>, Error<R>> {
        let Some(val) = vals.get_mut(idx) else {
            return Ok(None);
        };

        let val = std::mem::replace(val, Value::Undefined);

        Ok(Some(T::from_value(val)?))
    }
    
    impl<T: Obj<R>> FromValue2<R> for T {
        type Output = OwningGcGuard<'static, BoxedObj<R>, T>;

        fn from_value(value: Value<R>) -> Result<Self::Output, Error<R>> {
            let obj = match value {
                Value::Object(obj) => Ok(obj.get_owning()),
                _ => Err(Error::ty_error(format!(
                    "Expected an object, found {value:?}"
                ))),
            }?;

            obj.maybe_map(|o| (**o).as_any().downcast_ref::<T>())
                .map_err(|obj| {
                    Error::ty_error(format!(
                        "Expected an object of type {:?}, found {:?}",
                        obj.name(),
                        type_name::<T>(),
                    ))
                })
        }

    }

    struct Extractor {
        values: Vec<UnsafeCell<Value<R>>>,
        idx: Cell<usize>,
    }

    impl Extractor {
        fn new(values: Vec<Value<R>>) -> Self {
            Self {
                values: values.into_iter().map(|v| UnsafeCell::new(v)).collect(),
                idx: Cell::new(0),
            }
        }
    }

    trait ExtractValue<T>: Sized {
        type Output;
        fn extract(&self) -> Result<Self::Output, Error<R>>;
    }

    impl<T: FromValue2<R>> ExtractValue<T> for Extractor {
        type Output = T::Output;
        fn extract(&self) -> Result<Self::Output, Error<R>> {
            let idx = self.idx.get();
            self.idx.set(idx + 1);

            let val = self.values.get(idx).ok_or_else(|| Error::ty_error("Expected a value".to_owned()))?;

            let val = unsafe { &mut *val.get() };

            let val = mem::replace(val, Value::Undefined);



            T::from_value(val)
        }
    }

    impl<T: FromValue2<R>> ExtractValue<Option<T>> for Extractor {

        type Output = Option<T::Output>;

        fn extract(&self) -> Result<Self::Output, Error<R>> {
            let idx = self.idx.get();
            self.idx.set(idx + 1);

            let Some(val) = self.values.get(idx) else {
                return Ok(None);
            };

            let val = unsafe { &mut *val.get() };

            let val = mem::replace(val, Value::Undefined);

            Ok(Some(T::from_value(val)?))
        }
    }

    #[derive(Debug, PartialEq, Eq, Clone)]
    struct CustomObj;

    impl Obj<R> for CustomObj {
        fn define_property(&self, name: Value<R>, value: Value<R>) -> Result<(), Error<R>> {
            Ok(())
        }

        fn define_variable(&self, name: Value<R>, value: Variable<R>) -> Result<(), Error<R>> {
            Ok(())
        }

        fn resolve_property(&self, name: &Value<R>) -> Result<Option<ObjectProperty<R>>, Error<R>> {
            Ok(None)
        }

        fn get_property(&self, name: &Value<R>) -> Result<Option<ObjectProperty<R>>, Error<R>> {
            Ok(None)
        }

        fn define_getter(&self, name: Value<R>, value: Value<R>) -> Result<(), Error<R>> {
            Ok(())
        }

        fn define_setter(&self, name: Value<R>, value: Value<R>) -> Result<(), Error<R>> {
            Ok(())
        }

        fn get_getter(&self, name: &Value<R>) -> Result<Option<Value<R>>, Error<R>> {
            Ok(None)
        }

        fn get_setter(&self, name: &Value<R>) -> Result<Option<Value<R>>, Error<R>> {
            Ok(None)
        }

        fn delete_property(&self, name: &Value<R>) -> Result<Option<Value<R>>, Error<R>> {
            Ok(None)
        }

        fn name(&self) -> String {
            "CustomObj".to_owned()
        }

        fn to_string(&self, realm: &mut R) -> Result<String, Error<R>> {
            Ok("CustomObj".to_owned())
        }

        fn to_string_internal(&self) -> Result<String, Error<R>> {
            Ok("CustomObj".to_owned())
        }

        fn properties(&self) -> Result<Vec<(Value<R>, Value<R>)>, Error<R>> {
            Ok(Vec::new())
        }

        fn keys(&self) -> Result<Vec<Value<R>>, Error<R>> {
            Ok(Vec::new())
        }

        fn values(&self) -> Result<Vec<Value<R>>, Error<R>> {
            Ok(Vec::new())
        }

        fn get_array_or_done(&self, index: usize) -> Result<(bool, Option<Value<R>>), Error<R>> {
            Ok((false, None))
        }

        fn clear_values(&self) -> Result<(), Error<R>> {
            Ok(())
        }
    }


    #[test]
    fn test_extract() {
        let values: Vec<Value<R>> = vec![
            Value::from("hello"),
            Value::from(8),
            Value::from(true),
            CustomObj.into_value()
        ];

        let extractor = Extractor::new(values);

        let a = ExtractValue::<&str>::extract(&extractor).unwrap();
        let b = ExtractValue::<i32>::extract(&extractor).unwrap();
        let c = ExtractValue::<bool>::extract(&extractor).unwrap();
        let d = ExtractValue::<CustomObj>::extract(&extractor).unwrap();
        let e = ExtractValue::<Option<CustomObj>>::extract(&extractor).unwrap();

        let e = e.as_ref().map(|e| &**e);

        test(&a, b, c, &d, e);

    }

    fn test(a: &str, b: i32, c: bool, d: &CustomObj, e: Option<&CustomObj>) {
        println!("{} {} {} {:?} {:?}", a, b, c, d, e);

    }
}

use crate::{Error, ObjectHandle, Realm, Res, Symbol, Value, ValueResult};
use num_bigint::BigInt;
use std::mem;
use std::slice::IterMut;
use yavashark_garbage::OwningGcGuard;
use yavashark_value::{BoxedObj, FromValue, IntoValue, Obj};

pub trait TryIntoValue: Sized {
    fn try_into_value(self) -> ValueResult;
}

impl<T: IntoValue<Realm>> TryIntoValue for T {
    fn try_into_value(self) -> ValueResult {
        Ok(self.into_value())
    }
}

impl<T: TryIntoValue> TryIntoValue for Res<T, Error> {
    fn try_into_value(self) -> ValueResult {
        self?.try_into_value()
    }
}

pub trait FromValueOutput {
    type Output;
    fn from_value_out(value: Value) -> Res<Self::Output>;
}

// TODO: this might work in future rust versions with specialization, but unfortunately not at this time...
// default impl<T: FromValue<Realm>> FromValueOutput for T {
//     type Output = T;
//
//     fn from_value_out(value: Value) -> Result<Self::Output> {
//         T::from_value(value)
//     }
// }

impl<O: Obj<Realm>> FromValueOutput for &O {
    type Output = OwningGcGuard<'static, BoxedObj<Realm>, O>;

    fn from_value_out(value: Value) -> Res<Self::Output> {
        FromValue::from_value(value)
    }
}

impl FromValueOutput for ObjectHandle {
    type Output = Self;

    fn from_value_out(value: Value) -> Res<Self::Output> {
        match value {
            Value::Object(obj) => Ok(obj),
            _ => Err(Error::ty_error(format!("Expected object, found {value:?}"))),
        }
    }
}

impl FromValueOutput for &ObjectHandle {
    type Output = ObjectHandle;

    fn from_value_out(value: Value) -> Res<Self::Output> {
        match value {
            Value::Object(obj) => Ok(obj),
            _ => Err(Error::ty_error(format!("Expected object, found {value:?}"))),
        }
    }
}

impl FromValueOutput for Value {
    type Output = Self;
    fn from_value_out(value: Value) -> Res<Self::Output> {
        Ok(value)
    }
}

impl FromValueOutput for &Value {
    type Output = Value;
    fn from_value_out(value: Value) -> Res<Self::Output> {
        Ok(value)
    }
}

impl FromValueOutput for () {
    type Output = ();
    fn from_value_out(_value: Value) -> Res<Self::Output> {
        Ok(())
    }
}

impl FromValueOutput for bool {
    type Output = Self;
    fn from_value_out(value: Value) -> Res<Self::Output> {
        match value {
            Value::Boolean(b) => Ok(b),
            _ => Err(Error::ty_error(format!(
                "Expected boolean, found {value:?}"
            ))),
        }
    }
}

impl FromValueOutput for String {
    type Output = Self;
    fn from_value_out(value: Value) -> Res<Self::Output> {
        value.to_string_no_realm()
    }
}

impl FromValueOutput for &str {
    type Output = String;
    fn from_value_out(value: Value) -> Res<Self::Output> {
        String::from_value_out(value)
    }
}

impl FromValueOutput for Symbol {
    type Output = Self;
    fn from_value_out(value: Value) -> Res<Self::Output> {
        match value {
            Value::Symbol(s) => Ok(s),
            _ => Err(Error::ty_error(format!("Expected symbol, found {value:?}"))),
        }
    }
}

impl FromValueOutput for BigInt {
    type Output = Self;
    fn from_value_out(value: Value) -> Res<Self::Output> {
        match value {
            Value::BigInt(n) => Ok(n),
            _ => Err(Error::ty_error(format!("Expected bigint, found {value:?}"))),
        }
    }
}

impl FromValueOutput for &BigInt {
    type Output = BigInt;
    fn from_value_out(value: Value) -> Res<Self::Output> {
        BigInt::from_value_out(value)
    }
}

macro_rules! impl_from_value_output {
    ($($t:ty),*) => {
        $(
            impl FromValueOutput for $t {
                type Output = $t;

                fn from_value_out(value: Value) -> Res<Self::Output> {
                    match value {
                        Value::Number(n) => Ok(n as $t),
                        Value::String(ref s) => s.parse().map_err(|_| Error::ty_error(format!("Expected a number, found {value:?}"))),
                        _ => Err(Error::ty_error(format!("Expected a number, found {value:?}"))),
                    }
                }
            }
        )*
    };
    () => {};
}

impl_from_value_output!(u8, u16, u32, u64, i8, i16, i32, i64, usize, isize, f32, f64);

pub struct Extractor<'a> {
    values: IterMut<'a, Value>,
}

impl<'a> Extractor<'a> {
    pub fn new(values: &'a mut [Value]) -> Self {
        Self {
            values: values.iter_mut(),
        }
    }
}

pub trait ExtractValue<T>: Sized {
    type Output;
    fn extract(&mut self) -> Res<Self::Output>;
}

impl<T: FromValueOutput> ExtractValue<T> for Extractor<'_> {
    type Output = T::Output;
    fn extract(&mut self) -> Res<Self::Output> {
        let val = self
            .values
            .next()
            .map_or(Value::Undefined, |val| mem::replace(val, Value::Undefined));

        T::from_value_out(val)
    }
}

impl<T: FromValueOutput> ExtractValue<Option<T>> for Extractor<'_> {
    type Output = Option<T::Output>;

    fn extract(&mut self) -> Res<Self::Output> {
        let Some(val) = self.values.next() else {
            return Ok(None);
        };

        let val = mem::replace(val, Value::Undefined);

        Ok(Some(T::from_value_out(val)?))
    }
}

impl<T: FromValueOutput> ExtractValue<&Option<T>> for Extractor<'_> {
    type Output = Option<T::Output>;

    fn extract(&mut self) -> Res<Self::Output> {
        ExtractValue::<Option<T>>::extract(self)
    }
}

impl<T: FromValueOutput> ExtractValue<Vec<T>> for Extractor<'_> {
    type Output = Vec<T::Output>;

    fn extract(&mut self) -> Res<Self::Output> {
        let mut vec = Vec::new();
        for val in &mut self.values {
            let val = mem::replace(val, Value::Undefined);
            vec.push(T::from_value_out(val)?);
        }

        Ok(vec)
    }
}
impl<T: FromValueOutput> ExtractValue<&'_ [T]> for Extractor<'_> {
    type Output = Vec<T::Output>;

    fn extract(&mut self) -> Res<Self::Output> {
        let mut vec = Vec::new();
        for val in &mut self.values {
            let val = mem::replace(val, Value::Undefined);
            vec.push(T::from_value_out(val)?);
        }

        Ok(vec)
    }
}

use crate::utils::ValueIterator;
use crate::{Error, Realm, Result, Symbol, Value, ValueResult};
use num_bigint::BigInt;
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

impl<T: TryIntoValue> TryIntoValue for Result<T, Error> {
    fn try_into_value(self) -> ValueResult {
        self?.try_into_value()
    }
}

pub trait FromValueOutput {
    type Output;
    fn from_value_out(value: Value) -> Result<Self::Output>;
}

// TODO: this might work in future rust versions with specialization, but unfortunately not at this time...
// default impl<T: FromValue<Realm>> FromValueOutput for T {
//     type Output = T;
//
//     fn from_value_out(value: Value) -> Result<Self::Output> {
//         T::from_value(value)
//     }
// }

impl<O: Obj<Realm>> FromValueOutput for O {
    type Output = OwningGcGuard<'static, BoxedObj<Realm>, O>;

    fn from_value_out(value: Value) -> Result<Self::Output> {
        FromValue::from_value(value)
    }
}

impl FromValueOutput for Value {
    type Output = Self;
    fn from_value_out(value: Value) -> Result<Self::Output> {
        Ok(value)
    }
}

impl FromValueOutput for () {
    type Output = ();
    fn from_value_out(_value: Value) -> Result<Self::Output> {
        Ok(())
    }
}

impl FromValueOutput for bool {
    type Output = bool;
    fn from_value_out(value: Value) -> Result<Self::Output> {
        match value {
            Value::Boolean(b) => Ok(b),
            _ => Err(Error::ty_error(format!(
                "Expected boolean, found {value:?}"
            ))),
        }
    }
}

impl FromValueOutput for String {
    type Output = String;
    fn from_value_out(value: Value) -> Result<Self::Output> {
        match value {
            Value::String(s) => Ok(s),
            _ => Err(Error::ty_error(format!("Expected string, found {value:?}"))),
        }
    }
}

impl FromValueOutput for Symbol {
    type Output = Symbol;
    fn from_value_out(value: Value) -> Result<Self::Output> {
        match value {
            Value::Symbol(s) => Ok(s),
            _ => Err(Error::ty_error(format!("Expected symbol, found {value:?}"))),
        }
    }
}

impl FromValueOutput for BigInt {
    type Output = BigInt;
    fn from_value_out(value: Value) -> Result<Self::Output> {
        match value {
            Value::BigInt(n) => Ok(n),
            _ => Err(Error::ty_error(format!("Expected bigint, found {value:?}"))),
        }
    }
}

macro_rules! impl_from_value_output {
    ($($t:ty),*) => {
        $(
            impl FromValueOutput for $t {
                type Output = $t;

                fn from_value_out(value: Value) -> Result<Self::Output> {
                    match value {
                        Value::Number(n) => Ok(n as $t),
                        _ => Err(Error::ty_error(format!("Expected a number, found {value:?}"))),
                    }
                }
            }
        )*
    };
    () => {};
}

impl_from_value_output!(u8, u16, u32, u64, i8, i16, i32, i64, usize, isize, f32, f64);

use yavashark_garbage::OwningGcGuard;
use crate::{Error, Realm, ValueResult, Value, Result};
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
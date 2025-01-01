use yavashark_value::IntoValue;
use crate::{Error, Realm, ValueResult};

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


use std::str::FromStr;
use temporal_rs::Calendar;
use crate::conversion::FromValueOutput;
use crate::{Error, Realm, Res, Value};

impl FromValueOutput for Calendar {
    type Output = Self;

    fn from_value_out(value: Value, realm: &mut Realm) -> Res<Self::Output> {
        let Value::String(calendar) = value else {
            return Err(Error::ty(
                "Calendar value must be a string",
            ));
        };

        Calendar::from_str(&calendar)
            .map_err(Error::from_temporal)
    }
}
use crate::conversion::FromValueOutput;
use crate::{Error, Realm, Res, Value};
use temporal_rs::TimeZone;

impl FromValueOutput for TimeZone {
    type Output = Self;

    fn from_value_out(value: Value, _realm: &mut Realm) -> Res<Self::Output> {
        let tz_str = value.as_string()?;

        Self::try_from_str(&tz_str.as_str_lossy()).map_err(Error::from_temporal)
    }
}

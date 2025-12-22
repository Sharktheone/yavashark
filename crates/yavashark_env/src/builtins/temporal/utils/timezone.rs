use crate::conversion::FromValueOutput;
use crate::{Error, Realm, Res, Value};
use temporal_rs::TimeZone;

impl FromValueOutput for TimeZone {
    type Output = Self;

    fn from_value_out(value: Value, realm: &mut Realm) -> Res<Self::Output> {
        let tz_str = value.to_string(realm)?;

        Self::try_from_str(&tz_str).map_err(Error::from_temporal)
    }
}

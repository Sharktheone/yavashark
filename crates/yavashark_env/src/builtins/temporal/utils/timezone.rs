use crate::conversion::FromValueOutput;
use crate::{Error, Realm, Res, Value};
use temporal_rs::TimeZone;

impl FromValueOutput for TimeZone {
    type Output = Self;

    fn from_value_out(value: Value, _realm: &mut Realm) -> Res<Self::Output> {
        let tz_str = match value {
            Value::String(value) => value,
            Value::Object(obj) => {
                if let Some(zdt) = obj.downcast::<NativeObject<ZonedDateTime>>() {
                    return Ok(*zdt.date.time_zone());
                }
                return Err(Error::ty(
                    "Time zone object must be a Temporal.ZonedDateTime",
                ));
            }
            _ => return Err(Error::ty("Time zone must be a string or object")),
        };

        let identifier = tz_str.as_str_lossy();
        Self::try_from_str(&identifier).map_err(Error::from_temporal)
    }
}

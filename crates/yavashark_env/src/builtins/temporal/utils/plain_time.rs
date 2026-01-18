use crate::builtins::{PlainDateTime, PlainTime};
use crate::conversion::FromValueOutput;
use crate::native_obj::NativeObject;
use crate::{Error, Realm, Res, Value};

impl FromValueOutput for temporal_rs::PlainTime {
    type Output = Self;

    fn from_value_out(value: Value, realm: &mut Realm) -> Res<Self::Output> {
        match value {
            Value::Object(obj) => {
                if let Some(plain_time) = obj.downcast::<NativeObject<PlainTime>>() {
                    return Ok(plain_time.time);
                }

                if let Some(plain_date_time) = obj.downcast::<NativeObject<PlainDateTime>>() {
                    return Ok(plain_date_time.date.to_plain_time());
                }

                // Try to parse as a property bag
                let hour = obj.get("hour", realm).and_then(|v| v.to_number(realm))? as u8;
                let minute = obj.get("minute", realm).and_then(|v| v.to_number(realm))? as u8;
                let second = obj.get("second", realm).and_then(|v| v.to_number(realm))? as u8;
                let millisecond = obj
                    .get("millisecond", realm)
                    .and_then(|v| v.to_number(realm))? as u16;
                let microsecond = obj
                    .get("microsecond", realm)
                    .and_then(|v| v.to_number(realm))? as u16;
                let nanosecond = obj
                    .get("nanosecond", realm)
                    .and_then(|v| v.to_number(realm))? as u16;

                Self::new(hour, minute, second, millisecond, microsecond, nanosecond)
                    .map_err(Error::from_temporal)
            }
            Value::String(s) => s.parse().map_err(Error::from_temporal),
            _ => Err(Error::ty(
                "PlainTime value must be a string or a PlainTime-like object",
            )),
        }
    }
}

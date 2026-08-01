use crate::builtins::{PlainDate, PlainDateTime, ZonedDateTime, value_to_partial_date};
use crate::conversion::FromValueOutput;
use crate::native_obj::NativeObject;
use crate::{Error, Realm, Res, Value};

impl FromValueOutput for temporal_rs::PlainDate {
    type Output = Self;

    fn from_value_out(value: Value, realm: &mut Realm) -> Res<Self::Output> {
        match value {
            Value::Object(obj) => {
                if let Some(plain_date) = obj.downcast::<NativeObject<PlainDate>>() {
                    return Ok(plain_date.date.clone());
                }

                if let Some(date_time) = obj.downcast::<NativeObject<PlainDateTime>>() {
                    return Ok(date_time.date.to_plain_date());
                }

                if let Some(zoned_date_time) = obj.downcast::<NativeObject<ZonedDateTime>>() {
                    return Ok(zoned_date_time.date.to_plain_date());
                }

                let partial = value_to_partial_date(&obj, realm)?;
                Self::from_partial(partial, None).map_err(Error::from_temporal)
            }
            Value::String(s) => s.parse().map_err(Error::from_temporal),
            _ => Err(Error::ty(
                "PlainDate value must be a string or a PlainDate-like object",
            )),
        }
    }
}

use crate::builtins::{value_to_zoned_date_time, PlainDate};
use crate::conversion::FromValueOutput;
use crate::native_obj::NativeObject;
use crate::{Error, Realm, Res, Value};
use temporal_rs::options::RelativeTo;

impl FromValueOutput for RelativeTo {
    type Output = Self;

    fn from_value_out(value: Value, realm: &mut Realm) -> Res<Self::Output> {
        match value {
            Value::Object(obj) => {
                if let Some(pd) = obj.downcast::<NativeObject<PlainDate>>() {
                    return Ok(Self::PlainDate(pd.date.clone()));
                }

                // Try to parse as ZonedDateTime
                let zdt = value_to_zoned_date_time(&obj.into(), None, realm)?;
                Ok(Self::ZonedDateTime(zdt))
            }
            Value::String(s) => {
                Self::try_from_str(&*s.as_str_lossy()).map_err(Error::from_temporal)
            }
            _ => Err(Error::ty(
                "RelativeTo must be a PlainDate, ZonedDateTime, or a string",
            )),
        }
    }
}

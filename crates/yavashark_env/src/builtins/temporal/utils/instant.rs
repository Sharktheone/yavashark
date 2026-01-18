use crate::builtins::Instant;
use crate::conversion::FromValueOutput;
use crate::native_obj::NativeObject;
use crate::{Error, Realm, Res, Value};

impl FromValueOutput for temporal_rs::Instant {
    type Output = Self;

    fn from_value_out(value: Value, realm: &mut Realm) -> Res<Self::Output> {
        match value {
            Value::Object(obj) => {
                if let Some(instant) = obj.downcast::<NativeObject<Instant>>() {
                    return Ok(instant.stamp);
                }

                // Try to convert to string and parse
                let s = obj.to_string(realm)?;
                s.parse().map_err(Error::from_temporal)
            }
            Value::String(s) => s.parse().map_err(Error::from_temporal),
            _ => Err(Error::ty(
                "Instant value must be a string or a Temporal.Instant object",
            )),
        }
    }
}

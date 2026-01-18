use crate::builtins::temporal::utils::overflow_options;
use crate::builtins::{value_to_partial_date, PlainDate};
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

                // Check if it's a property bag with year/month/day
                if obj.contains_key("year".into(), realm)?
                    && (obj.contains_key("month".into(), realm)?
                        || obj.contains_key("monthCode".into(), realm)?)
                    && obj.contains_key("day".into(), realm)?
                {
                    let partial = value_to_partial_date(&obj, realm)?;
                    let overflow = overflow_options(&obj, realm)?;

                    return Self::from_partial(partial, overflow).map_err(Error::from_temporal);
                }

                Err(Error::ty(
                    "PlainDate object must have year, month (or monthCode), and day properties",
                ))
            }
            Value::String(s) => s.parse().map_err(Error::from_temporal),
            _ => Err(Error::ty(
                "PlainDate value must be a string or a PlainDate-like object",
            )),
        }
    }
}

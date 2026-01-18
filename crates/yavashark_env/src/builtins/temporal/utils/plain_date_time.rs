use crate::builtins::PlainDateTime;
use crate::conversion::FromValueOutput;
use crate::native_obj::NativeObject;
use crate::{Error, Realm, Res, Value};
use temporal_rs::Calendar;

impl FromValueOutput for temporal_rs::PlainDateTime {
    type Output = Self;

    fn from_value_out(value: Value, realm: &mut Realm) -> Res<Self::Output> {
        match value {
            Value::Object(obj) => {
                if let Some(pdt) = obj.downcast::<NativeObject<PlainDateTime>>() {
                    return Ok(pdt.date.clone());
                }

                // Check if it's a property bag with date/time fields
                if obj.contains_key("year".into(), realm)?
                    || obj.contains_key("month".into(), realm)?
                    || obj.contains_key("monthCode".into(), realm)?
                    || obj.contains_key("day".into(), realm)?
                {
                    let year = obj
                        .resolve_property("year", realm)?
                        .map_or(Ok(0), |v| v.to_number(realm).map(|v| v as i32))?;
                    let month = obj
                        .resolve_property("month", realm)?
                        .map_or(Ok(0), |v| v.to_number(realm).map(|v| v as u8))?;

                    let month = if month == 0 {
                        obj.resolve_property("monthCode", realm)?
                            .and_then(|v| v.to_string(realm).ok())
                            .and_then(|s| {
                                if s.is_empty() {
                                    None
                                } else {
                                    s.as_str_lossy()[1..].parse::<u8>().ok()
                                }
                            })
                            .unwrap_or(0)
                    } else {
                        month
                    };

                    let day = obj
                        .resolve_property("day", realm)?
                        .map_or(Ok(0), |v| v.to_number(realm).map(|v| v as u8))?;
                    let hour = obj
                        .resolve_property("hour", realm)?
                        .map_or(Ok(0), |v| v.to_number(realm).map(|v| v as u8))?;
                    let minute = obj
                        .resolve_property("minute", realm)?
                        .map_or(Ok(0), |v| v.to_number(realm).map(|v| v as u8))?;
                    let second = obj
                        .resolve_property("second", realm)?
                        .map_or(Ok(0), |v| v.to_number(realm).map(|v| v as u8))?;
                    let millisecond = obj
                        .resolve_property("millisecond", realm)?
                        .map_or(Ok(0), |v| v.to_number(realm).map(|v| v as u16))?;
                    let microsecond = obj
                        .resolve_property("microsecond", realm)?
                        .map_or(Ok(0), |v| v.to_number(realm).map(|v| v as u16))?;
                    let nanosecond = obj
                        .resolve_property("nanosecond", realm)?
                        .map_or(Ok(0), |v| v.to_number(realm).map(|v| v as u16))?;

                    let calendar = obj
                        .extract_opt::<Calendar>("calendar", realm)?
                        .unwrap_or_default();

                    return Self::new(
                        year,
                        month,
                        day,
                        hour,
                        minute,
                        second,
                        millisecond,
                        microsecond,
                        nanosecond,
                        calendar,
                    )
                    .map_err(Error::from_temporal);
                }

                Err(Error::ty(
                    "PlainDateTime object must have date fields (year, month/monthCode, day)",
                ))
            }
            Value::String(s) => s.parse().map_err(Error::from_temporal),
            _ => Err(Error::ty(
                "PlainDateTime value must be a string or a PlainDateTime-like object",
            )),
        }
    }
}

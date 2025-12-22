use crate::builtins::{PlainDate, PlainDateTime, PlainMonthDay, PlainYearMonth, ZonedDateTime};
use crate::conversion::FromValueOutput;
use crate::{Error, Realm, Res, Value};
use std::str::FromStr;
use temporal_rs::Calendar;

impl FromValueOutput for Calendar {
    type Output = Self;

    fn from_value_out(value: Value, _: &mut Realm) -> Res<Self::Output> {
        match value {
            Value::Object(obj) => {
                if let Some(calendar_like) = obj.downcast::<PlainDate>() {
                    return Ok(calendar_like.date.calendar().clone());
                }

                if let Some(calendar_like) = obj.downcast::<PlainDateTime>() {
                    return Ok(calendar_like.date.calendar().clone());
                }

                if let Some(calendar_like) = obj.downcast::<PlainYearMonth>() {
                    return Ok(calendar_like.year_month.calendar().clone());
                }

                if let Some(calendar_like) = obj.downcast::<PlainMonthDay>() {
                    return Ok(calendar_like.month_day.calendar().clone());
                }

                if let Some(calendar_like) = obj.downcast::<ZonedDateTime>() {
                    return Ok(calendar_like.date.calendar().clone());
                }

                Err(Error::ty(
                    "Calendar object must be a 'calendar like' Temporal object",
                ))
            }
            Value::String(s) => Self::from_str(&s).map_err(Error::from_temporal),
            _ => Err(Error::ty(
                "Calendar value must be a string or a 'calendar like' value",
            )),
        }
    }
}

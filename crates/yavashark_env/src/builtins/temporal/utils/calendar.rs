use crate::builtins::{PlainDate, PlainDateTime, PlainMonthDay, PlainYearMonth, ZonedDateTime};
use crate::conversion::FromValueOutput;
use crate::native_obj::NativeObject;
use crate::{Error, Realm, Res, Value};
use temporal_rs::Calendar;

pub struct CalendarIdentifier(pub Calendar);

impl FromValueOutput for CalendarIdentifier {
    type Output = Self;

    fn from_value_out(value: Value, _: &mut Realm) -> Res<Self::Output> {
        let Value::String(identifier) = value else {
            return Err(Error::ty("Calendar identifier must be a string"));
        };
        let calendar: Calendar = identifier.parse().map_err(Error::from_temporal)?;
        if !identifier
            .as_str_lossy()
            .eq_ignore_ascii_case(calendar.identifier())
        {
            return Err(Error::range("Invalid calendar identifier"));
        }
        Ok(Self(calendar))
    }
}

impl FromValueOutput for Calendar {
    type Output = Self;

    fn from_value_out(value: Value, _: &mut Realm) -> Res<Self::Output> {
        match value {
            Value::Object(obj) => {
                if let Some(calendar_like) = obj.downcast::<NativeObject<PlainDate>>() {
                    return Ok(calendar_like.date.calendar().clone());
                }

                if let Some(calendar_like) = obj.downcast::<NativeObject<PlainDateTime>>() {
                    return Ok(calendar_like.date.calendar().clone());
                }

                if let Some(calendar_like) = obj.downcast::<NativeObject<PlainYearMonth>>() {
                    return Ok(calendar_like.year_month.calendar().clone());
                }

                if let Some(calendar_like) = obj.downcast::<NativeObject<PlainMonthDay>>() {
                    return Ok(calendar_like.month_day.calendar().clone());
                }

                if let Some(calendar_like) = obj.downcast::<NativeObject<ZonedDateTime>>() {
                    return Ok(calendar_like.date.calendar().clone());
                }

                Err(Error::ty(
                    "Calendar object must be a 'calendar like' Temporal object",
                ))
            }
            Value::String(s) => s.parse().map_err(Error::from_temporal),
            _ => Err(Error::ty(
                "Calendar value must be a string or a 'calendar like' value",
            )),
        }
    }
}

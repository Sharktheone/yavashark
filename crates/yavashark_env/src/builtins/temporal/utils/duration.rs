use crate::builtins::Duration;
use crate::conversion::FromValueOutput;
use crate::native_obj::NativeObject;
use crate::{Error, Realm, Res, Value};

impl FromValueOutput for temporal_rs::Duration {
    type Output = Self;

    fn from_value_out(value: Value, realm: &mut Realm) -> Res<Self::Output> {
        match value {
            Value::Object(obj) => {
                if let Some(dur) = obj.downcast::<NativeObject<Duration>>() {
                    return Ok(dur.dur);
                }

                // Try to parse as a property bag
                let mut extract =
                    |name: &'static str| match obj.resolve_property(name, realm)?.map(|v| {
                        v.to_number(realm).and_then(|n| {
                            if n.is_infinite() || n.is_nan() || n.fract() != 0.0 {
                                Err(Error::range("Invalid value for Duration"))
                            } else {
                                Ok(n as i64)
                            }
                        })
                    }) {
                        Some(Ok(n)) => Ok(Some(n)),
                        Some(Err(e)) => Err(e),
                        None => Ok(None),
                    };

                let years = extract("years")?;
                let months = extract("months")?;
                let weeks = extract("weeks")?;
                let days = extract("days")?;
                let hours = extract("hours")?;
                let minutes = extract("minutes")?;
                let seconds = extract("seconds")?;
                let milliseconds = extract("milliseconds")?;
                let microseconds = extract("microseconds")?.map(i128::from);
                let nanoseconds = extract("nanoseconds")?.map(i128::from);

                if years.is_none()
                    && months.is_none()
                    && weeks.is_none()
                    && days.is_none()
                    && hours.is_none()
                    && minutes.is_none()
                    && seconds.is_none()
                    && milliseconds.is_none()
                    && microseconds.is_none()
                    && nanoseconds.is_none()
                {
                    return Err(Error::ty(
                        "At least one field must be provided for Duration",
                    ));
                }

                Self::new(
                    years.unwrap_or(0),
                    months.unwrap_or(0),
                    weeks.unwrap_or(0),
                    days.unwrap_or(0),
                    hours.unwrap_or(0),
                    minutes.unwrap_or(0),
                    seconds.unwrap_or(0),
                    milliseconds.unwrap_or(0),
                    microseconds.unwrap_or(0),
                    nanoseconds.unwrap_or(0),
                )
                .map_err(Error::from_temporal)
            }
            Value::String(s) => s.parse().map_err(Error::from_temporal),
            _ => Err(Error::ty(
                "Duration value must be a string or a Duration-like object",
            )),
        }
    }
}

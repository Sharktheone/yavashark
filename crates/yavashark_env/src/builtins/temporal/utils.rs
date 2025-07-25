use crate::builtins::temporal::plain_date::value_to_plain_date;
use crate::{Error, ObjectHandle, Realm, Res, Value};
use std::str::FromStr;
use temporal_rs::options::{
    ArithmeticOverflow, DifferenceSettings, Disambiguation, DisplayCalendar, DisplayOffset,
    DisplayTimeZone, OffsetDisambiguation, RelativeTo, RoundingIncrement, RoundingOptions,
    ToStringRoundingOptions, Unit,
};
use temporal_rs::parsers::Precision;
use temporal_rs::provider::TransitionDirection;
use temporal_rs::Calendar;

pub fn opt_relative_to_wrap(
    obj: Option<ObjectHandle>,
    realm: &mut Realm,
) -> Res<Option<RelativeTo>> {
    obj.map_or_else(|| Ok(None), |obj| relative_to_wrap(&obj, realm))
}

pub fn relative_to_wrap(obj: &ObjectHandle, realm: &mut Realm) -> Res<Option<RelativeTo>> {
    let rel = obj.get_property_opt(&"relativeTo".into())?.map(|v| v.value);

    rel.map_or_else(|| Ok(None), |rel| relative_to(rel, realm))
}

pub fn relative_to(rel: Value, realm: &mut Realm) -> Res<Option<RelativeTo>> {
    Ok(match rel {
        Value::Object(obj) => {
            let plain_date = value_to_plain_date(obj.into(), realm)?;

            Some(RelativeTo::PlainDate(plain_date))
        }
        Value::String(str) => Some(
            RelativeTo::try_from_str_with_provider(str.as_str(), &realm.env.tz_provider)
                .map_err(Error::from_temporal)?,
        ),

        _ => None,
    })
}

pub fn string_rounding_mode_opts(
    obj: Option<ObjectHandle>,
    realm: &mut Realm,
) -> Res<ToStringRoundingOptions> {
    let mut opts = ToStringRoundingOptions::default();

    if let Some(obj) = obj {
        let smallest = obj.get("smallestUnit", realm)?;

        opts.smallest_unit = if smallest.is_undefined() {
            None
        } else {
            let smallest = smallest.to_string(realm)?;

            Some(
                Unit::from_str(smallest.as_str())
                    .map_err(|_| Error::range("Invalid unit for Duration.toString"))?,
            )
        };

        let rm = obj.get("roundingMode", realm)?;

        opts.rounding_mode = if rm.is_undefined() {
            None
        } else {
            let rm = rm.to_string(realm)?;

            Some(
                temporal_rs::options::RoundingMode::from_str(rm.as_str())
                    .map_err(|_| Error::range("Invalid rounding mode for Duration.toString"))?,
            )
        };

        let digits = obj.get("fractionalSecondDigits", realm)?;

        let digits = if digits.is_undefined()
            | matches!(&digits, Value::String(s) if s.as_str() == "auto")
        {
            None
        } else {
            let digits = digits.to_number(realm)?;

            if digits.is_infinite() || digits.is_nan() {
                return Err(Error::range(
                    "Invalid fractionalSecondDigits for Duration.toString",
                ));
            }

            let digits = digits.floor();

            if !(0.0..=9.0).contains(&digits) {
                return Err(Error::range(
                    "fractionalSecondDigits must be between 0 and 9",
                ));
            }

            Some(digits as u8)
        };

        opts.precision = match (opts.smallest_unit, digits) {
            (Some(Unit::Minute), _) => Precision::Minute,
            (Some(Unit::Second), _) => Precision::Digit(0),
            (Some(Unit::Millisecond), _) => Precision::Digit(3),
            (Some(Unit::Microsecond), _) => Precision::Digit(6),
            (Some(Unit::Nanosecond), _) => Precision::Digit(9),
            (_, None) => Precision::Auto,
            (_, Some(d)) => Precision::Digit(d),
        };
    }

    Ok(opts)
}

pub fn rounding_options(
    unit: Value,
    realm: &mut Realm,
) -> Res<(RoundingOptions, Option<RelativeTo>)> {
    let mut opts = RoundingOptions::default();

    let mut rel = None;

    if let Value::String(s) = unit {
        let smallest = Unit::from_str(s.as_str()).map_err(|_| Error::range("Invalid unit"))?;

        opts.smallest_unit = Some(smallest);
    } else if let Value::Object(obj) = unit {
        let smallest = obj.get("smallestUnit", realm)?;

        opts.smallest_unit = if smallest.is_undefined() {
            None
        } else {
            let smallest = smallest.to_string(realm)?;

            Some(Unit::from_str(smallest.as_str()).map_err(|_| Error::range("Invalid unit"))?)
        };

        let largest = obj.get("largestUnit", realm)?;
        opts.largest_unit = if largest.is_undefined() {
            None
        } else {
            Some(
                Unit::from_str(largest.to_string(realm)?.as_str())
                    .map_err(|_| Error::range("Invalid unit"))?,
            )
        };

        let increment = obj.get("roundingIncrement", realm)?;

        opts.increment = if increment.is_undefined() {
            None
        } else {
            let increment = increment.to_number(realm)?;

            Some(RoundingIncrement::try_from(increment).map_err(Error::from_temporal)?)
        };

        let rounding_mode = obj.get("roundingMode", realm)?;

        opts.rounding_mode = if rounding_mode.is_undefined() {
            None
        } else {
            let rounding_mode = rounding_mode.to_string(realm)?;

            Some(
                temporal_rs::options::RoundingMode::from_str(rounding_mode.as_str())
                    .map_err(|_| Error::range("Invalid rounding mode"))?,
            )
        };

        let r = obj.get_property_opt(&"relativeTo".into())?.map(|v| v.value);

        rel = match r {
            Some(Value::Object(obj)) => {
                let plain_date = value_to_plain_date(obj.into(), realm)?;

                Some(RelativeTo::PlainDate(plain_date))
            }
            Some(Value::String(str)) => Some(
                RelativeTo::try_from_str_with_provider(str.as_str(), &realm.env.tz_provider)
                    .map_err(Error::from_temporal)?,
            ),

            _ => None,
        };
    } else {
        return Err(Error::ty("Invalid unit"));
    };

    Ok((opts, rel))
}

pub fn difference_settings(obj: ObjectHandle, realm: &mut Realm) -> Res<DifferenceSettings> {
    let mut opts = DifferenceSettings::default();

    let smallest = obj.get("smallestUnit", realm)?;

    opts.smallest_unit = if smallest.is_undefined() {
        None
    } else {
        let smallest = smallest.to_string(realm)?;

        Some(Unit::from_str(smallest.as_str()).map_err(|_| Error::range("Invalid unit"))?)
    };

    let largest = obj.get("largestUnit", realm)?;
    opts.largest_unit = if largest.is_undefined() {
        None
    } else {
        Some(
            Unit::from_str(largest.to_string(realm)?.as_str())
                .map_err(|_| Error::range("Invalid unit"))?,
        )
    };

    let rm = obj.get("roundingMode", realm)?;

    opts.rounding_mode = if rm.is_undefined() {
        None
    } else {
        let rm = rm.to_string(realm)?;

        Some(
            temporal_rs::options::RoundingMode::from_str(rm.as_str())
                .map_err(|_| Error::range("Invalid rounding mode for Duration.toString"))?,
        )
    };

    let increment = obj.get("roundingIncrement", realm)?;

    opts.increment = if increment.is_undefined() {
        None
    } else {
        let increment = increment.to_number(realm)?;

        Some(RoundingIncrement::try_from(increment).map_err(Error::from_temporal)?)
    };

    Ok(opts)
}

pub fn overflow_options(obj: &ObjectHandle, realm: &mut Realm) -> Res<Option<ArithmeticOverflow>> {
    let overflow = obj.get_opt("overflow", realm)?;

    let Some(overflow) = overflow else {
        return Ok(None);
    };

    if overflow.is_undefined() {
        return Ok(None);
    }

    let overflow = overflow.to_string(realm)?;

    let overflow = match overflow.as_str() {
        "constrain" => ArithmeticOverflow::Constrain,
        "reject" => ArithmeticOverflow::Reject,
        _ => return Err(Error::range("Invalid overflow option")),
    };

    Ok(Some(overflow))
}

pub fn overflow_options_opt(
    obj: Option<&ObjectHandle>,
    realm: &mut Realm,
) -> Res<Option<ArithmeticOverflow>> {
    Ok(match obj {
        Some(obj) => overflow_options(obj, realm)?,
        None => None,
    })
}

pub fn calendar_opt(cal: Option<&str>) -> Res<Calendar> {
    Ok(cal
        .map(temporal_rs::Calendar::from_str)
        .transpose()
        .map_err(crate::Error::from_temporal)?
        .unwrap_or_default())
}

pub fn display_calendar(cal: Option<&ObjectHandle>, realm: &mut Realm) -> Res<DisplayCalendar> {
    let Some(cal) = cal else {
        return Ok(DisplayCalendar::default());
    };

    let cal = cal.get_opt("calendarName", realm)?;

    let Some(cal) = cal else {
        return Ok(DisplayCalendar::default());
    };

    let cal = cal.to_string(realm)?;

    DisplayCalendar::from_str(&cal).map_err(Error::from_temporal)
}

pub fn display_offset(cal: Option<&ObjectHandle>, realm: &mut Realm) -> Res<DisplayOffset> {
    let Some(cal) = cal else {
        return Ok(DisplayOffset::default());
    };

    let display_offset = cal.get("displayOffset", realm)?;

    if display_offset.is_undefined() {
        return Ok(DisplayOffset::default());
    }

    let display_offset = display_offset.to_string(realm)?;

    let display_offset = DisplayOffset::from_str(&display_offset).map_err(Error::from_temporal)?;

    Ok(display_offset)
}

pub fn display_timezone(cal: Option<&ObjectHandle>, realm: &mut Realm) -> Res<DisplayTimeZone> {
    let Some(cal) = cal else {
        return Ok(DisplayTimeZone::default());
    };

    let display_timezone = cal.get("displayTimezone", realm)?;

    if display_timezone.is_undefined() {
        return Ok(DisplayTimeZone::default());
    }

    let display_timezone = display_timezone.to_string(realm)?;

    let display_timezone =
        DisplayTimeZone::from_str(&display_timezone).map_err(Error::from_temporal)?;

    Ok(display_timezone)
}

pub fn disambiguation_opt(
    obj: Option<&ObjectHandle>,
    realm: &mut Realm,
) -> Res<Option<Disambiguation>> {
    let Some(obj) = obj else {
        return Ok(None);
    };

    let disambiguation = obj.get("disambiguation", realm)?;

    if disambiguation.is_undefined() {
        return Ok(None);
    }

    let disambiguation = disambiguation.to_string(realm)?;

    Ok(Some(Disambiguation::from_str(&disambiguation).map_err(
        |_| Error::range("Invalid disambiguation option"),
    )?))
}

pub fn offset_disambiguation_opt(
    obj: Option<&ObjectHandle>,
    realm: &mut Realm,
) -> Res<Option<OffsetDisambiguation>> {
    let Some(obj) = obj else {
        return Ok(None);
    };

    let disambiguation = obj.get("offsetDisambiguation", realm)?;

    if disambiguation.is_undefined() {
        return Ok(None);
    }

    let disambiguation = disambiguation.to_string(realm)?;

    Ok(Some(
        OffsetDisambiguation::from_str(&disambiguation)
            .map_err(|_| Error::range("Invalid offsetDisambiguation option"))?,
    ))
}

pub fn transition_direction(obj: &Value, realm: &mut Realm) -> Res<TransitionDirection> {
    match obj {
        Value::Object(obj) => {
            let direction = obj.get("direction", realm)?;
            let direction = direction.to_string(realm)?;

            TransitionDirection::from_str(&direction)
                .map_err(|_| Error::range("Invalid transition direction"))
        }
        Value::String(s) => TransitionDirection::from_str(s.as_str())
            .map_err(|_| Error::range("Invalid transition direction")),
        _ => Err(Error::ty(
            "Expected an object or string for transition direction",
        )),
    }
}

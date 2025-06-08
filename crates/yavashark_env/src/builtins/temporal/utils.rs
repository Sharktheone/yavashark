use std::str::FromStr;
use temporal_rs::options::{RelativeTo, RoundingOptions, ToStringRoundingOptions, Unit};
use temporal_rs::{Calendar, PlainDate};
use temporal_rs::parsers::Precision;
use crate::{Error, ObjectHandle, Realm, Res, Value};

pub fn opt_relative_to_wrap(obj: Option<ObjectHandle>, realm: &mut Realm) -> Res<Option<RelativeTo>> {
    obj.map_or_else(|| Ok(None), |obj| relative_to_wrap(&obj, realm))
}

pub fn relative_to_wrap(obj: &ObjectHandle, realm: &mut Realm) -> Res<Option<RelativeTo>> {
    let rel = obj.get_property_opt(&"relativeTo".into())?.map(|v| v.value);

    rel.map_or_else(|| Ok(None), |rel| relative_to(rel, realm))
}

pub fn relative_to(rel: Value, realm: &mut Realm) -> Res<Option<RelativeTo>> {
    Ok(match rel {
        Value::Object(obj) => {
            let year = obj
                .get_opt("year", realm)?
                .ok_or(Error::ty("Invalid year for PlainDate"))?
                .to_number(realm)
                .and_then(|n| {
                    if n.fract() == 0.0 {
                        Ok(n as _)
                    } else {
                        Err(Error::range("Invalid year for PlainDate"))
                    }
                })?;

            let month = obj
                .get_opt("month", realm)?
                .ok_or(Error::ty("Invalid month for PlainDate"))?
                .to_number(realm)
                .and_then(|n| {
                    if n.fract() == 0.0 {
                        Ok(n as _)
                    } else {
                        Err(Error::range("Invalid year for PlainDate"))
                    }
                })?;

            let day = obj
                .get_opt("day", realm)?
                .ok_or(Error::ty("Invalid day for PlainDate"))?
                .to_number(realm)
                .and_then(|n| {
                    if n.fract() == 0.0 {
                        Ok(n as _)
                    } else {
                        Err(Error::range("Invalid year for PlainDate"))
                    }
                })?;

            let pd = PlainDate::new(year, month, day, Calendar::default())
                .map_err(Error::from_temporal)?;

            Some(RelativeTo::PlainDate(pd))
        }
        Value::String(str) => Some(
            RelativeTo::try_from_str_with_provider(str.as_str(), &realm.env.tz_provider)
                .map_err(Error::from_temporal)?,
        ),

        _ => None,
    })
}

pub fn string_rounding_mode_opts(obj: Option<ObjectHandle>, realm: &mut Realm) -> Res<ToStringRoundingOptions> {
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

pub fn rounding_options(unit: Value, realm: &mut Realm) -> Res<(RoundingOptions, Option<RelativeTo>)> {
    let mut opts = RoundingOptions::default();

    let mut rel = None;

    if let Value::String(s) = unit {
        let smallest = Unit::from_str(s.as_str())
            .map_err(|_| Error::range("Invalid unit"))?;

        opts.smallest_unit = Some(smallest);
    } else if let Value::Object(obj) = unit {
        let smallest = obj.get("smallestUnit", realm)?;

        opts.smallest_unit = if smallest.is_undefined() {
            None
        } else {
            let smallest = smallest.to_string(realm)?;

            Some(
                Unit::from_str(smallest.as_str())
                    .map_err(|_| Error::range("Invalid unit"))?,
            )
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

        let r = obj.get_property_opt(&"relativeTo".into())?.map(|v| v.value);

        rel = match r {
            Some(Value::Object(obj)) => {
                let year = obj.get("year", realm)?.to_number(realm).and_then(|n| {
                    if n.fract() == 0.0 {
                        Ok(n as _)
                    } else {
                        Err(Error::range("Invalid year for PlainDate"))
                    }
                })?;

                let month = obj.get("month", realm)?.to_number(realm).and_then(|n| {
                    if n.fract() == 0.0 {
                        Ok(n as _)
                    } else {
                        Err(Error::range("Invalid year for PlainDate"))
                    }
                })?;

                let day = obj.get("day", realm)?.to_number(realm).and_then(|n| {
                    if n.fract() == 0.0 {
                        Ok(n as _)
                    } else {
                        Err(Error::range("Invalid year for PlainDate"))
                    }
                })?;

                let pd = PlainDate::new(year, month, day, Calendar::default())
                    .map_err(Error::from_temporal)?;

                Some(RelativeTo::PlainDate(pd))
            }
            Some(Value::String(str)) => Some(
                RelativeTo::try_from_str_with_provider(str.as_str(), &realm.env.tz_provider)
                    .map_err(Error::from_temporal)?
            ),

            _ => None,
        };
    } else {
        return Err(Error::ty("Invalid unit"));
    };
    
    Ok((opts,rel))
}
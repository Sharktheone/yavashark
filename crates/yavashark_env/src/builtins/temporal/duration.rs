use crate::builtins::temporal::utils::{
    opt_relative_to_wrap, rounding_options, string_rounding_mode_opts,
};
use crate::conversion::{downcast_obj, NonFract};
use crate::print::{fmt_properties_to, PrettyObjectOverride};
use crate::{Error, MutObject, ObjectHandle, Realm, RefOrOwned, Res, Value};
use std::cell::RefCell;
use std::str::FromStr;
use temporal_rs::options::{ToStringRoundingOptions, Unit};
use temporal_rs::provider::COMPILED_TZ_PROVIDER;
use yavashark_macro::{object, props};
use yavashark_value::{Obj, Object};

#[object]
#[derive(Debug)]
pub struct Duration {
    pub dur: temporal_rs::Duration,
}

impl Duration {
    #[allow(unused)]
    fn new(realm: &Realm) -> Self {
        Self::with_duration(realm, temporal_rs::Duration::default())
    }

    pub fn with_duration(realm: &Realm, duration: temporal_rs::Duration) -> Self {
        Self {
            inner: RefCell::new(MutableDuration {
                object: MutObject::with_proto(realm.intrinsics.temporal_duration.clone().into()),
            }),
            dur: duration,
        }
    }

    pub fn from_value_ref(info: Value, realm: &mut Realm) -> Res<RefOrOwned<Self>> {
        if let Ok(this) = downcast_obj::<Self>(info.copy()) {
            return Ok(RefOrOwned::Ref(this));
        }

        if let Value::Object(obj) = info {
            let mut extract =
                |name: &'static str| match obj.resolve_property(&name.into(), realm)?.map(|v| {
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
            let microseconds = extract("microseconds")?;
            let nanoseconds = extract("nanoseconds")?;

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

            return Ok(RefOrOwned::Owned(Self::constructor(
                years,
                months,
                weeks,
                days,
                hours,
                minutes,
                seconds,
                milliseconds,
                microseconds.map(i128::from),
                nanoseconds.map(i128::from),
                realm,
            )?));
        } else if let Value::String(s) = info {
            return Ok(RefOrOwned::Owned(
                temporal_rs::Duration::from_str(s.as_str())
                    .map_err(Error::from_temporal)
                    .map(|dur| Self::with_duration(realm, dur))?,
            ));
        }

        Err(Error::ty("Invalid value for Duration"))
    }

    fn from_value(info: Value, realm: &mut Realm) -> Res<Self> {
        Ok(match Self::from_value_ref(info, realm)? {
            RefOrOwned::Ref(r) => {
                return Ok(Self::with_duration(realm, r.dur));
            }
            RefOrOwned::Owned(o) => o,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn constructor(
        years: Option<i64>,
        months: Option<i64>,
        weeks: Option<i64>,
        days: Option<i64>,
        hours: Option<i64>,
        minutes: Option<i64>,
        seconds: Option<i64>,
        milliseconds: Option<i64>,
        microseconds: Option<i128>,
        nanoseconds: Option<i128>,
        realm: &Realm,
    ) -> Res<Self> {
        let years = years.unwrap_or(0);
        let months = months.unwrap_or(0);
        let weeks = weeks.unwrap_or(0);
        let days = days.unwrap_or(0);
        let hours = hours.unwrap_or(0);
        let minutes = minutes.unwrap_or(0);
        let seconds = seconds.unwrap_or(0);
        let milliseconds = milliseconds.unwrap_or(0);
        let microseconds = microseconds.unwrap_or(0);
        let nanoseconds = nanoseconds.unwrap_or(0);

        temporal_rs::Duration::new(
            years,
            months,
            weeks,
            days,
            hours,
            minutes,
            seconds,
            milliseconds,
            microseconds,
            nanoseconds,
        )
        .map_err(Error::from_temporal)
        .map(|dur| Self::with_duration(realm, dur))
    }
}

#[props]
impl Duration {
    #[constructor]
    #[allow(clippy::too_many_arguments)]
    pub fn construct(
        years: Option<NonFract<i64>>,
        months: Option<NonFract<i64>>,
        weeks: Option<NonFract<i64>>,
        days: Option<NonFract<i64>>,
        hours: Option<NonFract<i64>>,
        minutes: Option<NonFract<i64>>,
        seconds: Option<NonFract<i64>>,
        milliseconds: Option<NonFract<i64>>,
        microseconds: Option<NonFract<i128>>,
        nanoseconds: Option<NonFract<i128>>,
        #[realm] realm: &Realm,
    ) -> Res<ObjectHandle> {
        let years = years.map(|n| n.0);
        let months = months.map(|n| n.0);
        let weeks = weeks.map(|n| n.0);
        let days = days.map(|n| n.0);
        let hours = hours.map(|n| n.0);
        let minutes = minutes.map(|n| n.0);
        let seconds = seconds.map(|n| n.0);
        let milliseconds = milliseconds.map(|n| n.0);
        let microseconds = microseconds.map(|n| n.0);
        let nanoseconds = nanoseconds.map(|n| n.0);

        Ok(Self::constructor(
            years,
            months,
            weeks,
            days,
            hours,
            minutes,
            seconds,
            milliseconds,
            microseconds,
            nanoseconds,
            realm,
        )?
        .into_object())
    }

    fn from(info: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(Self::from_value(info, realm)?.into_object())
    }

    fn compare(
        left: Value,
        right: Value,
        obj: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> Res<i8> {
        let left = Self::from_value_ref(left, realm)?;
        let right = Self::from_value_ref(right, realm)?;

        let rel = opt_relative_to_wrap(obj, realm)?;

        Ok(left
            .dur
            .compare_with_provider(&right.dur, rel, &*COMPILED_TZ_PROVIDER)
            .map_err(Error::from_temporal)? as i8)
    }

    fn abs(&self, #[realm] realm: &Realm) -> Res<ObjectHandle> {
        let res = self.dur.abs();

        Ok(Self::with_duration(realm, res).into_object())
    }

    fn add(&self, other: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let other = Self::from_value_ref(other, realm)?;

        let dur = self.dur.add(&other.dur).map_err(Error::from_temporal)?;

        Ok(Self::with_duration(realm, dur).into_object())
    }

    fn negated(&self, #[realm] realm: &Realm) -> ObjectHandle {
        let neg = self.dur.negated();

        Self::with_duration(realm, neg).into_object()
    }

    fn round(&self, unit: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        if unit.is_undefined() {
            return Err(Error::ty("Invalid unit for Duration.round"));
        }

        let (opts, rel) = rounding_options(unit, realm)?;

        let dur = self
            .dur
            .round_with_provider(opts, rel, &*COMPILED_TZ_PROVIDER)
            .map_err(Error::from_temporal)?;

        Ok(Self::with_duration(realm, dur).into_object())
    }

    fn subtract(&self, other: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let other = Self::from_value(other, realm)?;

        let dur = self
            .dur
            .subtract(&other.dur)
            .map_err(Error::from_temporal)?;

        Ok(Self::with_duration(realm, dur).into_object())
    }

    #[prop("toJSON")]
    fn to_json(&self) -> String {
        let dur = self.dur;

        dur.to_string()
    }

    #[prop("toString")]
    fn to_js_string(&self, obj: Option<ObjectHandle>, #[realm] realm: &mut Realm) -> Res<String> {
        let opts = string_rounding_mode_opts(obj, realm)?;

        let dur = self.dur;

        dur.as_temporal_string(opts).map_err(Error::from_temporal)
    }

    #[prop("toLocaleString")]
    fn to_locale_string(&self) -> String {
        let dur = self.dur;

        dur.to_string()
    }

    fn total(&self, obj: Value, #[realm] realm: &mut Realm) -> Res<f64> {
        let (unit, obj) = if let Value::String(unit) = obj {
            (unit, None)
        } else if let Value::Object(obj) = obj {
            let unit = obj.get("unit", realm)?;

            (unit.to_string(realm)?, Some(obj))
        } else {
            return Err(Error::ty("Invalid unit for Duration.total"));
        };

        let unit = Unit::from_str(unit.as_str())
            .map_err(|_| Error::range("Invalid unit for Duration.total"))?;

        let rel = opt_relative_to_wrap(obj, realm)?;

        let dur = self
            .dur
            .total_with_provider(unit, rel, &*COMPILED_TZ_PROVIDER)
            .map_err(Error::from_temporal)?;

        Ok(dur.as_inner())
    }

    #[nonstatic]
    #[prop("valueOf")]
    const fn value_of() -> Res {
        Err(Error::ty("Invalid value for Duration"))
    }

    #[get("blank")]
    fn blank(&self) -> bool {
        self.dur.is_zero()
    }

    #[get("days")]
    const fn days(&self) -> i64 {
        self.dur.days()
    }

    #[get("hours")]
    const fn hours(&self) -> i64 {
        self.dur.hours()
    }

    #[get("microseconds")]
    const fn microseconds(&self) -> i128 {
        self.dur.microseconds()
    }

    #[get("milliseconds")]
    const fn milliseconds(&self) -> i64 {
        self.dur.milliseconds()
    }

    #[get("minutes")]
    const fn minutes(&self) -> i64 {
        self.dur.minutes()
    }

    #[get("months")]
    const fn months(&self) -> i64 {
        self.dur.months()
    }

    #[get("nanoseconds")]
    const fn nanoseconds(&self) -> i128 {
        self.dur.nanoseconds()
    }

    #[get("seconds")]
    const fn seconds(&self) -> i64 {
        self.dur.seconds()
    }

    #[get("sign")]
    fn sign(&self) -> i8 {
        self.dur.sign() as i8
    }

    #[get("weeks")]
    const fn weeks(&self) -> i64 {
        self.dur.weeks()
    }

    #[get("years")]
    const fn years(&self) -> i64 {
        self.dur.years()
    }
}

pub fn value_to_duration(value: Value, realm: &mut Realm) -> Res<temporal_rs::Duration> {
    Ok(Duration::from_value_ref(value, realm)?.dur)
}

impl PrettyObjectOverride for Duration {
    fn pretty_inline(&self, obj: &Object<Realm>, not: &mut Vec<usize>) -> Option<String> {
        let mut s = self
            .dur
            .as_temporal_string(ToStringRoundingOptions::default())
            .ok()?;

        fmt_properties_to(obj, &mut s, not);

        Some(s)
    }
}

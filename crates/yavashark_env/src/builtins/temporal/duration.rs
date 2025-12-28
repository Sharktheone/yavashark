use crate::builtins::temporal::utils::{
    opt_relative_to_wrap, rounding_options, string_rounding_mode_opts,
};
use crate::conversion::{downcast_obj, NonFract};
use crate::native_obj::NativeObject;
use crate::print::{fmt_properties_to, PrettyObjectOverride};
use crate::value::Object;
use crate::{Error, ObjectHandle, Realm, RefOrOwned, Res, Value};
use std::str::FromStr;
use temporal_rs::options::{ToStringRoundingOptions, Unit};
use yavashark_macro::props;

#[derive(Debug)]
pub struct Duration {
    pub dur: temporal_rs::Duration,
}

impl Duration {
    #[allow(unused)]
    fn new(realm: &mut Realm) -> Res<NativeObject<Self>> {
        Self::with_duration(realm, temporal_rs::Duration::default())
    }

    pub fn with_duration(
        realm: &mut Realm,
        duration: temporal_rs::Duration,
    ) -> Res<NativeObject<Self>> {
        NativeObject::new(Self { dur: duration }, realm)
    }

    pub fn from_value_ref(info: Value, realm: &mut Realm) -> Res<RefOrOwned<NativeObject<Self>>> {
        if let Ok(this) = downcast_obj::<NativeObject<Self>>(info.copy()) {
            return Ok(RefOrOwned::Ref(this));
        }

        if let Value::Object(obj) = info {
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
                    .and_then(|dur| Self::with_duration(realm, dur))?,
            ));
        }

        Err(Error::ty("Invalid value for Duration"))
    }

    fn from_value(info: Value, realm: &mut Realm) -> Res<NativeObject<Self>> {
        Ok(match Self::from_value_ref(info, realm)? {
            RefOrOwned::Ref(r) => {
                return Ok(Self::with_duration(realm, r.dur)?);
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
        realm: &mut Realm,
    ) -> Res<NativeObject<Self>> {
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
        .and_then(|dur| Self::with_duration(realm, dur))
    }
}

#[props(intrinsic_name = temporal_duration, to_string_tag = "Temporal.Duration")]
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
        #[realm] realm: &mut Realm,
    ) -> Res<NativeObject<Self>> {
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

        Self::constructor(
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
        )
    }

    fn from(info: Value, #[realm] realm: &mut Realm) -> Res<NativeObject<Self>> {
        Self::from_value(info, realm)
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
            .compare(&right.dur, rel)
            .map_err(Error::from_temporal)? as i8)
    }

    fn abs(&self, #[realm] realm: &mut Realm) -> Res<NativeObject<Self>> {
        let res = self.dur.abs();

        Self::with_duration(realm, res)
    }

    fn add(&self, other: Value, #[realm] realm: &mut Realm) -> Res<NativeObject<Self>> {
        let other = Self::from_value_ref(other, realm)?;

        let dur = self.dur.add(&other.dur).map_err(Error::from_temporal)?;

        Self::with_duration(realm, dur)
    }

    fn negated(&self, #[realm] realm: &mut Realm) -> Res<NativeObject<Self>> {
        let neg = self.dur.negated();

        Self::with_duration(realm, neg)
    }

    fn round(&self, unit: Value, #[realm] realm: &mut Realm) -> Res<NativeObject<Self>> {
        if unit.is_undefined() {
            return Err(Error::ty("Invalid unit for Duration.round"));
        }

        let (opts, rel) = rounding_options(unit, realm)?;

        let dur = self.dur.round(opts, rel).map_err(Error::from_temporal)?;

        Self::with_duration(realm, dur)
    }

    fn subtract(&self, other: Value, #[realm] realm: &mut Realm) -> Res<NativeObject<Self>> {
        let other = Self::from_value(other, realm)?;

        let dur = self
            .dur
            .subtract(&other.dur)
            .map_err(Error::from_temporal)?;

        Self::with_duration(realm, dur)
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

        let dur = self.dur.total(unit, rel).map_err(Error::from_temporal)?;

        Ok(dur.as_inner())
    }

    fn with(&self, obj: ObjectHandle, #[realm] realm: &mut Realm) -> Res<NativeObject<Self>> {
        // Macro to extract an optional field value
        macro_rules! extract_opt {
            ($name:literal, $ty:ty) => {{
                let val = obj.get($name, realm)?;
                if val.is_undefined() {
                    None
                } else {
                    let n = val.to_number(realm)?;
                    if n.is_infinite() || n.is_nan() || n.fract() != 0.0 {
                        return Err(Error::range("Invalid value for Duration field"));
                    }
                    Some(n as $ty)
                }
            }};
        }

        // Extract fields from the options object in alphabetical order per spec
        let days_opt: Option<i64> = extract_opt!("days", i64);
        let hours_opt: Option<i64> = extract_opt!("hours", i64);
        let microseconds_opt: Option<i128> = extract_opt!("microseconds", i128);
        let milliseconds_opt: Option<i64> = extract_opt!("milliseconds", i64);
        let minutes_opt: Option<i64> = extract_opt!("minutes", i64);
        let months_opt: Option<i64> = extract_opt!("months", i64);
        let nanoseconds_opt: Option<i128> = extract_opt!("nanoseconds", i128);
        let seconds_opt: Option<i64> = extract_opt!("seconds", i64);
        let weeks_opt: Option<i64> = extract_opt!("weeks", i64);
        let years_opt: Option<i64> = extract_opt!("years", i64);

        // Check if at least one field was provided
        if years_opt.is_none()
            && months_opt.is_none()
            && weeks_opt.is_none()
            && days_opt.is_none()
            && hours_opt.is_none()
            && minutes_opt.is_none()
            && seconds_opt.is_none()
            && milliseconds_opt.is_none()
            && microseconds_opt.is_none()
            && nanoseconds_opt.is_none()
        {
            return Err(Error::ty(
                "At least one field must be provided for Duration.with",
            ));
        }

        // Fall back to current duration values for fields not provided
        let years = years_opt.unwrap_or(self.dur.years());
        let months = months_opt.unwrap_or(self.dur.months());
        let weeks = weeks_opt.unwrap_or(self.dur.weeks());
        let days = days_opt.unwrap_or(self.dur.days());
        let hours = hours_opt.unwrap_or(self.dur.hours());
        let minutes = minutes_opt.unwrap_or(self.dur.minutes());
        let seconds = seconds_opt.unwrap_or(self.dur.seconds());
        let milliseconds = milliseconds_opt.unwrap_or(self.dur.milliseconds());
        let microseconds = microseconds_opt.unwrap_or(self.dur.microseconds());
        let nanoseconds = nanoseconds_opt.unwrap_or(self.dur.nanoseconds());

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
        .and_then(|dur| Self::with_duration(realm, dur))
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
    fn pretty_inline(
        &self,
        obj: &Object,
        not: &mut Vec<usize>,
        realm: &mut Realm,
    ) -> Option<String> {
        let mut s = self
            .dur
            .as_temporal_string(ToStringRoundingOptions::default())
            .ok()?;

        fmt_properties_to(obj, &mut s, not, realm);

        Some(s)
    }
}

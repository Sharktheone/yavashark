use crate::conversion::FromValueOutput;
use crate::{Error, MutObject, ObjectHandle, Realm, RefOrOwned, Res, Value};
use std::cell::{Cell, RefCell};
use chrono::TimeDelta;
use yavashark_macro::{object, props};
use yavashark_value::Obj;

#[object]
#[derive(Debug)]
pub struct Duration {
    dur: Cell<std::time::Duration>,
    negative: Cell<bool>,
}

impl Duration {
    #[allow(unused)]
    fn new(realm: &Realm) -> Self {
        Self::with_duration(realm, std::time::Duration::ZERO)
    }

    fn with_duration(realm: &Realm, duration: std::time::Duration) -> Self {
        Self::with_sign(realm, duration, false)
    }

    fn with_sign(realm: &Realm, duration: std::time::Duration, negative: bool) -> Self {
        Self {
            inner: RefCell::new(MutableDuration {
                object: MutObject::with_proto(realm.intrinsics.temporal_duration.clone().into()),
            }),
            dur: Cell::new(duration),
            negative: Cell::new(negative),
        }
    }
    
    pub fn from_duration(delta: TimeDelta, realm: &Realm) -> Res<Self> {
        Ok(Self::with_sign(realm, delta.to_std()?, false))
    }

    fn from_secs(realm: &Realm, secs: i64) -> Self {
        Self::with_sign(
            realm,
            std::time::Duration::from_secs(secs.unsigned_abs()),
            secs.is_negative(),
        )
    }

    pub fn from_value_ref(info: Value, realm: &mut Realm) -> Res<RefOrOwned<Self>> {
        if let Ok(this) = <&Self>::from_value_out(info.copy()) {
            return Ok(RefOrOwned::Ref(this));
        }

        if let Value::Object(obj) = info {
            let mut extract = || {
                Result::<Option<i64>, Error>::Ok(
                    obj.resolve_property(&"years".into(), realm)?
                        .map(|v| v.to_number(realm).unwrap_or(0.0) as i64),
                )
            };

            let years = extract()?;
            let months = extract()?;
            let weeks = extract()?;
            let days = extract()?;
            let hours = extract()?;
            let minutes = extract()?;
            let seconds = extract()?;
            let milliseconds = extract()?;
            let microseconds = extract()?;

            return Ok(RefOrOwned::Owned(Self::constructor(
                years,
                months,
                weeks,
                days,
                hours,
                minutes,
                seconds,
                milliseconds,
                microseconds,
                realm,
            )?));
        }

        Err(Error::ty("Invalid value for Duration"))
    }

    fn from_value(info: Value, realm: &mut Realm) -> Res<Self> {
        Ok(match Self::from_value_ref(info, realm)? {
            RefOrOwned::Ref(r) => {
                return Ok(Self::with_duration(realm, r.dur.get()));
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
        microseconds: Option<i64>,
        realm: &Realm,
    ) -> Res<Self> {
        let mut dur = std::time::Duration::ZERO;

        let mut negative = false;

        let mut check_sign = |val: i64| -> Res<u64> {
            if negative && val.is_positive() {
                return Err(Error::range("Invalid value for Duration"));
            }

            if val.is_negative() {
                negative = true;
            }

            Ok(val.unsigned_abs())
        };

        if let Some(years) = years {
            dur += std::time::Duration::from_secs(check_sign(years)? * 365 * 24 * 60 * 60);
        }

        if let Some(months) = months {
            dur += std::time::Duration::from_secs(check_sign(months)? * 30 * 24 * 60 * 60);
        }

        if let Some(weeks) = weeks {
            dur += std::time::Duration::from_secs(check_sign(weeks)? * 7 * 24 * 60 * 60);
        }

        if let Some(days) = days {
            dur += std::time::Duration::from_secs(check_sign(days)? * 24 * 60 * 60);
        }

        if let Some(hours) = hours {
            dur += std::time::Duration::from_secs(check_sign(hours)? * 60 * 60);
        }

        if let Some(minutes) = minutes {
            dur += std::time::Duration::from_secs(check_sign(minutes)? * 60);
        }

        if let Some(seconds) = seconds {
            dur += std::time::Duration::from_secs(check_sign(seconds)?);
        }

        if let Some(milliseconds) = milliseconds {
            dur += std::time::Duration::from_millis(check_sign(milliseconds)?);
        }

        if let Some(microseconds) = microseconds {
            dur += std::time::Duration::from_micros(check_sign(microseconds)?);
        }

        Ok(Self::with_sign(realm, dur, negative))
    }

    pub fn to_duration(&self) -> std::time::Duration {
        self.dur.get()
    }

    pub fn is_negative(&self) -> bool {
        self.negative.get()
    }
}

#[props]
impl Duration {
    #[constructor]
    #[allow(clippy::too_many_arguments)]
    pub fn construct(
        years: Option<i64>,
        months: Option<i64>,
        weeks: Option<i64>,
        days: Option<i64>,
        hours: Option<i64>,
        minutes: Option<i64>,
        seconds: Option<i64>,
        milliseconds: Option<i64>,
        microseconds: Option<i64>,
        #[realm] realm: &Realm,
    ) -> Res<ObjectHandle> {
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
            realm,
        )?
        .into_object())
    }

    fn from(info: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(Self::from_value(info, realm)?.into_object())
    }

    fn compare(left: Value, right: Value, #[realm] realm: &mut Realm) -> Res<i8> {
        let left = Self::from_value_ref(left, realm)?;
        let right = Self::from_value_ref(right, realm)?;

        if left.negative.get() && !right.negative.get() {
            return Ok(-1);
        }

        if !left.negative.get() && right.negative.get() {
            return Ok(1);
        }

        let neg = left.negative.get();

        let left = left.dur.get();
        let right = right.dur.get();



        let cmp = left.cmp(&right) as i8;



        Ok(if neg { -cmp } else { cmp })
    }

    fn abs(&self) {
        self.negative.set(false);
    }

    fn add(&self, other: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let other = Self::from_value_ref(other, realm)?;

        let other_dur = other.dur.get();
        let other_neg = other.negative.get();

        let self_dur = self.dur.get();
        let self_neg = self.negative.get();

        let mut neg = false;

        let dur = if self_neg == other_neg {
            neg = self_neg;
            self_dur
                .checked_add(other_dur)
                .ok_or_else(|| Error::range("Duration overflow"))?
        } else if self_neg {
            if self_dur > other_dur {
                neg = true;
                self_dur - other_dur
            } else {
                other_dur - self_dur
            }
        } else if self_dur > other_dur {
            other_dur - self_dur
        } else {
            neg = true;
            self_dur - other_dur
        };

        Ok(Self::with_sign(realm, dur, neg).into_object())
    }

    fn negated(&self, #[realm] realm: &Realm) -> ObjectHandle {
        Self::with_sign(realm, self.dur.get(), !self.negative.get()).into_object()
    }

    fn round(&self, unit: Value, #[realm] realm: &Realm) -> Res<ObjectHandle> {
        if let Value::String(unit) = unit {
            let dur = self.dur.get();

            let mut dur = match unit.as_str() {
                "years" => dur.as_secs() as i64 / (365 * 24 * 60 * 60),
                "months" => dur.as_secs() as i64 / (30 * 24 * 60 * 60),
                "days" => dur.as_secs() as i64 / (24 * 60 * 60),
                "hours" => dur.as_secs() as i64 / (60 * 60),
                "minutes" => dur.as_secs() as i64 / 60,
                "seconds" => dur.as_secs() as i64,
                "milliseconds" => dur.as_millis() as i64,
                "microseconds" => dur.as_micros() as i64,
                "nanoseconds" => dur.as_nanos() as i64,
                _ => return Err(Error::range("Invalid unit for Duration.round")),
            };

            if self.negative.get() {
                dur = -dur;
            }

            return Ok(Self::from_secs(realm, dur).into_object());
        }

        Err(Error::ty("Invalid unit for Duration.round")) //TODO: handle with options object
    }

    fn subtract(&self, other: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let other = Self::from_value(other, realm)?;

        other.negative.set(!other.negative.get());

        self.add(other.into_value(), realm)
    }

    #[prop("toJSON")]
    fn to_json(&self) -> String {
        let dur = self.dur.get();

        let mut dur = dur.as_secs() as i64;

        if self.negative.get() {
            dur = -dur;
        }

        dur.to_string()
    }

    #[prop("toString")]
    fn to_js_string(&self) -> String {
        let dur = self.dur.get();

        let mut dur = dur.as_secs() as i64;

        if self.negative.get() {
            dur = -dur;
        }

        dur.to_string()
    }

    fn total(&self, unit: &str) -> Res<f64> {
        let dur = self.dur.get();

        let mut dur = match unit {
            "years" => dur.as_secs() as f64 / (365.0 * 24.0 * 60.0 * 60.0),
            "months" => dur.as_secs() as f64 / (30.0 * 24.0 * 60.0 * 60.0),
            "days" => dur.as_secs() as f64 / (24.0 * 60.0 * 60.0),
            "hours" => dur.as_secs() as f64 / (60.0 * 60.0),
            "minutes" => dur.as_secs() as f64 / 60.0,
            "seconds" => dur.as_secs() as f64,
            "milliseconds" => dur.as_millis() as f64,
            "microseconds" => dur.as_micros() as f64,
            "nanoseconds" => dur.as_nanos() as f64,
            _ => return Err(Error::range("Invalid unit for Duration.total")),
        };

        if self.negative.get() {
            dur = -dur;
        }

        Ok(dur)
    }

    #[nonstatic]
    fn value_of() -> Res {
        Err(Error::ty("Invalid value for Duration"))
    }

    #[get("blank")]
    fn blank(&self) -> bool {
        self.dur.get() == std::time::Duration::ZERO
    }

    #[get("days")]
    fn days(&self) -> i64 {
        self.dur.get().as_secs() as i64 / (24 * 60 * 60) % 365 % 30 % 7
    }

    #[get("hours")]
    fn hours(&self) -> i64 {
        self.dur.get().as_secs() as i64 / (60 * 60) % 24
    }

    #[get("microseconds")]
    fn microseconds(&self) -> i64 {
        self.dur.get().as_micros() as i64 % 1000
    }

    #[get("milliseconds")]
    fn milliseconds(&self) -> i64 {
        self.dur.get().as_millis() as i64 % 1000
    }

    #[get("minutes")]
    fn minutes(&self) -> i64 {
        self.dur.get().as_secs() as i64 / 60 % 60
    }

    #[get("months")]
    fn months(&self) -> i64 {
        self.dur.get().as_secs() as i64 / (30 * 24 * 60 * 60) % 12
    }

    #[get("nanoseconds")]
    fn nanoseconds(&self) -> i64 {
        self.dur.get().as_nanos() as i64 % 1000
    }

    #[get("seconds")]
    fn seconds(&self) -> i64 {
        self.dur.get().as_secs() as i64 % 60
    }

    #[get("sign")]
    fn sign(&self) -> i8 {
        if self.negative.get() {
            -1
        } else {
            1
        }
    }

    #[get("weeks")]
    fn weeks(&self) -> i64 {
        self.dur.get().as_secs() as i64 / (7 * 24 * 60 * 60) % 7
    }

    #[get("years")]
    fn years(&self) -> i64 {
        self.dur.get().as_secs() as i64 / (365 * 24 * 60 * 60)
    }
}

use crate::builtins::temporal::duration::{value_to_duration, Duration};
use crate::builtins::temporal::now::Now;
use crate::builtins::temporal::plain_date::value_to_plain_date;
use crate::builtins::temporal::plain_date_time::PlainDateTime;
use crate::builtins::temporal::utils::{difference_settings, string_rounding_mode_opts};
use crate::{Error, MutObject, ObjectHandle, Realm, Res, Value};
use std::cell::RefCell;
use std::str::FromStr;
use temporal_rs::options::ToStringRoundingOptions;
use temporal_rs::TimeZone;
use yavashark_macro::{object, props};
use yavashark_value::{Obj, Object};
use crate::print::{fmt_properties_to, PrettyObjectOverride};

#[object]
#[derive(Debug)]
pub struct PlainTime {
    time: temporal_rs::PlainTime,
}

impl PlainTime {
    pub fn new(time: temporal_rs::PlainTime, realm: &Realm) -> Self {
        Self {
            inner: RefCell::new(MutablePlainTime {
                object: MutObject::with_proto(realm.intrinsics.temporal_plain_time.clone().into()),
            }),
            time,
        }
    }

    pub fn now(realm: &Realm, tz: Option<TimeZone>) -> Res<temporal_rs::PlainTime> {
        Now::get_now()?
            .plain_time_with_provider(tz, &realm.env.tz_provider)
            .map_err(Error::from_temporal)
    }

    pub fn now_obj(realm: &Realm, tz: Option<TimeZone>) -> Res<ObjectHandle> {
        let time = Self::now(realm, tz)?;

        Ok(Self::new(time, realm).into_object())
    }
}

#[props]
impl PlainTime {
    #[constructor]
    pub fn construct(
        #[realm] realm: &Realm,
        hour: u8,
        minute: u8,
        second: u8,
        millisecond: u16,
        microsecond: u16,
        nanosecond: u16,
    ) -> Res<ObjectHandle> {
        let time =
            temporal_rs::PlainTime::new(hour, minute, second, millisecond, microsecond, nanosecond)
                .map_err(Error::from_temporal)?;

        Ok(Self::new(time, realm).into_object())
    }

    pub fn compare(left: Value, right: Value, #[realm] realm: &mut Realm) -> Res<i8> {
        let left_time = value_to_plain_time(left, realm)?;
        let right_time = value_to_plain_time(right, realm)?;

        let result = left_time.cmp(&right_time);

        Ok(result as i8)
    }

    pub fn from(value: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let time = value_to_plain_time(value, realm)?;

        Ok(Self::new(time, realm).into_object())
    }

    pub fn add(&self, dur: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let dur = value_to_duration(dur, realm)?;

        let new_time = self.time.add(&dur).map_err(Error::from_temporal)?;

        Ok(Self::new(new_time, realm).into_object())
    }

    pub fn equals(&self, other: Value, #[realm] realm: &mut Realm) -> Res<bool> {
        let other_time = value_to_plain_time(other, realm)?;

        Ok(self.time == other_time)
    }

    pub fn round(&self, _opts: Option<Value>, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        //TODO
        Ok(Self::new(self.time, realm).into_object())
    }

    pub fn since(
        &self,
        other: Value,
        opts: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let other_time = value_to_plain_time(other, realm)?;

        let opts = opts
            .map(|opts| difference_settings(opts, realm))
            .transpose()?
            .unwrap_or_default();

        let duration = self
            .time
            .since(&other_time, opts)
            .map_err(Error::from_temporal)?;

        Ok(Duration::with_duration(realm, duration).into_object())
    }

    pub fn subtract(&self, dur: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let dur = value_to_duration(dur, realm)?;

        let new_time = self.time.subtract(&dur).map_err(Error::from_temporal)?;

        Ok(Self::new(new_time, realm).into_object())
    }

    #[prop("toJSON")]
    pub fn to_json(&self) -> Res<String> {
        self.time
            .to_ixdtf_string(ToStringRoundingOptions::default())
            .map_err(Error::from_temporal)
    }

    #[prop("toString")]
    pub fn to_string(&self, opts: Option<ObjectHandle>, #[realm] realm: &mut Realm) -> Res<String> {
        let opts = string_rounding_mode_opts(opts, realm)?;

        self.time
            .to_ixdtf_string(opts)
            .map_err(Error::from_temporal)
    }

    #[prop("toLocaleString")]
    pub fn to_locale_string(&self) -> Res<String> {
        self.time
            .to_ixdtf_string(ToStringRoundingOptions::default())
            .map_err(Error::from_temporal)
    }

    #[prop("toPlainDateTime")]
    pub fn to_plain_date_time(&self, date: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let date = value_to_plain_date(date, realm)?;

        let plain_date_time = temporal_rs::PlainDateTime::from_date_and_time(date, self.time)
            .map_err(Error::from_temporal)?;

        Ok(PlainDateTime::new(plain_date_time, realm).into_object())
    }

    pub fn until(
        &self,
        other: Value,
        opts: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let other_time = value_to_plain_time(other, realm)?;

        let opts = opts
            .map(|opts| difference_settings(opts, realm))
            .transpose()?
            .unwrap_or_default();

        let duration = self
            .time
            .until(&other_time, opts)
            .map_err(Error::from_temporal)?;

        Ok(Duration::with_duration(realm, duration).into_object())
    }

    #[prop("valueOf")]
    #[nonstatic]
    pub const fn value_of() -> Res<()> {
        Err(Error::ty("Called valueOf on a Temporal.PlainTime object"))
    }

    #[get("hour")]
    pub const fn hour(&self) -> u8 {
        self.time.hour()
    }

    #[get("minute")]
    pub const fn minute(&self) -> u8 {
        self.time.minute()
    }

    #[get("second")]
    pub const fn second(&self) -> u8 {
        self.time.second()
    }

    #[get("millisecond")]
    pub const fn millisecond(&self) -> u16 {
        self.time.millisecond()
    }

    #[get("microsecond")]
    pub const fn microsecond(&self) -> u16 {
        self.time.microsecond()
    }

    #[get("nanosecond")]
    pub const fn nanosecond(&self) -> u16 {
        self.time.nanosecond()
    }
}

pub fn value_to_plain_time(info: Value, realm: &mut Realm) -> Res<temporal_rs::PlainTime> {
    match info {
        Value::String(str) => {
            let time = temporal_rs::PlainTime::from_str(&str).map_err(Error::from_temporal)?;

            Ok(time)
        }
        Value::Object(obj) => {
            if let Some(plain_time) = obj.downcast::<PlainTime>() {
                return Ok(plain_time.time);
            }

            if let Some(plain_date_time) = obj.downcast::<PlainDateTime>() {
                return plain_date_time
                    .date
                    .to_plain_time()
                    .map_err(Error::from_temporal);
            }

            let hour = obj.get("hour", realm).and_then(|v| v.to_number(realm))? as u8;

            let minute = obj.get("minute", realm).and_then(|v| v.to_number(realm))? as u8;

            let second = obj.get("second", realm).and_then(|v| v.to_number(realm))? as u8;

            let millisecond = obj
                .get("millisecond", realm)
                .and_then(|v| v.to_number(realm))? as u16;

            let microsecond = obj
                .get("microsecond", realm)
                .and_then(|v| v.to_number(realm))? as u16;

            let nanosecond = obj
                .get("nanosecond", realm)
                .and_then(|v| v.to_number(realm))? as u16;

            temporal_rs::PlainTime::new(hour, minute, second, millisecond, microsecond, nanosecond)
                .map_err(Error::from_temporal)
        }

        _ => Err(Error::ty(
            "Expected a string or an object representing PlainTime",
        )),
    }
}


impl PrettyObjectOverride for PlainTime {
    fn pretty_inline(&self, obj: &Object<Realm>, not: &mut Vec<usize>) -> Option<String> {
        let mut s = self.time
            .to_ixdtf_string(ToStringRoundingOptions::default()).ok()?;
            

        fmt_properties_to(obj, &mut s, not);

        Some(s)
    }
}


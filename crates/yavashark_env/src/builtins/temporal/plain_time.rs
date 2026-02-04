use crate::builtins::temporal::duration::{value_to_duration, Duration};
use crate::builtins::temporal::plain_date_time::PlainDateTime;
use crate::builtins::temporal::utils::{
    difference_settings, overflow_options, string_rounding_mode_opts, value_to_partial_time,
};
use crate::native_obj::NativeObject;
use crate::print::{fmt_properties_to, PrettyObjectOverride};
use crate::value::{Obj, Object};
use crate::{Error, ObjectHandle, Realm, Res, Value};
use temporal_rs::options::ToStringRoundingOptions;
use temporal_rs::{Temporal, TimeZone};
use yavashark_macro::props;

#[derive(Debug)]
pub struct PlainTime {
    pub(crate) time: temporal_rs::PlainTime,
}

impl PlainTime {
    pub fn new(time: temporal_rs::PlainTime, realm: &mut Realm) -> Res<NativeObject<Self>> {
        NativeObject::new(Self { time }, realm)
    }

    pub fn now(tz: Option<TimeZone>) -> Res<temporal_rs::PlainTime> {
        Temporal::now()
            .plain_time_iso(tz)
            .map_err(Error::from_temporal)
    }

    pub fn now_obj(realm: &mut Realm, tz: Option<TimeZone>) -> Res<NativeObject<Self>> {
        let time = Self::now(tz)?;

        Self::new(time, realm)
    }
}

#[props(intrinsic_name = temporal_plain_time, to_string_tag = "Temporal.PlainTime", constructor_length = 1)]
impl PlainTime {
    #[constructor]
    pub fn construct(
        #[realm] realm: &mut Realm,
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

        Ok(Self::new(time, realm)?.into_object())
    }

    pub fn compare(left: Value, right: Value, #[realm] realm: &mut Realm) -> Res<i8> {
        let left_time = value_to_plain_time(left, realm)?;
        let right_time = value_to_plain_time(right, realm)?;

        let result = left_time.cmp(&right_time);

        Ok(result as i8)
    }

    pub fn from(value: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let time = value_to_plain_time(value, realm)?;

        Ok(Self::new(time, realm)?.into_object())
    }

    pub fn add(&self, dur: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let dur = value_to_duration(dur, realm)?;

        let new_time = self.time.add(&dur).map_err(Error::from_temporal)?;

        Ok(Self::new(new_time, realm)?.into_object())
    }

    pub fn equals(&self, other: Value, #[realm] realm: &mut Realm) -> Res<bool> {
        let other_time = value_to_plain_time(other, realm)?;

        Ok(self.time == other_time)
    }

    pub fn round(&self, _opts: Option<Value>, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        //TODO
        Ok(Self::new(self.time, realm)?.into_object())
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

        Ok(Duration::with_duration(realm, duration)?.into_object())
    }

    pub fn subtract(&self, dur: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let dur = value_to_duration(dur, realm)?;

        let new_time = self.time.subtract(&dur).map_err(Error::from_temporal)?;

        Ok(Self::new(new_time, realm)?.into_object())
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

        Ok(Duration::with_duration(realm, duration)?.into_object())
    }

    #[prop("valueOf")]
    #[nonstatic]
    pub const fn value_of() -> Res<()> {
        Err(Error::ty("Called valueOf on a Temporal.PlainTime object"))
    }

    fn with(&self, other: &ObjectHandle, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let overflow = overflow_options(other, realm)?;
        let (partial_time, _) = value_to_partial_time(other, false, realm)?;

        let date = self
            .time
            .with(partial_time, overflow)
            .map_err(Error::from_temporal)?;

        Ok(Self::new(date, realm)?.into_object())
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
            let time = str.parse().map_err(Error::from_temporal)?;

            Ok(time)
        }
        Value::Object(obj) => {
            if let Some(plain_time) = obj.downcast::<NativeObject<PlainTime>>() {
                return Ok(plain_time.time);
            }

            if let Some(plain_date_time) = obj.downcast::<NativeObject<PlainDateTime>>() {
                return Ok(plain_date_time.date.to_plain_time());
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
    fn pretty_inline(
        &self,
        obj: &Object,
        not: &mut Vec<usize>,
        realm: &mut Realm,
    ) -> Option<String> {
        let mut s = self
            .time
            .to_ixdtf_string(ToStringRoundingOptions::default())
            .ok()?;

        fmt_properties_to(obj, &mut s, not, realm);

        Some(s)
    }
}
